//! Battle-mode inspector panel renderer.
//!
//! Renders battle-specific inspector content when the battle command HUD
//! is active, replacing the legacy right inspector panel.

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::state::{BattleCommandHudState, ShellVisibilityTier};

/// Renders the battle command inspector panel.
///
/// Only draws content when visibility tier is >= Tier2.
pub fn battle_inspector_system(mut contexts: EguiContexts, bc_hud: Res<BattleCommandHudState>) {
    if bc_hud.visibility_tier == ShellVisibilityTier::Tier1 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!(
        "battle_inspector_system — tier: {:?}",
        bc_hud.visibility_tier
    );

    egui::SidePanel::right("battle_inspector")
        .resizable(true)
        .default_width(260.0)
        .min_width(200.0)
        .max_width(400.0)
        .show(ctx, |ui| {
            ui.heading("Battle Inspector");
            ui.separator();

            ui.label(format!("Visibility: {:?}", bc_hud.visibility_tier));
            ui.label(format!("Event rail: {:?}", bc_hud.event_rail));
            ui.label(format!("Fleet rail: {:?}", bc_hud.fleet_rail));
            ui.label(format!("Event badges: {}", bc_hud.event_badge_count));
            ui.label(format!("Fleet badges: {}", bc_hud.fleet_badge_count));
        });
}
