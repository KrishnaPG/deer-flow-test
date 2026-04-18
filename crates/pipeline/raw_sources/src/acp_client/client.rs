use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{self as acp, Agent as _};
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr};
use tokio::task::{spawn_local, LocalSet};
use tokio::time::{timeout, Duration};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::instrument;

use crate::acp_client::durable_handoff::AcpDurableEventHandoff;
use crate::acp_client::fanout::AcpResponseStreamFanout;
use crate::acp_client::notification_mapper::map_session_notification_to_live_event;
use crate::acp_client::publish::{AcpCapturedEventPublisher, NoopAcpCapturedEventPublisher};
use crate::acp_client::registry::{
    AcpChatRunRegistry, AcpChatRunState, AcpChatSessionRegistry, AcpChatSessionState,
};
use crate::acp_client::subprocess::{
    spawn_acp_subprocess, AcpSubprocessCommand, AcpSubprocessLifecycleKind,
};
use crate::acp_client::{
    AcpCaptureContextStore, AcpSequenceAllocator, ChatRunId, ChatSessionId,
};
use crate::acp_client::{AcpSubprocessId, AcpResponseStreamEvent, AcpResponseStreamEventKind};

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
    durable_handoff: Arc<AcpDurableEventHandoff>,
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
}

impl Default for OurAcpClient {
    fn default() -> Self {
        let publisher: Arc<dyn AcpCapturedEventPublisher> = Arc::new(NoopAcpCapturedEventPublisher);
        let sequence_allocator = Arc::new(AcpSequenceAllocator::default());
        Self {
            fanout: AcpResponseStreamFanout::default(),
            durable_handoff: Arc::new(AcpDurableEventHandoff::new(
                Arc::clone(&publisher),
                Arc::clone(&sequence_allocator),
            )),
            capture_contexts: Arc::new(AcpCaptureContextStore::default()),
            chat_session_registry: Arc::new(AcpChatSessionRegistry::default()),
            chat_run_registry: Arc::new(AcpChatRunRegistry::default()),
            session_connections: Arc::new(DashMap::new()),
        }
    }
}

impl OurAcpClient {
    pub fn fanout(&self) -> &AcpResponseStreamFanout {
        &self.fanout
    }

