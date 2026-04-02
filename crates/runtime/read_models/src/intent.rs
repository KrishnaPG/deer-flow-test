use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IntentDraftState {
    pub source_record_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IntentLifecycleState {
    pub stage: String,
    pub draft: IntentDraftState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentAction {
    SeedFromSelection { source_record_id: String },
    OpenComposer,
    TakeOwnership,
    Validate,
    Submit,
}

pub fn reduce_intent_state(
    mut state: IntentLifecycleState,
    action: IntentAction,
) -> IntentLifecycleState {
    match action {
        IntentAction::SeedFromSelection { source_record_id } => {
            state.stage = "prefill_seed".into();
            state.draft.source_record_id = Some(source_record_id);
        }
        IntentAction::OpenComposer => state.stage = "prefill".into(),
        IntentAction::TakeOwnership => state.stage = "draft".into(),
        IntentAction::Validate => state.stage = "validated".into(),
        IntentAction::Submit => state.stage = "submitted".into(),
    }

    state
}
