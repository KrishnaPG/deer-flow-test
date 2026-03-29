//! Precursors theme — warm amber and earth tones.
//!
//! A warm, organic palette inspired by ancient civilizations and riverside
//! cultures. Deep amber backgrounds, golden accents, and natural greens.

use bevy::log::{debug, trace};
use bevy_egui::egui;

use super::theme::Theme;
use crate::constants::visual::{HUD_PANEL_ALPHA, HUD_PANEL_ROUNDING};

/// Returns the Precursors theme.
pub fn precursors_theme() -> Theme {
    debug!("precursors_theme — constructing Precursors theme");
    Theme {
        name: "Precursors".to_string(),
        background: bevy::color::Color::srgba(0.22, 0.16, 0.10, 1.0),
        surface: bevy::color::Color::srgba(0.30, 0.25, 0.18, 1.0),
        accent: bevy::color::Color::srgba(0.85, 0.60, 0.20, 1.0),
        accent_secondary: bevy::color::Color::srgba(0.35, 0.55, 0.30, 1.0),
        text_primary: bevy::color::Color::srgba(0.95, 0.90, 0.80, 1.0),
        text_secondary: bevy::color::Color::srgba(0.70, 0.65, 0.55, 1.0),
        success: bevy::color::Color::srgba(0.3, 0.7, 0.3, 1.0),
        warning: bevy::color::Color::srgba(0.9, 0.7, 0.1, 1.0),
        error: bevy::color::Color::srgba(0.8, 0.2, 0.2, 1.0),
        panel_alpha: HUD_PANEL_ALPHA,
        panel_rounding: HUD_PANEL_ROUNDING,
        star_emissive: bevy::color::LinearRgba::new(1.8, 1.4, 0.8, 1.0),
        monolith_emissive: bevy::color::LinearRgba::new(0.8, 0.6, 0.3, 1.0),
        trail_emissive: bevy::color::LinearRgba::new(0.6, 0.8, 0.4, 1.0),
        trail_base_color: bevy::color::Color::srgb(0.5, 0.7, 0.3),
        monolith_glow_channels: [0.8, 0.6, 0.3],
    }
}

/// Converts a Bevy [`Color`] to an [`egui::Color32`].
fn bevy_to_egui_color(color: bevy::color::Color) -> egui::Color32 {
    let srgba = color.to_srgba();
    egui::Color32::from_rgba_unmultiplied(
        (srgba.red.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.green.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.blue.clamp(0.0, 1.0) * 255.0) as u8,
        (srgba.alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Builds an [`egui::Visuals`] configured from the given [`Theme`].
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

    visuals.selection = egui::style::Selection {
        bg_fill: accent.gamma_multiply(0.25),
        stroke: egui::Stroke::new(1.0, accent),
    };

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
