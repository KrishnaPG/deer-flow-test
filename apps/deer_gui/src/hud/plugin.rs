//! [`HudPlugin`] — registers the HUD state resource and all panel systems.
//!
//! All HUD systems run during `Update` and depend on [`HudState`] being
//! available as a resource. The plugin initializes the state with defaults.

use bevy::log::info;
use bevy::prelude::*;

use super::bottom_console::bottom_console_system;
use super::center_canvas::center_canvas_system;
use super::event_ticker::event_ticker_system;
use super::left_panel::left_panel_system;
use super::modal::modal_system;
use super::right_inspector::right_inspector_system;
use super::top_bar::top_bar_system;
use super::HudState;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the HUD overlay subsystem.
///
/// * Inserts the [`HudState`] resource with default values.
/// * Adds all HUD panel systems to the `Update` schedule.
///
/// Panel render order matters for egui layout:
/// 1. Top bar (claims top edge)
/// 2. Bottom console (claims bottom edge)
/// 3. Left panel (claims left side of remaining area)
/// 4. Right inspector (claims right side of remaining area)
/// 5. Center canvas (fills remaining central area)
/// 6. Event ticker (floating overlay)
/// 7. Modal (foreground overlay, if active)
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        info!("HudPlugin::build — registering HUD resources and systems");

        app.init_resource::<HudState>().add_systems(
            Update,
            (
                top_bar_system,
                bottom_console_system,
                left_panel_system,
                right_inspector_system,
                center_canvas_system,
                event_ticker_system,
                modal_system,
            )
                .chain(),
        );
    }
}
