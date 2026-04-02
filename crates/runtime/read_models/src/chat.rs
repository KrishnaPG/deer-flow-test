use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ChatDraftState {
    pub prompt_text: String,
    pub attachment_ids: Vec<String>,
    pub clarification_pending: bool,
}
