use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LayoutRuntimeReadModel {
    pub layout_instance: u64,
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
        LayoutRuntimeAction::PresetLoaded { mode } => {
            state.layout_instance = state.layout_instance.saturating_add(1);
            state.active_mode = Some(mode);
            state.broker_epochs.clear();
        }
        LayoutRuntimeAction::BrokerEpochChanged {
            interaction_type,
            epoch,
        } => {
            state.broker_epochs.insert(interaction_type, epoch);
        }
    }

    state
}
