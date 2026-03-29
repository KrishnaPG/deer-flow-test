//! Event ticker HUD overlay — scrolling event log with auto-fade.
//!
//! Renders recent event log entries in a floating overlay above
//! the bottom console. Entries fade out based on their age.
//!
//! **Render-only**: mutation (aging, pruning, capping) lives in
//! [`super::state_systems::event_ticker_maintenance_system`].

use bevy::log::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::constants::timing::EVENT_TICKER_FADE_SECS;

use super::styles::event_severity_color;
use super::HudState;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the event ticker overlay (read-only access to [`HudState`]).
///
/// Filters visible entries (age < fade threshold) and draws them
/// in a floating overlay anchored to the bottom-right.
pub fn event_ticker_system(mut contexts: EguiContexts, hud: Res<HudState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Only render if there are visible entries
    let visible: Vec<_> = hud
        .event_log
        .iter()
        .filter(|e| e.age_secs < EVENT_TICKER_FADE_SECS)
        .collect();

    if visible.is_empty() {
        return;
    }

    trace!("event_ticker_system — rendering {} entries", visible.len());

    egui::Area::new(egui::Id::new("hud_event_ticker"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -64.0))
        .show(ctx, |ui| {
            for entry in visible.iter().rev().take(10) {
                let alpha = compute_alpha(entry.age_secs);
                let severity_color = event_severity_color(&entry.severity).gamma_multiply(alpha);

                ui.horizontal(|ui| {
                    let ts = egui::RichText::new(&entry.timestamp).size(9.0).color(
                        egui::Color32::from_rgba_unmultiplied(140, 145, 150, (alpha * 255.0) as u8),
                    );
                    ui.label(ts);

                    let msg = egui::RichText::new(&entry.message)
                        .size(11.0)
                        .color(severity_color);
                    ui.label(msg);
                });
            }
        });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Computes the opacity (0.0–1.0) for an entry based on its age.
///
/// Entries are fully opaque for the first 60% of the fade window,
/// then linearly fade to zero.
fn compute_alpha(age_secs: f32) -> f32 {
    let fade_start = EVENT_TICKER_FADE_SECS * 0.6;
    if age_secs <= fade_start {
        1.0
    } else {
        let remaining = EVENT_TICKER_FADE_SECS - age_secs;
        let fade_window = EVENT_TICKER_FADE_SECS - fade_start;
        (remaining / fade_window).clamp(0.0, 1.0)
    }
}
