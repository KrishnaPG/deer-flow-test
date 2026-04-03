use bevy::prelude::*;

use crate::shell::ShellState;

use super::{EntityInspectorData, HudState, InspectorDetails};

pub fn shell_selection_to_hud_system(mut hud: ResMut<HudState>, shell: Res<ShellState>) {
    hud.selected_entity = shell
        .selection
        .primary
        .as_ref()
        .map(|selection| EntityInspectorData {
            entity_id: selection.canonical_id.clone(),
            display_name: selection.canonical_id.clone(),
            details: InspectorDetails::Mission {
                progress: 0.0,
                assigned_agents: 0,
                description: String::new(),
            },
        });
}

pub fn shell_prefill_to_hud_system(mut hud: ResMut<HudState>, shell: Res<ShellState>) {
    if let Some(target) = &shell.intent_prefill.target {
        hud.command_input = format!("target:{}", target.canonical_id);
    }
}
