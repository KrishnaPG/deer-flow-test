use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LayoutRuntimeReadModel {
    pub active_mode: Option<String>,
    pub broker_epochs: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutRuntimeAction {
    PresetLoaded {
        mode: String,
    },
    BrokerEpochChanged {
        interaction_type: String,
        epoch: u64,
    },
}

pub fn reduce_layout_runtime_state(
    mut state: LayoutRuntimeReadModel,
    action: LayoutRuntimeAction,
) -> LayoutRuntimeReadModel {
    match action {
        LayoutRuntimeAction::PresetLoaded { mode } => state.active_mode = Some(mode),
        LayoutRuntimeAction::BrokerEpochChanged {
            interaction_type,
            epoch,
        } => {
            state.broker_epochs.insert(interaction_type, epoch);
        }
    }

    state
}
