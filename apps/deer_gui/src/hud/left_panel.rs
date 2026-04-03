//! Left panel HUD — mission list with progress bars and status indicators.
//!
//! Displays active missions in an RPG-style "party list" format,
//! each with a coloured status dot, progress bar, and agent count.

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::battle_command::{BattleCommandHudState, ShellVisibilityTier};
use super::styles::{draw_progress_bar, mission_status_color, side_panel_frame};
use super::{HudState, MissionStatus, MissionSummary};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the left panel with the mission list.
pub fn left_panel_system(
    mut contexts: EguiContexts,
    hud: Res<HudState>,
    bc_hud: Option<Res<BattleCommandHudState>>,
) {
    if let Some(bc) = &bc_hud {
        if bc.visibility_tier != ShellVisibilityTier::Tier1 {
            trace!("left_panel_system — battle mode active, skipping");
            return;
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!(
        "left_panel_system — rendering {} missions",
        hud.missions.len()
    );

    let frame = side_panel_frame(ctx);

    egui::SidePanel::left("hud_left_panel")
        .resizable(true)
        .default_width(220.0)
        .min_width(180.0)
        .max_width(320.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.heading("Missions");
            ui.separator();

            if hud.missions.is_empty() {
                ui.label(
                    egui::RichText::new("No active missions")
                        .color(egui::Color32::from_rgb(128, 128, 128))
                        .italics(),
                );
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for mission in &hud.missions {
                        render_mission_entry(ui, mission);
                        ui.add_space(4.0);
                    }
                });
        });
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Renders a single mission entry with status dot, name, progress bar, and agent count.
fn render_mission_entry(ui: &mut egui::Ui, mission: &MissionSummary) {
    let status_color = mission_status_color(&mission.status);

    ui.group(|ui| {
        ui.horizontal(|ui| {
            // Status dot
            let dot = egui::RichText::new("\u{25CF}")
                .color(status_color)
                .size(10.0);
            ui.label(dot);

            // Mission name
            ui.label(&mission.name);

            // Status label (right-aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(status_color, status_label(mission.status));
            });
        });

        // Progress bar
        draw_progress_bar(ui, mission.progress, status_color);

        // Agent count
        ui.label(
            egui::RichText::new(format!("{} agent(s)", mission.agent_count))
                .size(10.0)
                .color(egui::Color32::from_rgb(140, 145, 150)),
        );
    });
}

/// Returns a short human-readable label for a mission status.
fn status_label(status: MissionStatus) -> &'static str {
    match status {
        MissionStatus::Active => "ACTIVE",
        MissionStatus::Blocked => "BLOCKED",
        MissionStatus::Idle => "IDLE",
        MissionStatus::Completed => "DONE",
    }
}
