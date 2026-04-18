use async_trait::async_trait;

use crate::acp_client::captured_event::AcpCapturedProtocolEvent;

/// Durable publication boundary for captured ACP protocol events.
#[async_trait]
pub trait AcpCapturedEventPublisher: Send + Sync {
    async fn publish_captured_event(
        &self,
        event: &AcpCapturedProtocolEvent,
    ) -> Result<(), AcpCapturedEventPublishError>;
}

/// Publication failure for one captured ACP event.
#[derive(Debug, thiserror::Error)]
pub enum AcpCapturedEventPublishError {
    #[error("captured event serialization failed")]
    Serialization(#[from] serde_json::Error),
    #[error("captured event publication failed: {message}")]
    Transport { message: String },
}

/// In-memory no-op publisher used until the concrete Redpanda transport is enabled.
#[derive(Default)]
pub struct NoopAcpCapturedEventPublisher;

#[async_trait]
impl AcpCapturedEventPublisher for NoopAcpCapturedEventPublisher {
    async fn publish_captured_event(
        &self,
        _event: &AcpCapturedProtocolEvent,
    ) -> Result<(), AcpCapturedEventPublishError> {
        Ok(())
    }
}
