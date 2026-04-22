//! Atmospheric effects configuration.
//!
//! Configures fog, clouds, and atmospheric scattering.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Atmospheric configuration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereConfig {
    /// Enable atmospheric fog.
    pub fog_enabled: bool,
    /// Fog mode.
    pub fog_mode: FogMode,
    /// Fog color.
    pub fog_color: (f32, f32, f32),
    /// Fog density (for exponential modes).
    pub fog_density: f32,
    /// Fog start distance (for linear mode).
    pub fog_start: f32,
    /// Fog end distance (for linear mode).
    pub fog_end: f32,
    /// Fog height falloff (for height-based fog).
    pub fog_height_falloff: f32,

    /// Enable volumetric fog.
    pub volumetric_fog_enabled: bool,
    /// Volumetric fog intensity.
    pub volumetric_intensity: f32,

    /// Enable sky rendering.
    pub sky_enabled: bool,
    /// Sky color (zenith).
    pub sky_zenith_color: (f32, f32, f32),
    /// Sky color (horizon).
    pub sky_horizon_color: (f32, f32, f32),
    /// Sky ground color.
    pub sky_ground_color: (f32, f32, f32),

    /// Enable cloud rendering.
    pub clouds_enabled: bool,
    /// Cloud coverage (0.0 - 1.0).
    pub cloud_coverage: f32,
    /// Cloud height in meters.
    pub cloud_height: f32,
    /// Cloud scale.
    pub cloud_scale: f32,
    /// Cloud wind speed.
    pub cloud_wind_speed: f32,

    /// Enable atmospheric scattering.
    pub scattering_enabled: bool,
    /// Rayleigh scattering coefficient.
    pub rayleigh_scattering: (f32, f32, f32),
    /// Mie scattering coefficient.
    pub mie_scattering: f32,
    /// Mie scattering direction (anisotropy).
    pub mie_direction: f32,
}

/// Fog modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FogMode {
    /// No fog.
    None,
    /// Linear fog (distance-based).
    Linear,
    /// Exponential fog.
    Exponential,
    /// Exponential squared fog (denser at distance).
    ExponentialSquared,
    /// Height-based atmospheric fog.
    Atmospheric,
}

impl Default for FogMode {
    fn default() -> Self {
        Self::Exponential
    }
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            // Fog
            fog_enabled: true,
            fog_mode: FogMode::Exponential,
            fog_color: (0.75, 0.8, 0.9),
            fog_density: 0.002,
            fog_start: 50.0,
            fog_end: 500.0,
            fog_height_falloff: 0.1,

            // Volumetric fog
            volumetric_fog_enabled: false, // Expensive
            volumetric_intensity: 1.0,

            // Sky
            sky_enabled: true,
            sky_zenith_color: (0.2, 0.4, 0.8),
            sky_horizon_color: (0.6, 0.7, 0.85),
            sky_ground_color: (0.3, 0.25, 0.2),

            // Clouds
            clouds_enabled: true,
            cloud_coverage: 0.4,
            cloud_height: 2000.0,
            cloud_scale: 1.0,
            cloud_wind_speed: 5.0,

            // Scattering
            scattering_enabled: true,
            rayleigh_scattering: (0.0025, 0.005, 0.01),
            mie_scattering: 0.003,
            mie_direction: 0.8,
        }
    }
}

impl AtmosphereConfig {
    /// Create clear sky preset.
    pub fn clear_sky() -> Self {
        Self {
            fog_density: 0.001,
            cloud_coverage: 0.1,
            sky_zenith_color: (0.15, 0.35, 0.9),
            ..default()
        }
    }

    /// Create partly cloudy preset.
    pub fn partly_cloudy() -> Self {
        Self {
            cloud_coverage: 0.5,
            fog_density: 0.002,
            sky_zenith_color: (0.3, 0.5, 0.8),
            ..default()
        }
    }

    /// Create overcast preset.
    pub fn overcast() -> Self {
        Self {
            fog_density: 0.005,
            cloud_coverage: 0.9,
            sky_zenith_color: (0.5, 0.55, 0.6),
            sky_horizon_color: (0.6, 0.62, 0.65),
            ..default()
        }
    }

    /// Create misty morning preset.
    pub fn misty_morning() -> Self {
        Self {
            fog_mode: FogMode::Atmospheric,
            fog_density: 0.008,
            fog_height_falloff: 0.2,
            cloud_coverage: 0.2,
            sky_zenith_color: (0.4, 0.5, 0.7),
            sky_horizon_color: (0.8, 0.7, 0.6),
            ..default()
        }
    }

    /// Create foggy evening preset.
    pub fn foggy_evening() -> Self {
        Self {
            fog_mode: FogMode::Exponential,
            fog_density: 0.01,
            fog_color: (0.6, 0.5, 0.4),
            cloud_coverage: 0.3,
            sky_zenith_color: (0.3, 0.3, 0.5),
            sky_horizon_color: (0.7, 0.5, 0.4),
            ..default()
        }
    }
}

/// Bevy fog configuration helper.
/// Note: Fog support may vary by Bevy version.
pub fn apply_fog_config(config: &AtmosphereConfig) -> Option<(Color, f32)> {
    if !config.fog_enabled {
        return None;
    }

    let color = Color::srgb(config.fog_color.0, config.fog_color.1, config.fog_color.2);

    // Return color and density for custom fog implementation
    Some((color, config.fog_density))
}
