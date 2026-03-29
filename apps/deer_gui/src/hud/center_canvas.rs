//! Center canvas HUD panel — switchable content area.
//!
//! Renders one of several views in the central area:
//! - WorldView (passthrough — no panel drawn)
//! - ExpertsMeeting (multi-agent conversation log)
//! - SwarmMonitor (aggregated particle/hex grid)
//! - ArtifactGraph (artifact lineage DAG)
//! - Forensics (debug/trace view)

use bevy::log::{debug, trace};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::styles::glass_panel_frame;
use super::{CenterCanvasMode, HudFragmentRegistry, HudState};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the center canvas with mode-specific content.
///
/// In `WorldView` mode, no panel is drawn — the 3D world shows through.
pub fn center_canvas_system(
    mut contexts: EguiContexts,
    hud: Res<HudState>,
    fragment_registry: Res<HudFragmentRegistry>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // WorldView = passthrough, nothing to render
    if hud.center_mode == CenterCanvasMode::WorldView {
        trace!("center_canvas_system — WorldView mode, skipping");
        return;
    }

    trace!("center_canvas_system — rendering {:?}", hud.center_mode);

    let frame = glass_panel_frame(ctx);

    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        // Mode selector tabs
        render_mode_tabs(ui, &hud);
        ui.separator();

        // Mode-specific content
        match hud.center_mode {
            CenterCanvasMode::WorldView => unreachable!(), // handled above
            CenterCanvasMode::ExpertsMeeting => render_experts_meeting(ui, &hud),
            CenterCanvasMode::SwarmMonitor => render_swarm_monitor(ui),
            CenterCanvasMode::ArtifactGraph => render_artifact_graph(ui),
            CenterCanvasMode::Forensics => render_forensics(ui),
        }

        // Render registered fragments after built-in content
        if !fragment_registry.is_empty() {
            ui.separator();
            render_fragment_tabs(ui, &fragment_registry);
        }
    });
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Renders the mode selector tabs at the top of the center canvas.
fn render_mode_tabs(ui: &mut egui::Ui, hud: &HudState) {
    ui.horizontal(|ui| {
        let modes = [
            (CenterCanvasMode::WorldView, "World"),
            (CenterCanvasMode::ExpertsMeeting, "Experts"),
            (CenterCanvasMode::SwarmMonitor, "Swarm"),
            (CenterCanvasMode::ArtifactGraph, "Artifacts"),
            (CenterCanvasMode::Forensics, "Forensics"),
        ];
        for (mode, label) in &modes {
            let is_active = hud.center_mode == *mode;
            let text = if is_active {
                egui::RichText::new(*label).strong()
            } else {
                egui::RichText::new(*label).color(egui::Color32::from_rgb(140, 145, 150))
            };
            if ui.selectable_label(is_active, text).clicked() {
                debug!("center_canvas — tab clicked: {:?}", mode);
                // Mode switch is deferred — the HudState is read-only here.
                // In a real implementation, this would send an event.
            }
        }
    });
}

/// Renders the Experts Meeting view — multi-agent conversation log.
fn render_experts_meeting(ui: &mut egui::Ui, hud: &HudState) {
    ui.heading("Experts Meeting");

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if let Some(thread) = &hud.thread_cache {
                for msg in &thread.messages {
                    render_chat_message(ui, &msg.role, &msg.content);
                }
            } else {
                ui.label(
                    egui::RichText::new("No thread selected")
                        .color(egui::Color32::from_rgb(128, 128, 128))
                        .italics(),
                );
            }
        });
}

/// Renders a single chat message bubble.
fn render_chat_message(ui: &mut egui::Ui, role: &str, content: &str) {
    let color = match role {
        "user" => egui::Color32::from_rgb(0, 204, 255),
        "assistant" => egui::Color32::from_rgb(51, 230, 102),
        _ => egui::Color32::from_rgb(180, 190, 200),
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(format!("[{role}]"))
                .color(color)
                .strong(),
        );
        ui.label(content);
    });
    ui.add_space(4.0);
}

/// Renders the Swarm Monitor view — placeholder for particle/hex grid.
fn render_swarm_monitor(ui: &mut egui::Ui) {
    ui.heading("Swarm Monitor");
    ui.label(
        egui::RichText::new("Aggregated swarm view — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

/// Renders the Artifact Graph view — placeholder for DAG visualization.
fn render_artifact_graph(ui: &mut egui::Ui) {
    ui.heading("Artifact Graph");
    ui.label(
        egui::RichText::new("Artifact lineage DAG — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

/// Renders the Forensics view — placeholder for debug trace output.
fn render_forensics(ui: &mut egui::Ui) {
    ui.heading("Forensics");
    ui.label(
        egui::RichText::new("Debug trace output — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

// ---------------------------------------------------------------------------
// Fragment rendering
// ---------------------------------------------------------------------------

/// Renders all registered HUD fragments as collapsible sections.
///
/// Each fragment is rendered with its title as a collapsible header,
/// and its render callback is invoked when expanded.
fn render_fragment_tabs(ui: &mut egui::Ui, registry: &HudFragmentRegistry) {
    ui.heading("External Fragments");
    ui.add_space(8.0);

    for fragment in registry.fragments() {
        ui.collapsing(&fragment.title, |ui| {
            (fragment.render)(ui);
        });
        ui.add_space(4.0);
    }
}
