//! Top bar HUD panel — agent counts, token burn rate, system health.
//!
//! Renders as a thin horizontal strip across the top of the screen
//! showing real-time operational metrics.

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::styles::top_bar_frame;
use super::HudState;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the top bar panel with agent metrics and system status.
pub fn top_bar_system(mut contexts: EguiContexts, hud: Res<HudState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!("top_bar_system — rendering");

    let frame = top_bar_frame(ctx);

    egui::TopBottomPanel::top("hud_top_bar")
        .frame(frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                render_agent_counts(ui, &hud);
                ui.separator();
                render_token_metrics(ui, &hud);
                ui.separator();
                render_system_status(ui, &hud);
            });
        });
}

// ---------------------------------------------------------------------------
// Render helpers (kept under 50 LOC each)
// ---------------------------------------------------------------------------

/// Renders agent count indicators: active / retrying / failed.
fn render_agent_counts(ui: &mut egui::Ui, hud: &HudState) {
    let active_color = egui::Color32::from_rgb(51, 230, 102); // green
    let retry_color = egui::Color32::from_rgb(255, 191, 51); // amber
    let fail_color = egui::Color32::from_rgb(255, 77, 77); // red

    ui.colored_label(active_color, format!("Active: {}", hud.active_agents));
    ui.colored_label(retry_color, format!("Retry: {}", hud.retrying_agents));
    ui.colored_label(fail_color, format!("Fail: {}", hud.failed_agents));
}

/// Renders token throughput and cost metrics.
fn render_token_metrics(ui: &mut egui::Ui, hud: &HudState) {
    ui.label(format!("{:.0} tok/s", hud.tokens_per_sec));
    ui.label(format!("${:.2}/hr", hud.cost_per_hour));
}

/// Renders system online/offline status indicator.
fn render_system_status(ui: &mut egui::Ui, hud: &HudState) {
    let (color, label) = if hud.system_online {
        (egui::Color32::from_rgb(51, 230, 102), "ONLINE")
    } else {
        (egui::Color32::from_rgb(255, 77, 77), "OFFLINE")
    };

    let dot = egui::RichText::new("\u{25CF}").color(color).size(12.0);
    ui.label(dot);
    ui.colored_label(color, label);
}
