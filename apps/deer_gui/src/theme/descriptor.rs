//! Theme descriptor — data-driven theme definition.
//!
//! Deserialized from RON/JSON files. Converts to the runtime [`Theme`]
//! struct via [`ThemeDescriptor::into_theme`].

use bevy::color::{Color, LinearRgba};
use bevy::log::{debug, trace};
use serde::Deserialize;

use super::theme::Theme;

// ---------------------------------------------------------------------------
// ThemeDescriptor
// ---------------------------------------------------------------------------

/// Theme descriptor, deserialized from a RON/JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeDescriptor {
    pub name: String,
    pub background: [f32; 4],
    pub surface: [f32; 4],
    pub accent: [f32; 4],
    pub accent_secondary: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub success: [f32; 4],
    pub warning: [f32; 4],
    pub error: [f32; 4],
    pub panel_alpha: f32,
    pub panel_rounding: f32,
    pub star_emissive: [f32; 4],
    pub monolith_emissive: [f32; 4],
    pub trail_emissive: [f32; 4],
    pub trail_base_color: [f32; 4],
    pub monolith_glow_channels: [f32; 3],
    pub font_css_url: Option<String>,
}

impl ThemeDescriptor {
    /// Convert to the runtime [`Theme`] struct.
    pub fn into_theme(self) -> Theme {
        debug!("ThemeDescriptor::into_theme — name='{}'", self.name);
        Theme {
            name: self.name,
            background: arr_to_color(self.background),
            surface: arr_to_color(self.surface),
            accent: arr_to_color(self.accent),
            accent_secondary: arr_to_color(self.accent_secondary),
            text_primary: arr_to_color(self.text_primary),
            text_secondary: arr_to_color(self.text_secondary),
            success: arr_to_color(self.success),
            warning: arr_to_color(self.warning),
            error: arr_to_color(self.error),
            panel_alpha: self.panel_alpha,
            panel_rounding: self.panel_rounding,
            star_emissive: arr_to_linear(self.star_emissive),
            monolith_emissive: arr_to_linear(self.monolith_emissive),
            trail_emissive: arr_to_linear(self.trail_emissive),
            trail_base_color: arr_to_color(self.trail_base_color),
            monolith_glow_channels: self.monolith_glow_channels,
        }
    }

    /// Log descriptor contents for debugging.
    pub fn log_info(&self) {
        trace!(
            "ThemeDescriptor: name='{}' font_css={:?}",
            self.name,
            self.font_css_url,
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn arr_to_color(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}

fn arr_to_linear(c: [f32; 4]) -> LinearRgba {
    LinearRgba::new(c[0], c[1], c[2], c[3])
}
