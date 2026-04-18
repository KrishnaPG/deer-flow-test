use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::acp_client::ids::{ChatRunId, ChatSessionId, ChatThreadId};

/// UI-facing live stream event projected from ACP-observed activity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpResponseStreamEvent {
    pub chat_session_id: ChatSessionId,
    pub chat_run_id: ChatRunId,
    pub chat_thread_id: Option<ChatThreadId>,
    pub emitted_at: DateTime<Utc>,
    pub kind: AcpResponseStreamEventKind,
}

impl AcpResponseStreamEvent {
    pub fn new(
        chat_session_id: ChatSessionId,
        chat_run_id: ChatRunId,
        kind: AcpResponseStreamEventKind,
    ) -> Self {
        Self {
            chat_session_id,
            chat_run_id,
            chat_thread_id: None,
            emitted_at: Utc::now(),
            kind,
        }
    }
}

/// Phase-1 live events projected from Hermes ACP fidelity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AcpResponseStreamEventKind {
    RunStarted,
    AssistantThoughtStatusChunk { text: String },
    ToolCallStarted { tool_name: String },
    ToolCallCompleted { tool_name: String },
    AssistantTextFinal { text: String },
    RunCompleted,
    RunCancelled,
    StreamError { message: String },
}
