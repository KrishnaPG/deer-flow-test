use std::sync::Arc;

use serde::Serialize;

use crate::acp_client::captured_event::{AcpCapturedProtocolEvent, AcpProtocolFrameKind};
use crate::acp_client::control_capture::capture_client_control_event;
use crate::acp_client::publish::{AcpCapturedEventPublishError, AcpCapturedEventPublisher};
use crate::acp_client::stream_capture::capture_stream_message;
use crate::acp_client::{
    AcpSequenceAllocator, AcpSessionSequenceNumber, AcpSubprocessId, AcpSubprocessLifecycleKind,
    ChatRunId, ChatSessionId,
};

#[derive(Clone)]
pub struct AcpDurableEventHandoff {
    publisher: Arc<dyn AcpCapturedEventPublisher>,
    sequence_allocator: Arc<AcpSequenceAllocator>,
}

impl AcpDurableEventHandoff {
    pub fn new(
        publisher: Arc<dyn AcpCapturedEventPublisher>,
        sequence_allocator: Arc<AcpSequenceAllocator>,
    ) -> Self {
        Self {
            publisher,
            sequence_allocator,
        }
    }

    pub async fn publish_client_control<T>(
        &self,
        chat_session_id: ChatSessionId,
        chat_run_id: ChatRunId,
        acp_subprocess_id: AcpSubprocessId,
        payload: &T,
    ) -> Result<(), AcpCapturedEventPublishError>
    where
        T: Serialize,
    {
        let sequence_number = self.next_sequence(&chat_session_id);
        let event = capture_client_control_event(
            chat_session_id,
            chat_run_id,
            acp_subprocess_id,
            sequence_number,
            payload,
        )
        .map_err(AcpCapturedEventPublishError::Serialization)?;
        self.publisher.publish_captured_event(&event).await
    }

    pub async fn publish_stream_message(
        &self,
        chat_session_id: ChatSessionId,
        chat_run_id: ChatRunId,
        acp_subprocess_id: AcpSubprocessId,
        message: &agent_client_protocol::StreamMessage,
    ) -> Result<(), AcpCapturedEventPublishError> {
        let sequence_number = self.next_sequence(&chat_session_id);
        let event = capture_stream_message(
            chat_session_id,
            chat_run_id,
            acp_subprocess_id,
            sequence_number,
            message,
        )
        .map_err(AcpCapturedEventPublishError::Serialization)?;
        self.publisher.publish_captured_event(&event).await
    }

    pub async fn publish_subprocess_lifecycle(
        &self,
        chat_session_id: ChatSessionId,
        chat_run_id: ChatRunId,
        acp_subprocess_id: AcpSubprocessId,
        lifecycle_kind: AcpSubprocessLifecycleKind,
    ) -> Result<(), AcpCapturedEventPublishError> {
        let sequence_number = self.next_sequence(&chat_session_id);
        let payload = serde_json::value::to_raw_value(&serde_json::json!({
            "subprocessLifecycleKind": lifecycle_kind,
        }))
        .map_err(AcpCapturedEventPublishError::Serialization)?;
        let event = AcpCapturedProtocolEvent::new(
            chat_session_id,
            chat_run_id,
            acp_subprocess_id,
            sequence_number,
            AcpProtocolFrameKind::SubprocessLifecycle,
            payload,
        );
        self.publisher.publish_captured_event(&event).await
    }

    fn next_sequence(&self, chat_session_id: &ChatSessionId) -> AcpSessionSequenceNumber {
        self.sequence_allocator.next_for_session(chat_session_id)
    }
}
