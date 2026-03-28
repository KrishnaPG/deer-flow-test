//! HUD style helpers — reusable frame builders and colour utilities.
//!
//! All HUD panels use these helpers to achieve the consistent
//! "glass panel" aesthetic described in the design wireframes.

use bevy::log::trace;
use bevy_egui::egui;

use crate::constants::visual::{HUD_PANEL_ALPHA, HUD_PANEL_ROUNDING};
use crate::world::components::AgentState;

use super::{EventSeverity, MissionStatus};

// ---------------------------------------------------------------------------
// Frame builders
// ---------------------------------------------------------------------------

/// Returns an [`egui::Frame`] for a semi-transparent glass panel.
///
/// Uses the theme-configured alpha and rounding from constants.
pub fn glass_panel_frame(ctx: &egui::Context) -> egui::Frame {
    trace!("glass_panel_frame — building frame");
    let visuals = ctx.style().visuals.clone();
    let bg = visuals.panel_fill;
    let fill = bg.gamma_multiply(HUD_PANEL_ALPHA);
    egui::Frame::new()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(HUD_PANEL_ROUNDING as u8))
        .stroke(egui::Stroke::new(1.0, visuals.window_stroke.color))
        .inner_margin(egui::Margin::same(8))
}

/// Returns an [`egui::Frame`] for the top bar (thinner padding).
pub fn top_bar_frame(ctx: &egui::Context) -> egui::Frame {
    trace!("top_bar_frame — building frame");
    let visuals = ctx.style().visuals.clone();
    let bg = visuals.panel_fill;
    let fill = bg.gamma_multiply(HUD_PANEL_ALPHA);
    egui::Frame::new()
        .fill(fill)
        .corner_radius(egui::CornerRadius::ZERO)
        .stroke(egui::Stroke::new(
            1.0,
            visuals.window_stroke.color.gamma_multiply(0.5),
        ))
        .inner_margin(egui::Margin::symmetric(12, 6))
}

/// Returns an [`egui::Frame`] for side panels (left/right).
pub fn side_panel_frame(ctx: &egui::Context) -> egui::Frame {
    trace!("side_panel_frame — building frame");
    let visuals = ctx.style().visuals.clone();
    let bg = visuals.panel_fill;
    let fill = bg.gamma_multiply(HUD_PANEL_ALPHA);
    egui::Frame::new()
        .fill(fill)
        .corner_radius(egui::CornerRadius::ZERO)
        .stroke(egui::Stroke::new(
            1.0,
            visuals.window_stroke.color.gamma_multiply(0.5),
        ))
        .inner_margin(egui::Margin::same(8))
}

// ---------------------------------------------------------------------------
// Colour helpers
// ---------------------------------------------------------------------------

/// Returns the status colour for an agent state.
pub fn agent_state_color(state: &AgentState) -> egui::Color32 {
    match state {
        AgentState::Idle => egui::Color32::from_rgb(128, 128, 128), // grey
        AgentState::Working => egui::Color32::from_rgb(0, 204, 255), // cyan
        AgentState::Error => egui::Color32::from_rgb(255, 77, 77),  // red
        AgentState::ApprovalNeeded => egui::Color32::from_rgb(255, 191, 0), // amber
    }
}

/// Returns the status colour for a mission status.
pub fn mission_status_color(status: &MissionStatus) -> egui::Color32 {
    match status {
        MissionStatus::Active => egui::Color32::from_rgb(51, 230, 102), // green
        MissionStatus::Blocked => egui::Color32::from_rgb(255, 77, 77), // red
        MissionStatus::Idle => egui::Color32::from_rgb(128, 128, 128),  // grey
        MissionStatus::Completed => egui::Color32::from_rgb(0, 204, 255), // cyan
    }
}

/// Returns the colour for an event severity level.
pub fn event_severity_color(severity: &EventSeverity) -> egui::Color32 {
    match severity {
        EventSeverity::Info => egui::Color32::from_rgb(180, 190, 200), // light grey
        EventSeverity::Warning => egui::Color32::from_rgb(255, 191, 51), // amber
        EventSeverity::Error => egui::Color32::from_rgb(255, 77, 77),  // red
    }
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

/// Draws a horizontal progress bar in an egui layout.
///
/// `progress` is clamped to 0.0–1.0. The bar uses the given `color`
/// for the filled portion and a dim background for the track.
pub fn draw_progress_bar(ui: &mut egui::Ui, progress: f32, color: egui::Color32) {
    let clamped = progress.clamp(0.0, 1.0);
    let available_width = ui.available_width().min(200.0);
    let height = 8.0;

    let (rect, _response) =
        ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::hover());

    let painter = ui.painter_at(rect);

    // Track background
    painter.rect_filled(
        rect,
        egui::CornerRadius::same(3),
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20),
    );

    // Filled portion
    if clamped > 0.0 {
        let filled_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.min.x + rect.width() * clamped, rect.max.y),
        );
        painter.rect_filled(filled_rect, egui::CornerRadius::same(3), color);
    }
}
