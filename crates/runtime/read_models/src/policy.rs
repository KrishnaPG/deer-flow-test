use serde::Serialize;

use crate::linked_shell::LinkedShellState;

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

pub fn apply_policy_invalidation_to_linked_shell(
    mut state: LinkedShellState,
    action: &PolicyAction,
) -> LinkedShellState {
    match action {
        PolicyAction::RecordInvalidated {
            source_record_id, ..
        } => {
            if state.selected.as_deref() == Some(source_record_id.as_str()) {
                state.selected = None;
            }
            state.pinned.retain(|id| id != source_record_id);
        }
    }

    state
}
