use bytes::Bytes;

use crate::acp_client::captured_event::AcpCapturedProtocolEvent;

/// Strongly typed Redpanda topic name for ACP raw protocol capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpRedpandaTopic(&'static str);

impl AcpRedpandaTopic {
    pub const HERMES_L0_DROP: Self = Self("hermes.l0_drop");

    pub fn as_str(&self) -> &str {
        self.0
    }
}

/// Minimal Redpanda headers derived from one captured ACP event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpRedpandaHeaders {
    pairs: Vec<(&'static str, String)>,
}

impl AcpRedpandaHeaders {
    pub fn from_captured_event(event: &AcpCapturedProtocolEvent) -> Self {
        let mut pairs = Vec::with_capacity(5);
        pairs.push(("engine", event.engine_name.clone()));
        pairs.push(("schema_version", event.schema_version.clone()));
        pairs.push(("session_id", event.chat_session_id.to_string()));
        pairs.push(("run_id", event.chat_run_id.to_string()));
        pairs.push(("seq", event.acp_session_sequence_number.to_string()));
        Self { pairs }
    }

    #[cfg(feature = "redpanda")]
    pub fn to_owned_headers(&self) -> rdkafka::message::OwnedHeaders {
        self.pairs.iter().fold(
            rdkafka::message::OwnedHeaders::new(),
            |headers, (key, value)| {
                headers.insert(rdkafka::message::Header {
                    key,
                    value: Some(value.as_bytes()),
                })
            },
        )
    }

    pub fn as_pairs(&self) -> &[(&'static str, String)] {
        &self.pairs
    }
}

pub fn captured_event_payload_bytes(
    event: &AcpCapturedProtocolEvent,
) -> Result<Bytes, serde_json::Error> {
    serde_json::to_vec(event).map(Bytes::from)
}
