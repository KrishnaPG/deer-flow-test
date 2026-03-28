use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use directories::ProjectDirs;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::{
    AppStateUpdate, ArtifactInfo, ChatMessage, ModelInfo, ThreadRecord, ThreadSummary, Usage,
};

#[derive(Debug, Clone)]
pub enum BridgeEvent {
    Ready,
    Response {
        request_id: Option<String>,
        response: BridgeResponse,
    },
    StreamMessage {
        thread_id: String,
        message: ChatMessage,
    },
    State {
        state: AppStateUpdate,
    },
    Suggestions {
        thread_id: String,
        suggestions: Vec<String>,
    },
    Done {
        thread_id: String,
        usage: Option<Usage>,
    },
    Error {
        message: String,
    },
    BridgeExited,
}

#[derive(Debug, Clone)]
pub enum BridgeResponse {
    Threads(Vec<ThreadSummary>),
    Thread(ThreadRecord),
    CreatedThread(ThreadRecord),
    Models(Vec<ModelInfo>),
    Renamed(ThreadSummary),
    Deleted(String),
    Artifact(ArtifactInfo),
    Ack,
    Raw(Value),
}

#[derive(Debug, Serialize)]
struct OutboundCommand<'a, T: Serialize> {
    id: &'a str,
    command: &'a str,
    payload: T,
}

#[derive(Debug, Deserialize)]
struct InboundEnvelope {
    kind: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    ok: Option<bool>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    error: Option<String>,
}

pub struct BridgeClient {
    stdin: Arc<Mutex<ChildStdin>>,
    events_rx: Receiver<BridgeEvent>,
    _child: Child,
}

impl BridgeClient {
    /// Resolve the Python binary to use.
    ///
    /// Priority:
    /// 1. `PYTHON_BIN` env var (explicit override)
    /// 2. `<repo_root>/.venv/bin/python3` (project virtualenv)
    /// 3. `python3` on PATH (system fallback)
    fn resolve_python(repo_root: &Path) -> String {
        if let Ok(explicit) = std::env::var("PYTHON_BIN") {
            info!("Using PYTHON_BIN override: {explicit}");
            return explicit;
        }

        let venv_python = repo_root.join(".venv/bin/python3");
        if venv_python.exists() {
            let path = venv_python.to_string_lossy().to_string();
            info!("Using virtualenv Python: {path}");
            return path;
        }

        warn!(
            "No .venv found at {}; falling back to system python3",
            venv_python.display()
        );
        "python3".to_string()
    }

