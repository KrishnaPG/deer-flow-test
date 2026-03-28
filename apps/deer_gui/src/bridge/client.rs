//! Bridge client — spawns and communicates with the Python bridge process.
//!
//! All log output uses `bevy::log` macros for unified tracing.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::log::{debug, error, info, trace, warn};
use crossbeam_channel::{unbounded, Receiver, Sender};
use directories::ProjectDirs;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use super::commands::{InboundEnvelope, OutboundCommand};
use super::dispatch::dispatch_envelope;
use super::events::BridgeEvent;

/// Client handle for the Python bridge subprocess.
///
/// Communicates over stdin (JSON commands) / stdout (JSON events).
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

    /// Spawn the bridge subprocess and begin reading events.
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

        let default_config = manifest_dir.join("config.yaml");
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
            .stderr(Stdio::inherit());

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

    /// Spawn a background thread that reads stdout lines and dispatches events.
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
                    Ok(env) => dispatch_envelope(env, &events_tx),
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

    /// Access the event receiver channel.
    pub fn events(&self) -> &Receiver<BridgeEvent> {
        &self.events_rx
    }

    /// Send a JSON command to the bridge and return the request ID.
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

    /// List all threads.
    pub fn list_threads(&self) -> Result<String, String> {
        self.send_command("list_threads", json!({}))
    }

    /// Get a single thread by ID.
    pub fn get_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("get_thread", json!({ "thread_id": thread_id }))
    }

    /// Create a new thread.
    pub fn create_thread(&self) -> Result<String, String> {
        self.send_command("create_thread", json!({}))
    }

    /// Rename a thread.
    pub fn rename_thread(&self, thread_id: &str, title: &str) -> Result<String, String> {
        self.send_command(
            "rename_thread",
            json!({ "thread_id": thread_id, "title": title }),
        )
    }

    /// Delete a thread.
    pub fn delete_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("delete_thread", json!({ "thread_id": thread_id }))
    }

    /// List available models.
    pub fn list_models(&self) -> Result<String, String> {
        self.send_command("list_models", json!({}))
    }

    /// Send a chat message in a thread.
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

    /// Resolve an artifact's host path.
    pub fn resolve_artifact(&self, thread_id: &str, virtual_path: &str) -> Result<String, String> {
        self.send_command(
            "resolve_artifact",
            json!({ "thread_id": thread_id, "virtual_path": virtual_path }),
        )
    }
}
