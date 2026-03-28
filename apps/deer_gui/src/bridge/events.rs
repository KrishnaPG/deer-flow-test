//! Bridge event and response types.
//!
//! These enums represent the inbound event stream from the Python bridge
//! process and the parsed response payloads.

use crate::models::{
    AppStateUpdate, ArtifactInfo, ChatMessage, ModelInfo, ThreadRecord, ThreadSummary, Usage,
};
use serde_json::Value;

/// An event received from the Python bridge process.
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    Ready,
    Response {
        request_id: Option<String>,
        response: BridgeResponse,
    },
    StreamMessage {
        thread_id: String,
        message: ChatMessage,
    },
    State {
        state: AppStateUpdate,
    },
    Suggestions {
        thread_id: String,
        suggestions: Vec<String>,
    },
    Done {
        thread_id: String,
        usage: Option<Usage>,
    },
    Error {
        message: String,
    },
    BridgeExited,
}

/// A parsed response payload from a bridge command.
#[derive(Debug, Clone)]
pub enum BridgeResponse {
    Threads(Vec<ThreadSummary>),
    Thread(ThreadRecord),
    CreatedThread(ThreadRecord),
    Models(Vec<ModelInfo>),
    Renamed(ThreadSummary),
    Deleted(String),
    Artifact(ArtifactInfo),
    Ack,
    Raw(Value),
}
