//! TET Orchestrator theme — deep blues, blacks, and cyan accents.
//!
//! This is the signature dark theme for the Deer GUI control center,
//! inspired by futuristic command-and-control aesthetics.

use bevy::log::{debug, trace};
use bevy_egui::egui;

use super::theme::Theme;
use crate::constants::visual::{HUD_PANEL_ALPHA, HUD_PANEL_ROUNDING};

// ---------------------------------------------------------------------------
// Theme factory
// ---------------------------------------------------------------------------

/// Returns the TET Orchestrator theme.
pub fn tet_theme() -> Theme {
    debug!("tet_theme — constructing TET Orchestrator theme");
    Theme {
        name: "TET Orchestrator".to_string(),
        background: bevy::color::Color::srgba(0.02, 0.02, 0.05, 1.0), // near-black
        surface: bevy::color::Color::srgba(0.05, 0.05, 0.12, 1.0),    // dark navy
        accent: bevy::color::Color::srgba(0.0, 0.8, 1.0, 1.0),        // cyan
        accent_secondary: bevy::color::Color::srgba(0.3, 0.5, 1.0, 1.0), // soft blue
        text_primary: bevy::color::Color::srgba(0.9, 0.92, 0.95, 1.0), // off-white
        text_secondary: bevy::color::Color::srgba(0.5, 0.55, 0.6, 1.0), // grey
        success: bevy::color::Color::srgba(0.2, 0.9, 0.4, 1.0),       // green
        warning: bevy::color::Color::srgba(1.0, 0.75, 0.2, 1.0),      // amber
        error: bevy::color::Color::srgba(1.0, 0.3, 0.3, 1.0),         // red
        panel_alpha: HUD_PANEL_ALPHA,
        panel_rounding: HUD_PANEL_ROUNDING,
        // World material colours
        star_emissive: bevy::color::LinearRgba::new(2.0, 2.0, 2.0, 1.0),
        monolith_emissive: bevy::color::LinearRgba::new(0.3, 0.5, 1.0, 1.0),
        trail_emissive: bevy::color::LinearRgba::new(0.0, 1.5, 0.8, 1.0),
        trail_base_color: bevy::color::Color::srgb(0.0, 0.8, 0.5),
        monolith_glow_channels: [0.3, 0.5, 1.0],
    }
}

// ---------------------------------------------------------------------------
// Bevy Color -> egui Color32 conversion
// ---------------------------------------------------------------------------

/// Converts a Bevy [`Color`] to an [`egui::Color32`].
///
/// Extracts linear RGBA, converts to sRGB-space bytes, and packs.
fn bevy_to_egui_color(color: bevy::color::Color) -> egui::Color32 {
    let srgba = color.to_srgba();
    egui::Color32::from_rgba_unmultiplied(
        (srgba.red.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.green.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.blue.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

// ---------------------------------------------------------------------------
// Apply theme to egui Visuals
// ---------------------------------------------------------------------------

/// Builds an [`egui::Visuals`] configured from the given [`Theme`].
///
/// Starts from `Visuals::dark()` and overrides the palette colours,
/// panel fills, window fills, accent strokes, and widget states.
pub fn apply_theme_to_egui(theme: &Theme) -> egui::Visuals {
    trace!(
        "apply_theme_to_egui — building visuals for '{}'",
        theme.name
    );

    let bg = bevy_to_egui_color(theme.background);
    let surface = bevy_to_egui_color(theme.surface);
    let accent = bevy_to_egui_color(theme.accent);
    let text = bevy_to_egui_color(theme.text_primary);
    let text_dim = bevy_to_egui_color(theme.text_secondary);
    let warn = bevy_to_egui_color(theme.warning);
    let err = bevy_to_egui_color(theme.error);

    let rounding = egui::CornerRadius::same(theme.panel_rounding as u8);

    let mut visuals = egui::Visuals::dark();

    // Global overrides
    visuals.override_text_color = Some(text);
    visuals.panel_fill = surface;
    visuals.window_fill = bg;
    visuals.window_corner_radius = rounding;
    visuals.window_stroke = egui::Stroke::new(1.0, accent);
    visuals.faint_bg_color = surface;
    visuals.extreme_bg_color = bg;
    visuals.warn_fg_color = warn;
    visuals.error_fg_color = err;
    visuals.hyperlink_color = accent;

    // Selection accent
    visuals.selection = egui::style::Selection {
        bg_fill: accent.gamma_multiply(0.25),
        stroke: egui::Stroke::new(1.0, accent),
    };

    // Widget states
    visuals.widgets.noninteractive = egui::style::WidgetVisuals {
        weak_bg_fill: surface,
        bg_fill: surface,
        bg_stroke: egui::Stroke::new(1.0, text_dim),
        fg_stroke: egui::Stroke::new(1.0, text_dim),
        corner_radius: egui::CornerRadius::same(2),
        expansion: 0.0,
    };

    visuals.widgets.inactive = egui::style::WidgetVisuals {
        weak_bg_fill: bg,
        bg_fill: bg,
        bg_stroke: egui::Stroke::NONE,
        fg_stroke: egui::Stroke::new(1.0, text),
        corner_radius: egui::CornerRadius::same(2),
        expansion: 0.0,
    };

    visuals.widgets.hovered = egui::style::WidgetVisuals {
        weak_bg_fill: surface,
        bg_fill: surface,
        bg_stroke: egui::Stroke::new(1.0, accent),
        fg_stroke: egui::Stroke::new(1.5, accent),
        corner_radius: egui::CornerRadius::same(3),
        expansion: 1.0,
    };

    visuals.widgets.active = egui::style::WidgetVisuals {
        weak_bg_fill: accent.gamma_multiply(0.15),
        bg_fill: accent.gamma_multiply(0.15),
        bg_stroke: egui::Stroke::new(1.0, accent),
        fg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
        corner_radius: egui::CornerRadius::same(2),
        expansion: 1.0,
    };

    visuals.widgets.open = egui::style::WidgetVisuals {
        weak_bg_fill: surface,
        bg_fill: bg,
        bg_stroke: egui::Stroke::new(1.0, text_dim),
        fg_stroke: egui::Stroke::new(1.0, text),
        corner_radius: egui::CornerRadius::same(2),
        expansion: 0.0,
    };

    debug!(
        "apply_theme_to_egui — visuals built for '{}' (panel_fill={:?})",
        theme.name, visuals.panel_fill,
    );

    visuals
}
