use serde::Deserialize;

use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RawStreamEvent {
    MessageDelta {
        message_id: String,
        text: String,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
    },
    TaskProgress {
        task_id: String,
        state: String,
        label: String,
    },
    Clarification {
        clarification_id: String,
        prompt: String,
    },
    ArtifactPresented {
        artifact_id: String,
        name: String,
    },
    RuntimeStatus {
        state: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterEvent {
    Deerflow(RawStreamEvent),
    Hermes(RawStreamEvent),
}

pub fn load_stream_fixture(fixture_json: &str) -> Result<Vec<AdapterEvent>, RawSourceError> {
    let events: Vec<RawStreamEvent> =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    Ok(events.into_iter().map(AdapterEvent::Deerflow).collect())
}
