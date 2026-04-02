use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LinkedShellPanelRole {
    Source,
    Sink,
    Mirror,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LinkedShellState {
    pub selected: Option<String>,
    pub pinned: Vec<String>,
    pub drill_down_target: Option<String>,
    pub panel_roles: BTreeMap<String, Vec<LinkedShellPanelRole>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkedShellAction {
    Select {
        source_record_id: String,
    },
    Pin {
        source_record_id: String,
    },
    OpenDrillDown {
        panel_target: &'static str,
    },
    PanelParticipationDeclared {
        panel_id: String,
        roles: Vec<LinkedShellPanelRole>,
    },
    LayoutPanelsRestored {
        panel_ids: Vec<String>,
    },
    Exclude {
        source_record_id: String,
    },
}

pub fn reduce_linked_shell_state(
    mut state: LinkedShellState,
    action: LinkedShellAction,
) -> LinkedShellState {
    match action {
        LinkedShellAction::Select { source_record_id } => state.selected = Some(source_record_id),
        LinkedShellAction::Pin { source_record_id } => state.pinned.push(source_record_id),
        LinkedShellAction::OpenDrillDown { panel_target } => {
            state.drill_down_target = Some(panel_target.into())
        }
        LinkedShellAction::PanelParticipationDeclared {
            panel_id,
            mut roles,
        } => {
            roles.sort();
            roles.dedup();
            state.panel_roles.insert(panel_id, roles);
        }
        LinkedShellAction::LayoutPanelsRestored { panel_ids } => {
            state
                .panel_roles
                .retain(|panel_id, _| panel_ids.iter().any(|id| id == panel_id));
            if state
                .drill_down_target
                .as_ref()
                .is_some_and(|panel_id| !panel_ids.iter().any(|id| id == panel_id))
            {
                state.drill_down_target = None;
            }
        }
        LinkedShellAction::Exclude { source_record_id } => {
            if state.selected.as_deref() == Some(source_record_id.as_str()) {
                state.selected = None;
            }
            state.pinned.retain(|id| id != &source_record_id);
        }
    }

    state
}