    pub fn spawn() -> Result<Self, String> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "Failed to resolve repository root".to_string())?
            .to_path_buf();
        let bridge_path = manifest_dir.join("python/bridge.py");
        let python_bin = Self::resolve_python(&repo_root);

        info!("Repo root: {}", repo_root.display());
        info!("Bridge script: {}", bridge_path.display());

        let project_dirs = ProjectDirs::from("ai", "OpenCode", "deer-gui")
            .ok_or_else(|| "Failed to resolve app data directory".to_string())?;
        let app_home = std::env::var("DEER_GUI_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dirs.data_local_dir().to_path_buf());
        std::fs::create_dir_all(&app_home).map_err(|err| err.to_string())?;

        let deer_flow_home = std::env::var("DEER_FLOW_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| app_home.join("deer-flow-home"));
        std::fs::create_dir_all(&deer_flow_home).map_err(|err| err.to_string())?;

        info!("App home: {}", app_home.display());
        info!("Deer Flow home: {}", deer_flow_home.display());

        let default_config = repo_root.join("config.yaml");

        // Forward the DEER_GUI_LOG level to the Python bridge so both sides
        // honour the same env var for log verbosity.
        let log_level = std::env::var("DEER_GUI_LOG").unwrap_or_default();

        let mut command = Command::new(&python_bin);
        command
            .arg("-u")
            .arg(&bridge_path)
            .current_dir(&repo_root)
            .env("DEER_GUI_HOME", &app_home)
            .env("DEER_FLOW_HOME", &deer_flow_home)
            .env("DEER_GUI_REPO_ROOT", &repo_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()); // Python logs go to our stderr

        if !log_level.is_empty() {
            command.env("DEER_GUI_LOG", &log_level);
        }

        if std::env::var_os("DEER_FLOW_CONFIG_PATH").is_none() && default_config.exists() {
            info!("Using default config: {}", default_config.display());
            command.env("DEER_FLOW_CONFIG_PATH", &default_config);
        }

        info!(
            "Spawning Python bridge: {python_bin} -u {}",
            bridge_path.display()
        );

        let mut child = command
            .spawn()
            .map_err(|err| format!("Failed to start Python bridge with {python_bin}: {err}"))?;

        info!("Python bridge spawned (pid={})", child.id());

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture bridge stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture bridge stdout".to_string())?;

        let stdin = Arc::new(Mutex::new(stdin));
        let (events_tx, events_rx) = unbounded();

        Self::spawn_reader(stdout, events_tx);

        Ok(Self {
            stdin,
            events_rx,
            _child: child,
        })
    }

    fn spawn_reader(stdout: impl std::io::Read + Send + 'static, events_tx: Sender<BridgeEvent>) {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(err) => {
                        error!("Bridge stdout read error: {err}");
                        let _ = events_tx.send(BridgeEvent::Error {
                            message: format!("Bridge read error: {err}"),
                        });
                        break;
                    }
                };

                if line.trim().is_empty() {
                    continue;
                }

                trace!("bridge <- {line}");

                match serde_json::from_str::<InboundEnvelope>(&line) {
                    Ok(env) => Self::dispatch_envelope(env, &events_tx),
                    Err(err) => {
                        error!("Bridge protocol parse error: {err} — raw: {line}");
                        let _ = events_tx.send(BridgeEvent::Error {
                            message: format!("Bridge protocol error: {err}: {line}"),
                        });
                    }
                }
            }

            warn!("Bridge stdout reader exited");
            let _ = events_tx.send(BridgeEvent::BridgeExited);
        });
    }

    fn dispatch_envelope(env: InboundEnvelope, events_tx: &Sender<BridgeEvent>) {
        debug!(
            "Dispatch envelope: kind={}, id={:?}, event={:?}, ok={:?}",
            env.kind, env.id, env.event, env.ok
        );

        match env.kind.as_str() {
            "ready" => {
                info!("Python bridge ready");
                let _ = events_tx.send(BridgeEvent::Ready);
            }
            "response" => {
                let response = match (env.id, env.ok.unwrap_or(false), env.result, env.error) {
                    (Some(id), true, Some(result), _) => {
                        debug!("Response ok id={id}: {result}");
                        match parse_response(&result) {
                            Ok(result) => BridgeEvent::Response {
                                request_id: Some(id),
                                response: result,
                            },
                            Err(err) => {
                                error!("Failed to parse response payload: {err}");
                                BridgeEvent::Error { message: err }
                            }
                        }
                    }
                    (Some(id), true, None, _) => {
                        debug!("Response ack id={id}");
                        BridgeEvent::Response {
                            request_id: Some(id),
                            response: BridgeResponse::Ack,
                        }
                    }
                    (id, false, _, err) => {
                        let msg = err.unwrap_or_else(|| "Bridge command failed".to_string());
                        error!("Response error id={id:?}: {msg}");
                        BridgeEvent::Error { message: msg }
                    }
                    _ => {
                        error!("Malformed bridge response");
                        BridgeEvent::Error {
                            message: "Malformed bridge response".to_string(),
                        }
                    }
                };
                let _ = events_tx.send(response);
            }
            "event" => {
                let Some(event_name) = env.event else {
                    error!("Event envelope missing event name");
                    let _ = events_tx.send(BridgeEvent::Error {
                        message: "Missing event name".to_string(),
                    });
                    return;
                };

                debug!("Event: {event_name} id={:?}", env.id);

                let event = match event_name.as_str() {
                    "message" => env
                        .data
                        .and_then(|value| {
                            let thread_id = value.get("thread_id")?.as_str()?.to_string();
                            let msg_value = value.get("message")?.clone();
                            match serde_json::from_value::<ChatMessage>(msg_value.clone()) {
                                Ok(message) => {
                                    debug!(
                                        "Stream message: thread={thread_id} role={} len={}",
                                        message.role,
                                        message.content.len()
                                    );
                                    Some(BridgeEvent::StreamMessage { thread_id, message })
                                }
                                Err(err) => {
                                    error!(
                                        "Failed to deserialize ChatMessage: {err} — raw: {msg_value}"
                                    );
                                    None
                                }
                            }
                        })
                        .unwrap_or_else(|| BridgeEvent::Error {
                            message: "Invalid message event payload".to_string(),
                        }),
                    "state" => env
                        .data
                        .and_then(|value| {
                            match serde_json::from_value::<AppStateUpdate>(value.clone()) {
                                Ok(state) => {
                                    debug!(
                                        "State update: thread={} title={:?}",
                                        state.thread_id, state.title
                                    );
                                    Some(BridgeEvent::State { state })
                                }
                                Err(err) => {
                                    error!(
                                        "Failed to deserialize AppStateUpdate: {err} — raw: {value}"
                                    );
                                    None
                                }
                            }
                        })
                        .unwrap_or_else(|| BridgeEvent::Error {
                            message: "Invalid state event payload".to_string(),
                        }),
                    "suggestions" => env
                        .data
                        .and_then(|value| {
                            let thread_id = value.get("thread_id")?.as_str()?.to_string();
                            let suggestions = serde_json::from_value::<Vec<String>>(
                                value.get("suggestions")?.clone(),
                            )
                            .ok()?;
                            debug!(
                                "Suggestions: thread={thread_id} count={}",
                                suggestions.len()
                            );
                            Some(BridgeEvent::Suggestions {
                                thread_id,
                                suggestions,
                            })
                        })
                        .unwrap_or_else(|| BridgeEvent::Error {
                            message: "Invalid suggestions event payload".to_string(),
                        }),
                    "done" => env
                        .data
                        .and_then(|value| {
                            let thread_id = value.get("thread_id")?.as_str()?.to_string();
                            let usage = value
                                .get("usage")
                                .cloned()
                                .and_then(|usage| serde_json::from_value::<Usage>(usage).ok());
                            info!("Stream done: thread={thread_id} usage={usage:?}");
                            Some(BridgeEvent::Done { thread_id, usage })
                        })
                        .unwrap_or_else(|| BridgeEvent::Error {
                            message: "Invalid done event payload".to_string(),
                        }),
                    "error" => {
                        let msg = env
                            .data
                            .and_then(|value| {
                                value
                                    .get("message")
                                    .and_then(Value::as_str)
                                    .map(ToOwned::to_owned)
                            })
                            .unwrap_or_else(|| "Bridge stream error".to_string());
                        error!("Bridge stream error event: {msg}");
                        BridgeEvent::Error { message: msg }
                    }
                    _ => {
                        warn!("Unknown bridge event type: {event_name}");
                        BridgeEvent::Error {
                            message: format!("Unknown bridge event: {event_name}"),
                        }
                    }
                };
                let _ = events_tx.send(event);
            }
            other => {
                warn!("Unknown envelope kind: {other}");
                let _ = events_tx.send(BridgeEvent::Error {
                    message: format!("Unknown envelope kind: {other}"),
                });
            }
        }
    }

    pub fn events(&self) -> &Receiver<BridgeEvent> {
        &self.events_rx
    }

    fn send_command<T: Serialize>(&self, command: &str, payload: T) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let envelope = OutboundCommand {
            id: &id,
            command,
            payload,
        };
        let line = serde_json::to_string(&envelope).map_err(|err| err.to_string())?;

        debug!("Sending command: {command} id={id}");
        trace!("bridge -> {line}");

        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "Failed to lock bridge stdin".to_string())?;
        stdin.write_all(line.as_bytes()).map_err(|err| {
            error!("Failed to write to bridge stdin: {err}");
            err.to_string()
        })?;
        stdin.write_all(b"\n").map_err(|err| err.to_string())?;
        stdin.flush().map_err(|err| {
            error!("Failed to flush bridge stdin: {err}");
            err.to_string()
        })?;
        Ok(id)
    }

    pub fn list_threads(&self) -> Result<String, String> {
        self.send_command("list_threads", json!({}))
    }

    pub fn get_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("get_thread", json!({ "thread_id": thread_id }))
    }

    pub fn create_thread(&self) -> Result<String, String> {
        self.send_command("create_thread", json!({}))
    }

    pub fn rename_thread(&self, thread_id: &str, title: &str) -> Result<String, String> {
        self.send_command(
            "rename_thread",
            json!({ "thread_id": thread_id, "title": title }),
        )
    }

    pub fn delete_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("delete_thread", json!({ "thread_id": thread_id }))
    }

    pub fn list_models(&self) -> Result<String, String> {
        self.send_command("list_models", json!({}))
    }

    pub fn send_message(
        &self,
        thread_id: &str,
        text: &str,
        attachments: &[PathBuf],
        model_name: Option<&str>,
        mode: &str,
        reasoning_effort: Option<&str>,
    ) -> Result<String, String> {
        let attachments: Vec<String> = attachments
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        info!(
            "send_message: thread={thread_id} mode={mode} model={:?} text_len={} attachments={}",
            model_name,
            text.len(),
            attachments.len()
        );

        self.send_command(
            "send_message",
            json!({
                "thread_id": thread_id,
                "text": text,
                "attachments": attachments,
                "context": {
                    "model_name": model_name,
                    "mode": mode,
                    "reasoning_effort": reasoning_effort,
                }
            }),
        )
    }

    pub fn resolve_artifact(&self, thread_id: &str, virtual_path: &str) -> Result<String, String> {
        self.send_command(
            "resolve_artifact",
            json!({ "thread_id": thread_id, "virtual_path": virtual_path }),
        )
    }
}

