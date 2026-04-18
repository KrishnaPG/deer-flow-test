use async_trait::async_trait;
use bytes::Bytes;

use crate::acp_client::captured_event::AcpCapturedProtocolEvent;

/// Logical durable stream destination for captured ACP protocol traffic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpDurableStreamName(String);

impl AcpDurableStreamName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Broker-agnostic partition key for one chat session stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpDurablePartitionKey(String);

impl AcpDurablePartitionKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Broker-agnostic message headers for one durable event publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpDurableEventHeaders {
    pairs: Vec<(&'static str, String)>,
}

impl AcpDurableEventHeaders {
    pub fn from_captured_event(event: &AcpCapturedProtocolEvent) -> Self {
        let mut pairs = Vec::with_capacity(5);
        pairs.push(("engine", event.engine_name.clone()));
        pairs.push(("schema_version", event.schema_version.clone()));
        pairs.push(("session_id", event.chat_session_id.to_string()));
        pairs.push(("run_id", event.chat_run_id.to_string()));
        pairs.push(("seq", event.acp_session_sequence_number.to_string()));
        Self { pairs }
    }

    pub fn as_pairs(&self) -> &[(&'static str, String)] {
        &self.pairs
    }
}

/// Broker-agnostic durable event payload prepared for publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpDurableEventRecord {
    pub stream_name: AcpDurableStreamName,
    pub partition_key: AcpDurablePartitionKey,
    pub headers: AcpDurableEventHeaders,
    pub payload: Bytes,
}

impl AcpDurableEventRecord {
    pub fn from_captured_event(
        stream_name: AcpDurableStreamName,
        event: &AcpCapturedProtocolEvent,
    ) -> Result<Self, AcpCapturedEventPublishError> {
        Ok(Self {
            partition_key: AcpDurablePartitionKey::new(event.chat_session_id.to_string()),
            headers: AcpDurableEventHeaders::from_captured_event(event),
            payload: serde_json::to_vec(event)
                .map(Bytes::from)
                .map_err(AcpCapturedEventPublishError::Serialization)?,
            stream_name,
        })
    }
}

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
    Serialization(serde_json::Error),
    #[error("captured event publication failed: {message}")]
    Transport { message: String },
}

/// In-memory no-op publisher used until a concrete broker transport is configured.
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

#[cfg(test)]
mod tests {
    use serde_json::value::to_raw_value;

    use super::*;
    use crate::acp_client::{
        AcpCapturedProtocolEvent, AcpProtocolFrameKind, AcpSessionSequenceNumber, AcpSubprocessId,
        ChatRunId, ChatSessionId,
    };

    #[test]
    fn durable_event_record_uses_session_partition_key() {
        let event = AcpCapturedProtocolEvent::new(
            ChatSessionId::from("session-1"),
            ChatRunId::from("run-1"),
            AcpSubprocessId::from("proc-1"),
            AcpSessionSequenceNumber::new(7),
            AcpProtocolFrameKind::AcpServerNotification,
            to_raw_value(&serde_json::json!({"jsonrpc": "2.0"})).unwrap(),
        );
        let record = AcpDurableEventRecord::from_captured_event(
            AcpDurableStreamName::new("hermes.l0_drop"),
            &event,
        )
        .unwrap();

        assert_eq!(record.partition_key.as_str(), "session-1");
    }
}
