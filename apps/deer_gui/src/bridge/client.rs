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
use uuid::Uuid;

use super::commands::{InboundEnvelope, OutboundCommand};
use super::dispatch::dispatch_envelope;
use super::events::BridgeEvent;
use super::orchestrator::OrchestratorClient;
use super::payloads::{
    EmptyPayload, MessageContext, RenameThreadPayload, ResolveArtifactPayload, SendMessagePayload,
    ThreadIdPayload,
};
use super::transport::Transport;
use crate::constants::perf::LINE_BUFFER_CAPACITY;

// ---------------------------------------------------------------------------
// ProcessTransport — stdio-based Transport implementation
// ---------------------------------------------------------------------------

/// Transport implementation that communicates over subprocess stdio.
///
/// Wraps a `ChildStdin` (for sending) and a channel receiver (for receiving
/// lines parsed by the background reader thread).
pub(super) struct ProcessTransport {
    stdin: Mutex<ChildStdin>,
}

impl Transport for ProcessTransport {
    fn send_line(&self, line: &str) -> std::io::Result<()> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "stdin lock poisoned"))?;
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        Ok(())
    }

    fn recv_line(&self) -> std::io::Result<Option<String>> {
        // Receiving is handled by the background reader thread, not by this
        // transport directly.  This method exists for transports that own
        // their receive path (e.g. TCP, gRPC).  For the subprocess case the
        // reader thread pushes events via a channel instead.
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "ProcessTransport uses a background reader thread; call BridgeClient::events() instead",
        ))
    }
}

// ---------------------------------------------------------------------------
// BridgeClient
// ---------------------------------------------------------------------------

/// Client handle for the Python bridge subprocess.
///
/// Communicates over stdin (JSON commands) / stdout (JSON events).
/// Uses an [`Arc<dyn Transport>`] internally so the transport layer can be
/// swapped (e.g. for integration testing over TCP).
pub struct BridgeClient {
    transport: Arc<dyn Transport>,
    events_rx: Receiver<BridgeEvent>,
    _child: Child,
}

impl BridgeClient {
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

        let (app_home, deer_flow_home) = Self::resolve_directories(&manifest_dir)?;
        let default_config = manifest_dir.join("config.yaml");
        let log_level = std::env::var("DEER_GUI_LOG").unwrap_or_default();

        let mut command = Self::build_command(
            &python_bin,
            &bridge_path,
            &repo_root,
            &app_home,
            &deer_flow_home,
            &default_config,
            &log_level,
        );

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

        let transport: Arc<dyn Transport> = Arc::new(ProcessTransport {
            stdin: Mutex::new(stdin),
        });
        let (events_tx, events_rx) = unbounded();

        Self::spawn_reader(stdout, events_tx);