fn parse_response(value: &Value) -> Result<BridgeResponse, String> {
    if let Some(threads) = value.get("threads") {
        debug!("Parsed response: Threads");
        return serde_json::from_value(threads.clone())
            .map(BridgeResponse::Threads)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("thread") {
        debug!("Parsed response: Thread");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::Thread)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("created_thread") {
        debug!("Parsed response: CreatedThread");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::CreatedThread)
            .map_err(|err| err.to_string());
    }
    if let Some(models) = value.get("models") {
        debug!("Parsed response: Models");
        return serde_json::from_value(models.clone())
            .map(BridgeResponse::Models)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("renamed_thread") {
        debug!("Parsed response: Renamed");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::Renamed)
            .map_err(|err| err.to_string());
    }
    if let Some(thread_id) = value.get("deleted_thread_id").and_then(Value::as_str) {
        debug!("Parsed response: Deleted({thread_id})");
        return Ok(BridgeResponse::Deleted(thread_id.to_string()));
    }
    if let Some(artifact) = value.get("artifact") {
        debug!("Parsed response: Artifact");
        return serde_json::from_value(artifact.clone())
            .map(BridgeResponse::Artifact)
            .map_err(|err| err.to_string());
    }

    debug!(
        "Parsed response: Raw — keys={:?}",
        value.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    Ok(BridgeResponse::Raw(value.clone()))
}
