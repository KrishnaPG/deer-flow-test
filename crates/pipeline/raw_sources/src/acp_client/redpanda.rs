use crate::acp_client::captured_event::AcpCapturedProtocolEvent;
use crate::acp_client::publish::{AcpDurableEventHeaders, AcpDurableStreamName};

#[cfg(feature = "redpanda")]
use async_trait::async_trait;
#[cfg(feature = "redpanda")]
use crate::acp_client::publish::{
    AcpCapturedEventPublishError, AcpCapturedEventPublisher, AcpDurableEventRecord,
};

/// Initial Redpanda stream binding for Hermes ACP raw capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpRedpandaStreamBinding {
    stream_name: AcpDurableStreamName,
}

impl AcpRedpandaStreamBinding {
    pub fn hermes_l0_drop() -> Self {
        Self {
            stream_name: AcpDurableStreamName::new("hermes.l0_drop"),
        }
    }

    pub fn stream_name(&self) -> AcpDurableStreamName {
        self.stream_name.clone()
    }
}

/// Redpanda-specific header adapter layered over broker-agnostic headers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpRedpandaHeaders {
    inner: AcpDurableEventHeaders,
}

impl AcpRedpandaHeaders {
    pub fn from_captured_event(event: &AcpCapturedProtocolEvent) -> Self {
        Self {
            inner: AcpDurableEventHeaders::from_captured_event(event),
        }
    }

    pub fn as_pairs(&self) -> &[(&'static str, String)] {
        self.inner.as_pairs()
    }

    #[cfg(feature = "redpanda")]
    pub fn to_owned_headers(&self) -> rdkafka::message::OwnedHeaders {
        self.as_pairs().iter().fold(
            rdkafka::message::OwnedHeaders::new(),
            |headers, (key, value)| {
                headers.insert(rdkafka::message::Header {
                    key,
                    value: Some(value.as_bytes()),
                })
            },
        )
    }
}

/// Feature-gated Redpanda publisher implementation.
#[cfg(feature = "redpanda")]
pub struct RedpandaAcpCapturedEventPublisher {
    stream_binding: AcpRedpandaStreamBinding,
}

#[cfg(feature = "redpanda")]
impl RedpandaAcpCapturedEventPublisher {
    pub fn new(stream_binding: AcpRedpandaStreamBinding) -> Self {
        Self { stream_binding }
    }
}

#[cfg(feature = "redpanda")]
#[async_trait]
impl AcpCapturedEventPublisher for RedpandaAcpCapturedEventPublisher {
    async fn publish_captured_event(
        &self,
        event: &AcpCapturedProtocolEvent,
    ) -> Result<(), AcpCapturedEventPublishError> {
        let _record = AcpDurableEventRecord::from_captured_event(
            self.stream_binding.stream_name(),
            event,
        )?;
        Err(AcpCapturedEventPublishError::Transport {
            message: "redpanda transport not yet wired".to_string(),
        })
    }
}
