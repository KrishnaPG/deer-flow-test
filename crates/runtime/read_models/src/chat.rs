use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum ClarificationState {
    #[default]
    None,
    Requested,
    Responding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum OptimisticSendState {
    #[default]
    Idle,
    SendingPrompt,
    SendingClarificationResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ChatDraftState {
    pub prompt_text: String,
    pub attachment_ids: Vec<String>,
    pub clarification_state: ClarificationState,
    pub optimistic_send: OptimisticSendState,
    pub stream_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAction {
    AddAttachment { attachment_id: String },
    PromptSendStarted,
    ClarificationRequested,
    ClarificationResponseStarted,
    ClarificationResolved,
    SendCompleted,
    StreamStateChanged { state: String },
}

pub fn reduce_chat_state(mut state: ChatDraftState, action: ChatAction) -> ChatDraftState {
    match action {
        ChatAction::AddAttachment { attachment_id } => state.attachment_ids.push(attachment_id),
        ChatAction::PromptSendStarted => {
            state.clarification_state = ClarificationState::None;
            state.optimistic_send = OptimisticSendState::SendingPrompt;
        }
        ChatAction::ClarificationRequested => {
            state.clarification_state = ClarificationState::Requested;
            state.optimistic_send = OptimisticSendState::Idle;
        }
        ChatAction::ClarificationResponseStarted
            if state.clarification_state == ClarificationState::Requested =>
        {
            state.clarification_state = ClarificationState::Responding;
            state.optimistic_send = OptimisticSendState::SendingClarificationResponse;
        }
        ChatAction::ClarificationResolved
            if state.clarification_state == ClarificationState::Responding =>
        {
            state.clarification_state = ClarificationState::None;
            state.optimistic_send = OptimisticSendState::Idle;
        }
        ChatAction::SendCompleted => {
            state.optimistic_send = OptimisticSendState::Idle;
        }
        ChatAction::StreamStateChanged { state: next } => state.stream_state = Some(next),
        ChatAction::ClarificationResponseStarted | ChatAction::ClarificationResolved => {}
    }

    state
}
