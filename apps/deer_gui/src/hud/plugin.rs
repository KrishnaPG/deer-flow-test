//! [`HudPlugin`] — registers the HUD state resource and all panel systems.
//!
//! State-maintenance systems (mutation) run during `Update` so they execute
//! before the `EguiPrimaryContextPass` render systems that read the same
//! resource. This enforces a clean write-then-read ordering each frame.

use bevy::log::info;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use super::bottom_console::bottom_console_system;
use super::center_canvas::center_canvas_system;
use super::event_ticker::event_ticker_system;
use super::left_panel::left_panel_system;
use super::modal::modal_system;
use super::right_inspector::right_inspector_system;
use super::shell_sync::{shell_prefill_to_hud_system, shell_selection_to_hud_system};
use super::state_systems::{command_dispatch_system, event_ticker_maintenance_system};
use super::top_bar::top_bar_system;
use super::HudState;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the HUD overlay subsystem.
///
/// * Inserts the [`HudState`] resource with default values.
/// * Adds state-maintenance systems to `Update` (mutation phase).
/// * Adds all HUD panel systems to `EguiPrimaryContextPass` (render phase).
///
/// **State systems** (Update — before render):
/// 1. `event_ticker_maintenance_system` — ages/prunes event log
/// 2. `command_dispatch_system` — consumes pending commands
///
/// **Render systems** (EguiPrimaryContextPass — read-only where possible):
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

        app.init_resource::<HudState>()
            // State-maintenance systems run in Update, before EguiPrimaryContextPass.
            .add_systems(
                Update,
                (
                    event_ticker_maintenance_system,
                    command_dispatch_system,
                    shell_selection_to_hud_system,
                    shell_prefill_to_hud_system,
                ),
            )
            // Render systems run in EguiPrimaryContextPass, reading HudState.
            .add_systems(
                EguiPrimaryContextPass,
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
