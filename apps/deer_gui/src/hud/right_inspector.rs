//! Right inspector HUD panel — selected entity details.
//!
//! When an entity is selected (via picking or programmatically), this
//! panel shows its type-specific details: agent state, mission progress,
//! or artifact metadata.

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::styles::{draw_progress_bar, side_panel_frame};
use super::{EntityInspectorData, HudState, InspectorDetails};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the right inspector panel with selected entity details.
pub fn right_inspector_system(mut contexts: EguiContexts, hud: Res<HudState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!("right_inspector_system — rendering");

    let frame = side_panel_frame(ctx);

    egui::SidePanel::right("hud_right_inspector")
        .resizable(true)
        .default_width(260.0)
        .min_width(200.0)
        .max_width(400.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            match &hud.selected_entity {
                Some(data) => render_entity_details(ui, data),
                None => {
                    ui.label(
                        egui::RichText::new("No entity selected")
                            .color(egui::Color32::from_rgb(128, 128, 128))
                            .italics(),
                    );
                }
            }
        });
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Renders the full details for a selected entity.
fn render_entity_details(ui: &mut egui::Ui, data: &EntityInspectorData) {
    ui.label(egui::RichText::new(&data.display_name).strong().size(14.0));
    ui.label(
        egui::RichText::new(format!("ID: {}", data.entity_id))
            .size(10.0)
            .color(egui::Color32::from_rgb(140, 145, 150)),
    );
    ui.separator();

    match &data.details {
        InspectorDetails::Agent {
            state,
            tokens_used,
            context_size,
            pending_actions,
        } => render_agent_details(ui, state, *tokens_used, *context_size, pending_actions),
        InspectorDetails::Mission {
            progress,
            assigned_agents,
            description,
        } => render_mission_details(ui, *progress, *assigned_agents, description),
        InspectorDetails::Artifact {
            artifact_type,
            size_bytes,
            path,
        } => render_artifact_details(ui, artifact_type, *size_bytes, path),
    }
}

/// Renders agent-specific inspector details.
fn render_agent_details(
    ui: &mut egui::Ui,
    state: &str,
    tokens_used: u64,
    context_size: u64,
    pending_actions: &[String],
) {
    ui.label(format!("State: {state}"));
    ui.label(format!("Tokens: {tokens_used}"));
    ui.label(format!("Context: {context_size}"));

    if !pending_actions.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Pending Actions").strong());
        for action in pending_actions {
            ui.horizontal(|ui| {
                ui.label(format!("  \u{2022} {action}"));
                if ui.small_button("Approve").clicked() {
                    trace!("right_inspector — approve action: {action}");
                }
                if ui.small_button("Reject").clicked() {
                    trace!("right_inspector — reject action: {action}");
                }
            });
        }
    }
}

/// Renders mission-specific inspector details.
fn render_mission_details(
    ui: &mut egui::Ui,
    progress: f32,
    assigned_agents: u32,
    description: &str,
) {
    ui.label(format!("Agents: {assigned_agents}"));
    ui.label(format!("Progress: {:.0}%", progress * 100.0));
    draw_progress_bar(ui, progress, egui::Color32::from_rgb(51, 230, 102));
    ui.separator();
    ui.label(description);
}

/// Renders artifact-specific inspector details.
fn render_artifact_details(ui: &mut egui::Ui, artifact_type: &str, size_bytes: u64, path: &str) {
    ui.label(format!("Type: {artifact_type}"));
    ui.label(format!("Size: {} bytes", size_bytes));
    ui.label(format!("Path: {path}"));
}
