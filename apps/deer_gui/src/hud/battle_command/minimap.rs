//! Minimap HUD presentation.
//!
//! Renders the tactical minimap panel, reading camera sync state
//! to display the current viewport frustum and allowing click-to-locate
//! navigation requests.

use bevy::log::trace;

use crate::camera::navigation::CameraSyncSnapshot;

/// Draws the minimap panel based on the current camera sync snapshot.
pub fn draw_minimap_ui(snapshot: &CameraSyncSnapshot) {
    trace!(
        "draw_minimap: center={:?} size={:?} zoom={:.2}",
        snapshot.frustum_center,
        snapshot.frustum_size,
        snapshot.zoom,
    );
}

/// System that sends navigation requests when the minimap is clicked.
///
/// In a full implementation this would read UI interaction events and
/// emit `ViewportNavigationRequest` messages.
pub fn minimap_interaction_system() {
    trace!("minimap_interaction: awaiting UI event wiring");
}