        Ok(Self {
            transport,
            events_rx,
            _child: child,
        })
    }

    /// Access the event receiver channel.
    pub fn events(&self) -> &Receiver<BridgeEvent> {
        &self.events_rx
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

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

    /// Resolve application and Deer Flow home directories.
    fn resolve_directories(manifest_dir: &Path) -> Result<(PathBuf, PathBuf), String> {
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

        // Suppress unused variable warning — manifest_dir is used for context
        let _ = manifest_dir;

        Ok((app_home, deer_flow_home))
    }

    /// Build the subprocess [`Command`] with all required env vars.
    fn build_command(
        python_bin: &str,
        bridge_path: &Path,
        repo_root: &Path,
        app_home: &Path,
        deer_flow_home: &Path,
        default_config: &Path,
        log_level: &str,
    ) -> Command {
        let mut command = Command::new(python_bin);
        command
            .arg("-u")
            .arg(bridge_path)
            .current_dir(repo_root)
            .env("DEER_GUI_HOME", app_home)
            .env("DEER_FLOW_HOME", deer_flow_home)
            .env("DEER_GUI_REPO_ROOT", repo_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if !log_level.is_empty() {
            command.env("DEER_GUI_LOG", log_level);
        }

        if std::env::var_os("DEER_FLOW_CONFIG_PATH").is_none() && default_config.exists() {
            info!("Using default config: {}", default_config.display());
            command.env("DEER_FLOW_CONFIG_PATH", default_config);
        }

        command
    }

    /// Spawn a background thread that reads stdout lines and dispatches events.
    ///
    /// Uses a reusable `String` buffer ([`LINE_BUFFER_CAPACITY`] initial bytes)
    /// instead of `BufReader::lines()` to avoid allocating a new `String` for
    /// every line read from the subprocess.
    fn spawn_reader(stdout: impl std::io::Read + Send + 'static, events_tx: Sender<BridgeEvent>) {
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line_buf = String::with_capacity(LINE_BUFFER_CAPACITY);

            loop {
                line_buf.clear();
                match reader.read_line(&mut line_buf) {
                    Ok(0) => {
                        // EOF — subprocess closed stdout.
                        trace!("spawn_reader — EOF on bridge stdout");
                        break;
                    }
                    Ok(_) => {
                        let line = line_buf.trim();
                        if line.is_empty() {
                            continue;
                        }

                        trace!("bridge <- {line}");

                        match serde_json::from_str::<InboundEnvelope>(line) {
                            Ok(env) => dispatch_envelope(env, &events_tx),
                            Err(err) => {
                                error!("Bridge protocol parse error: {err} — raw: {line}");
                                let _ = events_tx.send(BridgeEvent::Error {
                                    message: format!("Bridge protocol error: {err}: {line}"),
                                });
                            }
                        }
                    }
                    Err(err) => {
                        error!("Bridge stdout read error: {err}");
                        let _ = events_tx.send(BridgeEvent::Error {
                            message: format!("Bridge read error: {err}"),
                        });
                        break;
                    }
                }
            }

            warn!("Bridge stdout reader exited");
            let _ = events_tx.send(BridgeEvent::BridgeExited);
        });
    }

    /// Send a JSON command via the transport and return the request ID.
    fn send_command<T: Serialize>(&self, command: &str, payload: T) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let envelope = OutboundCommand {
            id: &id,
            command,
            payload,
        };
        let line = serde_json::to_string(&envelope).map_err(|err| err.to_string())?;

        debug!("Sending command: {command} id={id}");
        if line.len() <= 512 {
            trace!("bridge -> {line}");
        } else {
            trace!("bridge -> {}…({} bytes)", &line[..512], line.len());
        }

        self.transport.send_line(&line).map_err(|err| {
            error!("Failed to send command via transport: {err}");
            err.to_string()
        })?;

        Ok(id)
    }
}

// ---------------------------------------------------------------------------
// OrchestratorClient implementation
// ---------------------------------------------------------------------------

impl OrchestratorClient for BridgeClient {
    fn list_threads(&self) -> Result<String, String> {
        self.send_command("list_threads", EmptyPayload)
    }

    fn get_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("get_thread", ThreadIdPayload { thread_id })
    }

    fn create_thread(&self) -> Result<String, String> {
        self.send_command("create_thread", EmptyPayload)
    }

    fn rename_thread(&self, thread_id: &str, title: &str) -> Result<String, String> {
        self.send_command("rename_thread", RenameThreadPayload { thread_id, title })
    }

    fn delete_thread(&self, thread_id: &str) -> Result<String, String> {
        self.send_command("delete_thread", ThreadIdPayload { thread_id })
    }

    fn list_models(&self) -> Result<String, String> {
        self.send_command("list_models", EmptyPayload)
    }

    fn send_message(
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
            SendMessagePayload {
                thread_id,
                text,
                attachments,
                context: MessageContext {
                    model_name,
                    mode,
                    reasoning_effort,
                },
            },
        )
    }

    fn resolve_artifact(&self, thread_id: &str, virtual_path: &str) -> Result<String, String> {
        self.send_command(
            "resolve_artifact",
            ResolveArtifactPayload {
                thread_id,
                virtual_path,
            },
        )
    }
}