    #[instrument(skip(self))]
    pub async fn connect_session(
        &self,
        config: AcpClientSessionConfig,
    ) -> Result<ChatSessionId, std::io::Error> {
        let command = AcpSubprocessCommand::new(config.executable, config.working_directory);
        let subprocess = spawn_acp_subprocess(&command).await?;
        let capture_context = self
            .capture_contexts
            .initialize_for_subprocess(subprocess.acp_subprocess_id.clone());
        publish_spawned_lifecycle(&self.durable_handoff, &capture_context).await?;

        let mut child = subprocess.child;
        let stderr = child.stderr.take().ok_or_else(missing_stderr_error)?;
        let stdin = child.stdin.take().ok_or_else(missing_stdin_error)?;
        let stdout = child.stdout.take().ok_or_else(missing_stdout_error)?;

        let local_set = LocalSet::new();
        let chat_session_id = local_set
            .run_until(connect_session_local(
                self.fanout.clone(),
                Arc::clone(&self.durable_handoff),
                Arc::clone(&self.capture_contexts),
                Arc::clone(&self.chat_session_registry),
                Arc::clone(&self.chat_run_registry),
                Arc::clone(&self.session_connections),
                capture_context,
                stdin,
                stdout,
                stderr,
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

        self.durable_handoff
            .publish_client_control(
                chat_session_id.clone(),
                chat_run_id.clone(),
                acp_subprocess_id,
                &request,
            )
            .await
            .map_err(io_error_from_publish)?;

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

async fn connect_session_local(
    fanout: AcpResponseStreamFanout,
    durable_handoff: Arc<AcpDurableEventHandoff>,
    capture_contexts: Arc<AcpCaptureContextStore>,
    chat_session_registry: Arc<AcpChatSessionRegistry>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    session_connections: Arc<DashMap<ChatSessionId, Arc<acp::ClientSideConnection>>>,
    capture_context: crate::acp_client::AcpCaptureContext,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    stderr: ChildStderr,
    child: Child,
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
    spawn_capture_tasks(
        Arc::clone(&capture_contexts),
        Arc::clone(&durable_handoff),
        Arc::clone(&chat_run_registry),
        connection.subscribe(),
        capture_context.clone(),
        stderr,
        child,
    );

    let chat_session_id = initialize_session_connection(Arc::clone(&connection), &capture_context, &durable_handoff).await?;
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

async fn publish_spawned_lifecycle(
    durable_handoff: &AcpDurableEventHandoff,
    capture_context: &crate::acp_client::AcpCaptureContext,
) -> Result<(), std::io::Error> {
    durable_handoff
        .publish_subprocess_lifecycle(
            capture_context.chat_session_id.clone(),
            capture_context.chat_run_id.clone(),
            capture_context.acp_subprocess_id.clone(),
            AcpSubprocessLifecycleKind::Spawned,
        )
        .await
        .map_err(io_error_from_publish)
}

fn spawn_capture_tasks(
    capture_contexts: Arc<AcpCaptureContextStore>,
    durable_handoff: Arc<AcpDurableEventHandoff>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    stream_receiver: acp::StreamReceiver,
    capture_context: crate::acp_client::AcpCaptureContext,
    stderr: ChildStderr,
    child: Child,
) {
    spawn_local(run_stream_capture_loop(
        capture_contexts,
        Arc::clone(&durable_handoff),
        chat_run_registry,
        stream_receiver,
    ));
    spawn_local(run_stderr_capture_loop(
        capture_context.chat_session_id.clone(),
        capture_context.chat_run_id.clone(),
        capture_context.acp_subprocess_id.clone(),
        Arc::clone(&durable_handoff),
        stderr,
    ));
    spawn_local(run_subprocess_exit_loop(
        capture_context.chat_session_id,
        capture_context.chat_run_id,
        capture_context.acp_subprocess_id,
        durable_handoff,
        child,
    ));
}

async fn initialize_session_connection(
    connection: Arc<acp::ClientSideConnection>,
    capture_context: &crate::acp_client::AcpCaptureContext,
    durable_handoff: &AcpDurableEventHandoff,
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
        Err(_) => {
            durable_handoff
                .publish_subprocess_lifecycle(
                    capture_context.chat_session_id.clone(),
                    capture_context.chat_run_id.clone(),
                    capture_context.acp_subprocess_id.clone(),
                    AcpSubprocessLifecycleKind::TimedOut,
                )
                .await
                .map_err(io_error_from_publish)?;
            Err(startup_timeout_error())
        }
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

fn io_error_from_publish(
    error: crate::acp_client::AcpCapturedEventPublishError,
) -> std::io::Error {
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

async fn run_stream_capture_loop(
    capture_contexts: Arc<AcpCaptureContextStore>,
    durable_handoff: Arc<AcpDurableEventHandoff>,
    chat_run_registry: Arc<AcpChatRunRegistry>,
    mut stream_receiver: acp::StreamReceiver,
) {
    while let Ok(message) = stream_receiver.recv().await {
        let Some(context) = first_context(&capture_contexts) else {
            continue;
        };
        let chat_run_id = chat_run_registry
            .latest_for_session(&context.chat_session_id)
            .map(|state| state.chat_run_id)
            .unwrap_or(context.chat_run_id.clone());
        let _ = durable_handoff
            .publish_stream_message(
                context.chat_session_id,
                chat_run_id,
                context.acp_subprocess_id,
                &message,
            )
            .await;
    }
}

async fn run_stderr_capture_loop(
    chat_session_id: ChatSessionId,
    chat_run_id: ChatRunId,
    acp_subprocess_id: AcpSubprocessId,
    durable_handoff: Arc<AcpDurableEventHandoff>,
    stderr: ChildStderr,
) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let _ = durable_handoff
            .publish_client_control(
                chat_session_id.clone(),
                chat_run_id.clone(),
                acp_subprocess_id.clone(),
                &serde_json::json!({
                    "stderr": line,
                }),
            )
            .await;
    }
}

async fn run_subprocess_exit_loop(
    chat_session_id: ChatSessionId,
    chat_run_id: ChatRunId,
    acp_subprocess_id: AcpSubprocessId,
    durable_handoff: Arc<AcpDurableEventHandoff>,
    mut child: Child,
) {
    let lifecycle_kind = match child.wait().await {
        Ok(status) if status.success() => AcpSubprocessLifecycleKind::Exited,
        Ok(_) => AcpSubprocessLifecycleKind::Crashed,
        Err(_) => AcpSubprocessLifecycleKind::Crashed,
    };

    let _ = durable_handoff
        .publish_subprocess_lifecycle(
            chat_session_id,
            chat_run_id,
            acp_subprocess_id,
            lifecycle_kind,
        )
        .await;
}

fn first_context(capture_contexts: &AcpCaptureContextStore) -> Option<crate::acp_client::AcpCaptureContext> {
    capture_contexts.first()
}
