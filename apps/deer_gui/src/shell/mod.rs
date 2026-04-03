mod panel_participation;
mod plugin;
mod state;
mod systems;

pub use panel_participation::{
    InteractionKind, PanelId, PanelParticipation, PanelParticipationRegistry, PanelRole,
};
pub use plugin::ShellPlugin;
pub use state::{CanonicalEntityRef, CanonicalRecordFamily, ShellState};
pub use systems::{selection_broker_system, ShellSelectionRequest};
