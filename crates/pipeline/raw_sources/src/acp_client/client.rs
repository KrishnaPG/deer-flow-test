use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use bytes::{Buf, BytesMut};

use agent_client_protocol::{self as acp, Agent as _};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::task::{spawn_local, LocalSet};
use tokio::time::{timeout, Duration};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::instrument;

use crate::acp_client::fanout::AcpResponseStreamFanout;
use crate::acp_client::notification_mapper::map_session_notification_to_live_event;
use crate::acp_client::registry::{
    AcpChatRunRegistry, AcpChatRunState, AcpChatSessionRegistry, AcpChatSessionState,
};
use crate::acp_client::subprocess::{
    spawn_acp_subprocess, AcpSubprocessCommand,
};
use crate::acp_client::{
    AcpCaptureContextStore, ChatRunId, ChatSessionId,
};
use crate::acp_client::{AcpResponseStreamEvent, AcpResponseStreamEventKind};

use crate::acp_client::raw_publisher::{RawEventPublisher, NoopRawEventPublisher};
use crate::acp_client::raw_fanout::RawEventFanout;

const ACP_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for establishing an ACP client session.
///
/// Provider-agnostic: the client can launch any ACP-compatible agent binary.
/// Use `AcpClientSessionConfig::discover()` to auto-discover the binary from environment
/// or PATH, or construct manually with specific binary path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpClientSessionConfig {
    /// Path to ACP-compatible agent binary.
    pub executable: PathBuf,
    /// Arguments to pass to the agent (e.g., ["acp"] for hermes).
    pub args: Vec<String>,
    /// Working directory for the subprocess.
    pub working_directory: PathBuf,
    /// Optional provider name for display/logging purposes.
    pub provider_name: Option<String>,
}

impl AcpClientSessionConfig {
    /// Create a new configuration with explicit binary path.
    pub fn new(executable: PathBuf) -> Self {
        Self {
            executable,
            args: Vec::new(),
            working_directory: std::env::current_dir().unwrap(),
            provider_name: None,
        }
    }

    /// Set arguments to pass to the binary (e.g., ["acp"]).
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set working directory for the subprocess.
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = dir;
        self
    }

    /// Set provider name for display/logging.
    pub fn with_provider_name(mut self, name: impl Into<String>) -> Self {
        self.provider_name = Some(name.into());
        self
    }

    /// Auto-discover the ACP agent binary.
    ///
    /// First checks `$ACP_AGENT_BIN` environment variable (supports "path arg1 arg2" format).
    /// Then searches PATH for known ACP providers (hermes, etc.) with their ACP subcommands.
    /// Returns error if no agent found.
    pub fn discover() -> Result<Self, std::io::Error> {
        // 1. Check environment variable override
        //    Format: just a path, or "path arg1 arg2" for binary + args
        if let Ok(val) = std::env::var("ACP_AGENT_BIN") {
            if !val.is_empty() {
                let mut parts = val.split_whitespace();
                let path = PathBuf::from(parts.next().unwrap());
                let args: Vec<String> = parts.map(String::from).collect();
                if path.exists() {
                    let config = Self::new(path).with_provider_name("env-override");
                    return Ok(if args.is_empty() { config } else { config.with_args(args) });
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("$ACP_AGENT_BIN path does not exist: {}", path.display()),
                ));
            }
        }

        // 2. Search PATH for known ACP providers with their ACP subcommands
        let known_providers: &[(&str, &[&str])] = &[("hermes", &["acp"])];
        for (provider, default_args) in known_providers {
            if let Ok(path) = which(provider) {
                if std::fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false) {
                    let args: Vec<String> = default_args.iter().map(|s| s.to_string()).collect();
                    return Ok(Self::new(path).with_provider_name(*provider).with_args(args));
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No ACP agent found. Set $ACP_AGENT_BIN or ensure 'hermes' is in PATH.".to_string(),
        ))
    }
}

/// Simple which() implementation for non-Unix systems.
fn which(name: &str) -> Result<PathBuf, std::io::Error> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let candidate = PathBuf::from(dir).join(name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    // Fallback: try direct path
    let direct = PathBuf::from(name);
    if direct.exists() {
        return Ok(direct);
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("{} not found in PATH", name),
    ))
}

