use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use bytes::Bytes;

use agent_client_protocol::{self as acp, Agent as _};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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

/// Configuration for establishing one Hermes ACP client session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpClientSessionConfig {
    pub executable: PathBuf,
    pub working_directory: PathBuf,
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
        let command = AcpSubprocessCommand::new(config.executable, config.working_directory);
        let subprocess = spawn_acp_subprocess(&command).await?;
        let acp_subprocess_id = subprocess.acp_subprocess_id.clone();
        
        let mut child = subprocess.child;
        let stderr = child.stderr.take().ok_or_else(missing_stderr_error)?;
        let stdin = child.stdin.take().ok_or_else(missing_stdin_error)?;
        let stdout = child.stdout.take().ok_or_else(missing_stdout_error)?;

        // Intercept stdout to compute session ID from the very first line
        let mut stdout_reader = BufReader::new(stdout);
        let mut first_line = String::new();
        stdout_reader.read_line(&mut first_line).await?;
        if first_line.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "subprocess stdout closed before first line"));
        }
        let first_line_bytes = first_line.as_bytes();
        let timestamp_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let pid = child.id().unwrap_or(0);
        let chat_session_id = ChatSessionId::from_first_event(first_line_bytes, pid, timestamp_ns);

        let sequence_counter = Arc::new(AtomicU64::new(1));
        self.session_sequences.insert(chat_session_id.clone(), Arc::clone(&sequence_counter));

        // Create duplex pipes to feed the acp crate
        let (acp_stdout_tx, acp_stdout_rx) = tokio::io::duplex(65536);

        // Spawn raw tee capture for stdout
        tokio::spawn(run_raw_stdout_capture_loop(
            chat_session_id.clone(),
            Arc::clone(&sequence_counter),
            first_line.clone(),
            stdout_reader,
            acp_stdout_tx,
            Arc::clone(&self.raw_publisher),
            self.raw_fanout.clone(),
        ));

        // Spawn raw tee capture for stderr
        tokio::spawn(run_raw_stderr_capture_loop(
            chat_session_id.clone(),
            Arc::clone(&sequence_counter),
            stderr,
            Arc::clone(&self.raw_publisher),
            self.raw_fanout.clone(),
        ));

        // We can just use the acp_stdout_rx and stdin directly.
        let mut capture_context = self.capture_contexts.initialize_for_subprocess(acp_subprocess_id.clone());
        capture_context.chat_session_id.clone_from(&chat_session_id); // update context

        let local_set = LocalSet::new();
        local_set
            .run_until(connect_session_local(
                self.fanout.clone(),
                Arc::clone(&self.capture_contexts),
                Arc::clone(&self.chat_session_registry),
                Arc::clone(&self.chat_run_registry),
                Arc::clone(&self.session_connections),
                capture_context,
                chat_session_id.clone(),
                stdin,
                acp_stdout_rx,
                child,
            ))
            .await?;

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
    first_line: String,
    stdout_reader: BufReader<ChildStdout>,
    mut acp_tx: tokio::io::DuplexStream,
    publisher: Arc<dyn RawEventPublisher>,
    fanout: RawEventFanout,
) {
    let handle_line = |line: String| {
        let bytes = Bytes::from(line);
        let seq = sequence.fetch_add(1, Ordering::SeqCst);
        let bytes_pub = bytes.clone();
        let bytes_fan = bytes.clone();
        let session_clone = session_id.clone();
        let session_clone2 = session_id.clone();
        let pub_clone = Arc::clone(&publisher);
        let fan_clone = fanout.clone();
        
        tokio::spawn(async move {
            let _ = tokio::join!(
                pub_clone.publish_raw_event(&session_clone, seq, bytes_pub),
                async move { fan_clone.publish(session_clone2, seq, bytes_fan); Ok::<(), ()>(()) }
            );
        });
        bytes
    };

    let first_bytes = handle_line(first_line);
    if let Err(_) = acp_tx.write_all(&first_bytes).await { return; }

    let mut lines = stdout_reader.lines();
    while let Ok(Some(mut line)) = lines.next_line().await {
        line.push('\n');
        let bytes = handle_line(line);
        if let Err(_) = acp_tx.write_all(&bytes).await { break; }
    }
}

async fn run_raw_stderr_capture_loop(
    session_id: ChatSessionId,
    sequence: Arc<AtomicU64>,
    stderr: ChildStderr,
    publisher: Arc<dyn RawEventPublisher>,
    fanout: RawEventFanout,
) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(mut line)) = lines.next_line().await {
        line.push('\n');
        let bytes = Bytes::from(line);
        let seq = sequence.fetch_add(1, Ordering::SeqCst);
        let bytes_pub = bytes.clone();
        let bytes_fan = bytes.clone();
        let session_clone = session_id.clone();
        let session_clone2 = session_id.clone();
        let pub_clone = Arc::clone(&publisher);
        let fan_clone = fanout.clone();
        
        tokio::spawn(async move {
            let _ = tokio::join!(
                pub_clone.publish_raw_event(&session_clone, seq, bytes_pub),
                async move { fan_clone.publish(session_clone2, seq, bytes_fan); Ok::<(), ()>(()) }
            );
        });
    }
}

async fn connect_session_local(
    fanout: AcpResponseStreamFanout,
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
    capture_context: crate::acp_client::AcpCaptureContext,
    chat_session_id: ChatSessionId,
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

    let _ = initialize_session_connection(Arc::clone(&connection), &capture_context).await?;
    register_connected_session(
        capture_contexts,
        chat_session_registry,
        chat_run_registry,
        session_connections,
        capture_context,
        chat_session_id.clone(),
        connection,
    );
    Ok(chat_session_id)
}

async fn initialize_session_connection(
    connection: Arc<acp::ClientSideConnection>,
    _capture_context: &crate::acp_client::AcpCaptureContext,
) -> Result<ChatSessionId, std::io::Error> {
    let initialize_future = async {
        connection
            .initialize(
                acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
                    acp::Implementation::new("deer-acp-client", env!("CARGO_PKG_VERSION"))
                        .title("Deer ACP Client"),
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

fn register_connected_session(
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
    capture_context: crate::acp_client::AcpCaptureContext,
    chat_session_id: ChatSessionId,
    connection: Arc<acp::ClientSideConnection>,
) {
    capture_contexts.assign_session(&capture_context.acp_subprocess_id, chat_session_id.clone());
    let bootstrap_run_id = ChatRunId::bootstrap_for_session(&chat_session_id);
    capture_contexts.assign_run(&capture_context.acp_subprocess_id, bootstrap_run_id.clone());
    chat_session_registry.insert(AcpChatSessionState {
        chat_session_id: chat_session_id.clone(),
        acp_subprocess_id: capture_context.acp_subprocess_id.clone(),
        chat_thread_id: None,
    });
    chat_run_registry.insert(AcpChatRunState {
        chat_run_id: bootstrap_run_id,
        chat_session_id: chat_session_id.clone(),
    });
    session_connections.insert(chat_session_id, connection);
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

