use async_trait::async_trait;
use bytes::Bytes;

use crate::acp_client::ids::ChatSessionId;

#[async_trait]
pub trait RawEventPublisher: Send + Sync {
    async fn publish_raw_event(
        &self,
        session_id: &ChatSessionId,
        sequence: u64,
        raw_bytes: Bytes,
    ) -> Result<(), RawEventPublishError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RawEventPublishError {
    #[error("raw event publication failed: {message}")]
    Transport { message: String },
}

#[derive(Default)]
pub struct NoopRawEventPublisher;

#[async_trait]
impl RawEventPublisher for NoopRawEventPublisher {
    async fn publish_raw_event(
        &self,
        _session_id: &ChatSessionId,
        _sequence: u64,
        _raw_bytes: Bytes,
    ) -> Result<(), RawEventPublishError> {
        Ok(())
    }
}
