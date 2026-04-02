use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct PolicyOverlayState {
    pub excluded_record_ids: Vec<String>,
    pub tombstoned_record_ids: Vec<String>,
    pub policy_epoch: u64,
    pub policy_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyAction {
    RecordInvalidated {
        source_record_id: String,
        policy_epoch: u64,
        policy_reason: String,
        tombstone_visible: bool,
    },
}

pub fn reduce_policy_state(
    mut state: PolicyOverlayState,
    action: PolicyAction,
) -> PolicyOverlayState {
    match action {
        PolicyAction::RecordInvalidated {
            source_record_id,
            policy_epoch,
            policy_reason,
            tombstone_visible,
        } => {
            state.policy_epoch = policy_epoch;
            state.policy_reason = Some(policy_reason);
            if !state
                .excluded_record_ids
                .iter()
                .any(|id| id == &source_record_id)
            {
                state.excluded_record_ids.push(source_record_id.clone());
            }
            if tombstone_visible {
                if !state
                    .tombstoned_record_ids
                    .iter()
                    .any(|id| id == &source_record_id)
                {
                    state.tombstoned_record_ids.push(source_record_id);
                }
            } else {
                state
                    .tombstoned_record_ids
                    .retain(|id| id != &source_record_id);
            }
        }
    }

    state
}
