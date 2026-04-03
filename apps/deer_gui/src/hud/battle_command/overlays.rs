//! Tier3 overlay system — blocks world navigation when active.
//!
//! When the shell is in Tier3 visibility, a full overlay is shown that
//! intercepts input and prevents world navigation while preserving
//! battle context.

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::state::{BattleCommandHudState, ShellVisibilityTier};

/// System that handles Tier3 overlay blocking world navigation.
///
/// When visibility_tier is Tier3, renders a blocking overlay and
/// sets overlay_blocks_world_navigation to true.
pub fn tier3_overlay_system(mut contexts: EguiContexts, mut bc_hud: ResMut<BattleCommandHudState>) {
    if bc_hud.visibility_tier != ShellVisibilityTier::Tier3 {
        bc_hud.overlay_blocks_world_navigation = false;
        return;
    }

    bc_hud.overlay_blocks_world_navigation = true;

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!("tier3_overlay_system — blocking world navigation");

    egui::Window::new("Battle Command Overlay")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .show(ctx, |ui| {
            ui.heading("Tier3 — Battle Command Overlay");
            ui.separator();
            ui.label("World navigation is paused.");
            ui.label("Battle context is preserved.");
        });
}
