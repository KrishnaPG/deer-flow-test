use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{self as acp, Agent as _};
use async_trait::async_trait;
use tokio::task::{spawn_local, LocalSet};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::instrument;

use crate::acp_client::fanout::AcpResponseStreamFanout;
use crate::acp_client::notification_mapper::map_session_notification_to_live_event;
use crate::acp_client::publish::{AcpCapturedEventPublisher, NoopAcpCapturedEventPublisher};
use crate::acp_client::live_event::{AcpResponseStreamEvent, AcpResponseStreamEventKind};
use crate::acp_client::subprocess::{spawn_acp_subprocess, AcpSubprocessCommand};
use crate::acp_client::{
    AcpCapturedProtocolEvent, AcpProtocolFrameKind, AcpSequenceAllocator, ChatRunId,
    ChatSessionId,
};

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
    captured_event_publisher: Arc<dyn AcpCapturedEventPublisher>,
    sequence_allocator: Arc<AcpSequenceAllocator>,
}

impl Default for OurAcpClient {
    fn default() -> Self {
        Self {
            fanout: AcpResponseStreamFanout::default(),
            captured_event_publisher: Arc::new(NoopAcpCapturedEventPublisher),
            sequence_allocator: Arc::new(AcpSequenceAllocator::default()),
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
        let mut subprocess = spawn_acp_subprocess(&command).await?;
        let acp_subprocess_id = subprocess.acp_subprocess_id.clone();
        let stdin = subprocess.child.stdin.take().expect("stdin must be piped");
        let stdout = subprocess.child.stdout.take().expect("stdout must be piped");

        let sequence_allocator = Arc::clone(&self.sequence_allocator);
        let captured_event_publisher = Arc::clone(&self.captured_event_publisher);
        let local_set = LocalSet::new();
        let chat_session_id = local_set
            .run_until(async move {
                let (connection, handle_io) = acp::ClientSideConnection::new(
                    AcpClientCallbacks {
                        fanout: self.fanout.clone(),
                        captured_event_publisher,
                        sequence_allocator,
                        acp_subprocess_id,
                    },
                    stdin.compat_write(),
                    stdout.compat(),
                    |future| {
                        spawn_local(future);
                    },
                );
                spawn_local(handle_io);

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

                Ok::<ChatSessionId, std::io::Error>(ChatSessionId::new(
                    response.session_id.to_string(),
                ))
            })
            .await?;

        Ok(chat_session_id)
    }

    pub fn publish_run_started(&self, chat_session_id: ChatSessionId, chat_run_id: ChatRunId) {
        self.fanout.publish(AcpResponseStreamEvent::new(
            chat_session_id,
            chat_run_id,
            AcpResponseStreamEventKind::RunStarted,
        ));
    }
}

struct AcpClientCallbacks {
    fanout: AcpResponseStreamFanout,
    captured_event_publisher: Arc<dyn AcpCapturedEventPublisher>,
    sequence_allocator: Arc<AcpSequenceAllocator>,
    acp_subprocess_id: crate::acp_client::AcpSubprocessId,
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
        let chat_run_id = ChatRunId::generate();
        let sequence_number = self.sequence_allocator.next_for_session(&chat_session_id);
        let raw_payload = serde_json::value::to_raw_value(&args)
            .map_err(|error| acp::Error::internal_error().data(error.to_string()))?;
        let captured_event = AcpCapturedProtocolEvent::new(
            chat_session_id.clone(),
            chat_run_id.clone(),
            self.acp_subprocess_id.clone(),
            sequence_number,
            AcpProtocolFrameKind::AcpServerNotification,
            raw_payload,
        );

        self.captured_event_publisher
            .publish_captured_event(&captured_event)
            .await
            .map_err(|error| acp::Error::internal_error().data(error.to_string()))?;

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
