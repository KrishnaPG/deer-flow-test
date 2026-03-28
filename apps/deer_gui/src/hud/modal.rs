//! Modal overlay system — full-screen Z2 modals for settings, tuning, etc.
//!
//! When `HudState::show_modal` is `Some(kind)`, this system renders
//! a dimmed backdrop with a centered modal window.

use bevy::log::{debug, trace};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{HudState, ModalKind};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders a modal overlay if one is active.
pub fn modal_system(mut contexts: EguiContexts, mut hud: ResMut<HudState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Some(kind) = hud.show_modal else {
        return;
    };

    trace!("modal_system — rendering {:?}", kind);

    // Dimmed backdrop
    let backdrop_id = egui::Id::new("modal_backdrop");
    egui::Area::new(backdrop_id)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen = ui.ctx().content_rect();
            let painter = ui.painter();
            painter.rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Modal window
    let title = modal_title(kind);
    let mut open = true;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(500.0)
        .open(&mut open)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| match kind {
            ModalKind::Settings => render_settings_modal(ui),
            ModalKind::AgentTuning => render_agent_tuning_modal(ui),
            ModalKind::ArtifactLineage => render_artifact_lineage_modal(ui),
        });

    // Close if the user clicked the X button
    if !open {
        debug!("modal_system — closing {:?}", kind);
        hud.show_modal = None;
    }
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Returns the window title for a modal kind.
fn modal_title(kind: ModalKind) -> &'static str {
    match kind {
        ModalKind::Settings => "Settings",
        ModalKind::AgentTuning => "Agent Tuning",
        ModalKind::ArtifactLineage => "Artifact Lineage",
    }
}

/// Renders the Settings modal content.
fn render_settings_modal(ui: &mut egui::Ui) {
    ui.label("Application settings");
    ui.separator();
    ui.label(
        egui::RichText::new("Configuration options will appear here.")
            .color(egui::Color32::from_rgb(140, 145, 150))
            .italics(),
    );
}

/// Renders the Agent Tuning modal content.
fn render_agent_tuning_modal(ui: &mut egui::Ui) {
    ui.label("Agent parameter tuning");
    ui.separator();
    ui.label(
        egui::RichText::new("Temperature, top-p, and other knobs will appear here.")
            .color(egui::Color32::from_rgb(140, 145, 150))
            .italics(),
    );
}

/// Renders the Artifact Lineage modal content.
fn render_artifact_lineage_modal(ui: &mut egui::Ui) {
    ui.label("Artifact lineage detail");
    ui.separator();
    ui.label(
        egui::RichText::new("Detailed lineage DAG for the selected artifact.")
            .color(egui::Color32::from_rgb(140, 145, 150))
            .italics(),
    );
}