/// Thin ACP client wrapper around the Rust `agent-client-protocol` crate.
#[derive(Clone)]
pub struct OurAcpClient {
    fanout: AcpResponseStreamFanout,
    raw_publisher: Arc<dyn RawEventPublisher>,
    raw_fanout: RawEventFanout,
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
    session_sequences: Arc<DashMap<ChatSessionId, Arc<AtomicU64>>>,
}

impl Default for OurAcpClient {
    fn default() -> Self {
        Self::new(Arc::new(NoopRawEventPublisher), RawEventFanout::default())
    }
}

impl OurAcpClient {
    pub fn new(raw_publisher: Arc<dyn RawEventPublisher>, raw_fanout: RawEventFanout) -> Self {
        Self {
            fanout: AcpResponseStreamFanout::default(),
            raw_publisher,
            raw_fanout,
            capture_contexts: Arc::new(AcpCaptureContextStore::default()),
            chat_session_registry: Arc::new(AcpChatSessionRegistry::default()),
            chat_run_registry: Arc::new(AcpChatRunRegistry::default()),
            session_connections: Arc::new(DashMap::new()),
            session_sequences: Arc::new(DashMap::new()),
        }
    }

    pub fn fanout(&self) -> &AcpResponseStreamFanout {
        &self.fanout
    }

    pub fn raw_fanout(&self) -> &RawEventFanout {
        &self.raw_fanout
    }

    #[instrument(skip(self))]
    pub async fn connect_session(
        &self,
        config: AcpClientSessionConfig,
    ) -> Result<ChatSessionId, std::io::Error> {
        let command = AcpSubprocessCommand::new(config.executable, config.working_directory).with_args(config.args);
        let subprocess = spawn_acp_subprocess(&command).await?;
        let acp_subprocess_id = subprocess.acp_subprocess_id.clone();
        
        let mut child = subprocess.child;
        let stderr = child.stderr.take().ok_or_else(missing_stderr_error)?;
        let stdin = child.stdin.take().ok_or_else(missing_stdin_error)?;
        let stdout = child.stdout.take().ok_or_else(missing_stdout_error)?;

        // Create duplex pipes to feed the acp crate.
        // The acp crate reads from acp_stdout_rx; our pipe task writes to acp_stdout_tx.
        let (acp_stdout_tx, acp_stdout_rx) = tokio::io::duplex(65536);

        // Pipe hermes stdout → acp duplex so the ACP crate can read responses.
        // This must start before the handshake so the acp crate gets data.
        // We use a placeholder session ID; it will be replaced after handshake.
        let pre_session_id = ChatSessionId::pending_for_subprocess(&acp_subprocess_id);
        let sequence_counter = Arc::new(AtomicU64::new(1));

        // Pipe stdout: hermes → acp_stdout_tx (for ACP crate) + raw capture
        tokio::spawn(run_raw_stdout_capture_loop(
            pre_session_id.clone(),
            Arc::clone(&sequence_counter),
            BufReader::new(stdout),
            acp_stdout_tx,
            Arc::clone(&self.raw_publisher) as Arc<_>,
            self.raw_fanout.clone(),
        ));

        // Pipe stderr: hermes → raw capture
        tokio::spawn(run_raw_stderr_capture_loop(
            pre_session_id.clone(),
            Arc::clone(&sequence_counter),
            stderr,
            Arc::clone(&self.raw_publisher) as Arc<_>,
            self.raw_fanout.clone(),
        ));

        // Perform the ACP handshake (initialize + new_session)
        let capture_context = self.capture_contexts.initialize_for_subprocess(acp_subprocess_id.clone());

        let local_set = LocalSet::new();
        let acp_session_id = local_set
            .run_until(perform_acp_handshake(
                self.fanout.clone(),
                capture_context,
                Arc::clone(&self.capture_contexts),
                self.chat_session_registry.clone(),
                self.chat_run_registry.clone(),
                Arc::clone(&self.session_connections),
                stdin,
                acp_stdout_rx,
                child,
            ))
            .await?;

        // Derive our canonical ChatSessionId from the ACP session response.
        // This is the same scheme as before: blake3(first_event_content) + pid + timestamp.
        let pid = std::process::id();
        let timestamp_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let chat_session_id = ChatSessionId::from_first_event(
            acp_session_id.as_str().as_bytes(),
            pid,
            timestamp_ns,
        );

        // Promote sequence counter from placeholder to canonical session ID.
        if let Some((_, counter)) = self.session_sequences.remove(&pre_session_id) {
            self.session_sequences.insert(chat_session_id.clone(), counter);
        }

        // Update capture context to map subprocess → canonical session.
        self.capture_contexts.assign_session(&acp_subprocess_id, chat_session_id.clone());

        Ok(chat_session_id)
    }

