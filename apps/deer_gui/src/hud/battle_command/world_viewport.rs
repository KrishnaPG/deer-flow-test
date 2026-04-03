//! World viewport HUD presentation.
//!
//! Renders the tactical viewport panel, reading camera state
//! to display the current view.

use bevy::log::trace;
use bevy::prelude::*;

use crate::camera::CinematicCamera;

/// Draws the world viewport panel.
pub fn draw_world_viewport_ui(query: Query<&CinematicCamera>) {
    for cam in &query {
        trace!(
            "draw_world_viewport: yaw={:.2} pitch={:.2} zoom={:.2}",
            cam.yaw_deg,
            cam.pitch_deg,
            cam.zoom,
        );
    }
}
