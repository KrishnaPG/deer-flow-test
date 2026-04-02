use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{RecordId, RecordRef, ThreadId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentAction {
    SubmitMessage,
    StopRun,
    ResumeClarification,
    OpenArtifact,
    ApplyPolicyOverlay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntentCommand {
    pub command_id: RecordId,
    pub action: IntentAction,
    pub target: Option<RecordRef>,
    pub thread_id: Option<ThreadId>,
    pub payload: Value,
}

impl IntentCommand {
    pub fn new(
        command_id: RecordId,
        action: IntentAction,
        target: Option<RecordRef>,
        thread_id: Option<ThreadId>,
        payload: Value,
    ) -> Self {
        Self {
            command_id,
            action,
            target,
            thread_id,
            payload,
        }
    }
}