    #[instrument(skip(self, prompt_blocks))]
    pub async fn start_prompt_run(
        &self,
        chat_session_id: &ChatSessionId,
        prompt_blocks: Vec<acp::ContentBlock>,
    ) -> Result<ChatRunId, std::io::Error> {
        let chat_run_id = ChatRunId::generate();
        self.chat_run_registry.insert(AcpChatRunState {
            chat_run_id: chat_run_id.clone(),
            chat_session_id: chat_session_id.clone(),
        });

        let acp_subprocess_id = self
            .chat_session_registry
            .subprocess_id_for(chat_session_id)
            .ok_or_else(|| missing_session_error(chat_session_id))?;
        self.capture_contexts
            .assign_run(&acp_subprocess_id, chat_run_id.clone());

        let session_connection = self
            .session_connections
            .get(chat_session_id)
            .ok_or_else(|| missing_session_error(chat_session_id))?;
        let request = acp::PromptRequest::new(chat_session_id.as_str().to_string(), prompt_blocks);

        // Intentionally bypassing manual client command capture since it gets captured
        // inside the protocol stream or we'd just tee stdin. We will tee stdin in a real prod path.
        // For now the raw bytes come from the responses.

        session_connection
            .prompt(request)
            .await
            .map_err(io_error_from_acp)?;

        self.publish_run_started(chat_session_id.clone(), chat_run_id.clone());
        Ok(chat_run_id)
    }

    pub fn publish_run_started(&self, chat_session_id: ChatSessionId, chat_run_id: ChatRunId) {
        self.fanout.publish(AcpResponseStreamEvent::new(
            chat_session_id,
            chat_run_id,
            AcpResponseStreamEventKind::RunStarted,
        ));
    }
}

async fn run_raw_stdout_capture_loop(
    session_id: ChatSessionId,
    sequence: Arc<AtomicU64>,
    mut stdout: BufReader<ChildStdout>,
    mut acp_tx: tokio::io::DuplexStream,
    publisher: Arc<dyn RawEventPublisher>,
    fanout: RawEventFanout,
) {
    // Raw byte pipe: read whatever the kernel gives us and forward as-is.
    // No line splitting, no String interpretation, no serde.
    // The ACP crate parses its own protocol from the duplex stream.
    // redb stores opaque byte chunks; downstream consumers parse them.
    let mut buf = BytesMut::with_capacity(8192);
    loop {
        buf.clear();
        match stdout.read_buf(&mut buf).await {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => {}
        }

        if buf.is_empty() {
            continue;
        }

        let seq = sequence.fetch_add(1, Ordering::Relaxed);
        let raw = buf.copy_to_bytes(buf.len());

        // Forward raw bytes to ACP crate duplex for protocol parsing.
        // The ACP crate reads line-delimited JSON-RPC from this stream internally.
        if acp_tx.write_all(&raw).await.is_err() {
            break;
        }

        // Publish to durable buffer — redb internally uses spawn_blocking.
        // Clone here is a reference count increment on a shared allocation (O(1)).
        let _ = publisher.publish_raw_event(&session_id, seq, raw.clone()).await;
        // Fan out the same Bytes — no copy, just another ref count increment.
        fanout.publish(session_id.clone(), seq, raw);
    }
}

