use deer_runtime_read_models::{ChatDraftState, ClarificationState, OptimisticSendState};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComposerVm {
    pub mode: &'static str,
    pub send_state: &'static str,
    pub prompt_text: String,
    pub attachments: Vec<String>,
}

pub fn derive_composer_vm(state: &ChatDraftState) -> ComposerVm {
    ComposerVm {
        mode: if state.clarification_state != ClarificationState::None {
            "clarification_response"
        } else {
            "prompt"
        },
        send_state: match state.optimistic_send {
            OptimisticSendState::Idle => "idle",
            OptimisticSendState::SendingPrompt => "sending_prompt",
            OptimisticSendState::SendingClarificationResponse => "sending_clarification_response",
        },
        prompt_text: state.prompt_text.clone(),
        attachments: state.attachment_ids.clone(),
    }
}
