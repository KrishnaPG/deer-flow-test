use bevy::prelude::*;

use super::panel_participation::PanelId;
use super::state::{CanonicalEntityRef, ShellState};

#[derive(Message, Debug, Clone)]
pub struct ShellSelectionRequest {
    pub next: CanonicalEntityRef,
    pub source: PanelId,
}

pub fn selection_broker_system(
    mut requests: MessageReader<ShellSelectionRequest>,
    mut shell: ResMut<ShellState>,
) {
    for request in requests.read() {
        shell.selection.primary = Some(request.next.clone());
        shell.selection.ordered.clear();
        shell.selection.ordered.push(request.next.clone());
    }
}