async fn run_raw_stderr_capture_loop(
    session_id: ChatSessionId,
    sequence: Arc<AtomicU64>,
    stderr: ChildStderr,
    publisher: Arc<dyn RawEventPublisher>,
    fanout: RawEventFanout,
) {
    let mut reader = BufReader::new(stderr);
    let mut buf = BytesMut::with_capacity(4096);
    loop {
        buf.clear();
        match reader.read_buf(&mut buf).await {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => {}
        }

        if buf.is_empty() {
            continue;
        }

        let seq = sequence.fetch_add(1, Ordering::Relaxed);
        let raw = buf.copy_to_bytes(buf.len());
        let _ = publisher.publish_raw_event(&session_id, seq, raw.clone()).await;
        fanout.publish(session_id.clone(), seq, raw);
    }
}

async fn perform_acp_handshake(
    fanout: AcpResponseStreamFanout,
    capture_context: crate::acp_client::AcpCaptureContext,
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::DuplexStream,
    mut mut_child: Child,
) -> Result<ChatSessionId, std::io::Error> {
    let (connection, handle_io) = acp::ClientSideConnection::new(
        AcpClientCallbacks {
            fanout,
            chat_run_registry: Arc::clone(&chat_run_registry),
        },
        stdin.compat_write(),
        stdout.compat(),
        |future| {
            spawn_local(future);
        },
    );
    spawn_local(handle_io);

    let connection = Arc::new(connection);

    tokio::spawn(async move {
        let _ = mut_child.wait().await;
    });

    // Perform handshake: initialize + new_session
    let chat_session_id = initialize_session_connection(Arc::clone(&connection)).await?;
    
    // Update capture context with session info
    capture_contexts.assign_session(&capture_context.acp_subprocess_id, chat_session_id.clone());
    let bootstrap_run_id = ChatRunId::bootstrap_for_session(&chat_session_id);
    capture_contexts.assign_run(&capture_context.acp_subprocess_id, bootstrap_run_id);
    chat_session_registry.insert(AcpChatSessionState {
        chat_session_id: chat_session_id.clone(),
        acp_subprocess_id: capture_context.acp_subprocess_id.clone(),
        chat_thread_id: None,
    });
    session_connections.insert(chat_session_id.clone(), connection);

    Ok(chat_session_id)
}

async fn initialize_session_connection(
    connection: Arc<acp::ClientSideConnection>,
) -> Result<ChatSessionId, std::io::Error> {
    let initialize_future = async {
        connection
            .initialize(
                acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
                    acp::Implementation::new("berg10-acp-client", env!("CARGO_PKG_VERSION"))
                        .title("Berg10 ACP Client"),
                ),
            )
            .await
            .map_err(io_error_from_acp)?;

        let current_directory = std::env::current_dir()?;
        let response = connection
            .new_session(acp::NewSessionRequest::new(current_directory))
            .await
            .map_err(io_error_from_acp)?;
        Ok::<ChatSessionId, std::io::Error>(ChatSessionId::new(response.session_id.to_string()))
    };

    match timeout(ACP_STARTUP_TIMEOUT, initialize_future).await {
        Ok(result) => result,
        Err(_) => Err(startup_timeout_error()),
    }
}

struct AcpClientCallbacks {
    fanout: AcpResponseStreamFanout,
    chat_run_registry: Arc<AcpChatRunRegistry>,
}

#[async_trait(?Send)]
impl acp::Client for AcpClientCallbacks {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> acp::Result<()> {
        let chat_session_id = ChatSessionId::new(args.session_id.to_string());
        let chat_run_id = self
            .chat_run_registry
            .latest_for_session(&chat_session_id)
            .map(|state| state.chat_run_id)
            .unwrap_or_else(|| ChatRunId::bootstrap_for_session(&chat_session_id));

        if let Some(event) = map_session_notification_to_live_event(&args, &chat_run_id) {
            self.fanout.publish(event);
        }
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Ok(())
    }
}

fn io_error_from_acp(error: acp::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, error.to_string())
}

fn missing_session_error(chat_session_id: &ChatSessionId) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("missing ACP session connection for {}", chat_session_id),
    )
}

fn startup_timeout_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::TimedOut, "ACP startup timed out")
}

fn missing_stderr_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "ACP subprocess stderr missing")
}

fn missing_stdin_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "ACP subprocess stdin missing")
}

fn missing_stdout_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "ACP subprocess stdout missing")
}

