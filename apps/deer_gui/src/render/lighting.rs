//! Cinematic lighting configuration.
//!
//! Provides realistic lighting setup for medieval scenes.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Cinematic lighting configuration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CinematicLighting {
    /// Sun light color (daytime).
    pub sun_color: (f32, f32, f32),
    /// Sun light intensity in lux.
    pub sun_intensity: f32,
    /// Sun light temperature in Kelvin.
    pub sun_temperature: f32,

    /// Moon light color (nighttime).
    pub moon_color: (f32, f32, f32),
    /// Moon light intensity in lux.
    pub moon_intensity: f32,

    /// Ambient light color.
    pub ambient_color: (f32, f32, f32),
    /// Ambient light intensity.
    pub ambient_intensity: f32,

    /// Enable automatic sun position based on time.
    pub auto_sun_position: bool,
    /// Day duration in seconds (for auto sun).
    pub day_duration: f32,

    /// Enable rim lighting on characters.
    pub rim_lighting_enabled: bool,
    /// Rim light color.
    pub rim_color: (f32, f32, f32),
    /// Rim light intensity.
    pub rim_intensity: f32,

    /// Enable subsurface scattering on vegetation.
    pub subsurface_scattering: bool,
}

impl Default for CinematicLighting {
    fn default() -> Self {
        Self {
            // Sun (warm golden hour style)
            sun_color: (1.0, 0.95, 0.85),
            sun_intensity: 100_000.0,
            sun_temperature: 5500.0,

            // Moon (cool blue)
            moon_color: (0.7, 0.75, 0.9),
            moon_intensity: 1_000.0,

            // Ambient (slightly blue for sky fill)
            ambient_color: (0.5, 0.55, 0.65),
            ambient_intensity: 0.05,

            // Auto sun
            auto_sun_position: true,
            day_duration: 600.0, // 10 minute day cycle

            // Rim lighting (backlight for characters)
            rim_lighting_enabled: true,
            rim_color: (1.0, 0.9, 0.7),
            rim_intensity: 0.5,

            // Subsurface scattering for vegetation
            subsurface_scattering: true,
        }
    }
}

impl CinematicLighting {
    /// Create morning lighting preset.
    pub fn morning() -> Self {
        Self {
            sun_color: (1.0, 0.9, 0.7),
            sun_intensity: 60_000.0,
            sun_temperature: 4000.0,
            ambient_intensity: 0.03,
            ..default()
        }
    }

    /// Create noon lighting preset.
    pub fn noon() -> Self {
        Self {
            sun_color: (1.0, 0.98, 0.95),
            sun_intensity: 120_000.0,
            sun_temperature: 6500.0,
            ambient_intensity: 0.08,
            ..default()
        }
    }

    /// Create sunset lighting preset.
    pub fn sunset() -> Self {
        Self {
            sun_color: (1.0, 0.6, 0.3),
            sun_intensity: 40_000.0,
            sun_temperature: 3000.0,
            ambient_color: (0.4, 0.3, 0.5),
            ambient_intensity: 0.02,
            ..default()
        }
    }

    /// Create night lighting preset.
    pub fn night() -> Self {
        Self {
            sun_color: (0.1, 0.1, 0.15),
            sun_intensity: 100.0,
            sun_temperature: 1000.0,
            ambient_color: (0.1, 0.1, 0.2),
            ambient_intensity: 0.005,
            rim_intensity: 0.2,
            ..default()
        }
    }

    /// Create overcast lighting preset.
    pub fn overcast() -> Self {
        Self {
            sun_color: (0.8, 0.82, 0.85),
            sun_intensity: 30_000.0,
            sun_temperature: 6000.0,
            ambient_color: (0.6, 0.62, 0.65),
            ambient_intensity: 0.1,
            ..default()
        }
    }
}
