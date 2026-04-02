use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LinkedShellState {
    pub selected: Option<String>,
    pub pinned: Vec<String>,
    pub drill_down_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkedShellAction {
    Select { source_record_id: String },
    Pin { source_record_id: String },
    OpenDrillDown { panel_target: &'static str },
    Exclude { source_record_id: String },
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
        LinkedShellAction::Exclude { source_record_id } => {
            if state.selected.as_deref() == Some(source_record_id.as_str()) {
                state.selected = None;
            }
            state.pinned.retain(|id| id != &source_record_id);
        }
    }

    state
}
