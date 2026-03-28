//! Diagnostics plugin — registers frame-time and entity-count collectors.
//!
//! This is intentionally thin: it only wires Bevy's built-in diagnostic
//! plugins so that downstream systems (HUD, telemetry) can query them.

use bevy::app::{App, Plugin};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::log::{debug, info};

/// Bundles Bevy's built-in diagnostic collectors into a single plugin.
///
/// Currently registers:
/// - [`FrameTimeDiagnosticsPlugin`] — per-frame dt / FPS
/// - [`EntityCountDiagnosticsPlugin`] — live entity count
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        info!("DiagnosticsPlugin::build — registering diagnostic collectors");

        debug!("DiagnosticsPlugin::build — adding FrameTimeDiagnosticsPlugin");
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        debug!("DiagnosticsPlugin::build — adding EntityCountDiagnosticsPlugin");
        app.add_plugins(EntityCountDiagnosticsPlugin::default());

        info!("DiagnosticsPlugin::build — diagnostic collectors registered");
    }
}
