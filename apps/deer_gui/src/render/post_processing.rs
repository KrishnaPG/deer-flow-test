//! Post-processing effects configuration.
//!
//! Configures bloom, tone mapping, and other screen-space effects.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Post-processing configuration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    /// Enable bloom effect.
    pub bloom_enabled: bool,
    /// Bloom settings.
    pub bloom: BloomSettings,

    /// Tone mapping mode.
    pub tone_mapping: ToneMapping,

    /// Color grading settings.
    pub color_grading: ColorGradingConfig,

    /// Enable vignette effect.
    pub vignette_enabled: bool,
    /// Vignette intensity (0.0 - 1.0).
    pub vignette_intensity: f32,

    /// Enable chromatic aberration.
    pub chromatic_aberration: bool,
    /// Chromatic aberration intensity.
    pub chromatic_aberration_intensity: f32,

    /// Enable film grain.
    pub film_grain_enabled: bool,
    /// Film grain intensity.
    pub film_grain_intensity: f32,

    /// Enable lens distortion.
    pub lens_distortion: f32,
}

/// Bloom effect settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomSettings {
    /// Bloom intensity (0.0 - 2.0).
    pub intensity: f32,
    /// Threshold for bloom (brightness that triggers glow).
    pub threshold: f32,
    /// Soft threshold knee.
    pub knee: f32,
    /// Number of blur passes (more = smoother but slower).
    pub num_passes: u32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            intensity: 0.8,
            threshold: 0.9,
            knee: 0.3,
            num_passes: 5,
        }
    }
}

/// Tone mapping modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToneMapping {
    /// No tone mapping.
    None,
    /// Reinhard tone mapping.
    Reinhard,
    /// ACES filmic tone mapping (cinematic).
    Aces,
    /// AgX (modern PBR tone mapping).
    AgX,
}

impl Default for ToneMapping {
    fn default() -> Self {
        Self::Aces
    }
}

/// Color grading configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingConfig {
    /// Exposure compensation (-2.0 to 2.0).
    pub exposure: f32,
    /// Gamma correction (0.5 to 2.0).
    pub gamma: f32,
    /// Contrast adjustment (0.5 to 2.0).
    pub contrast: f32,
    /// Saturation (0.0 to 2.0).
    pub saturation: f32,
    /// Temperature shift (-1.0 to 1.0).
    pub temperature: f32,
    /// Tint shift (-1.0 to 1.0).
    pub tint: f32,
}

impl Default for ColorGradingConfig {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            gamma: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            temperature: 0.0,
            tint: 0.0,
        }
    }
}

impl Default for PostProcessingConfig {
    fn default() -> Self {
        Self {
            bloom_enabled: true,
            bloom: BloomSettings::default(),
            tone_mapping: ToneMapping::Aces,
            color_grading: ColorGradingConfig::default(),
            vignette_enabled: true,
            vignette_intensity: 0.3,
            chromatic_aberration: false,
            chromatic_aberration_intensity: 0.0,
            film_grain_enabled: false,
            film_grain_intensity: 0.0,
            lens_distortion: 0.0,
        }
    }
}

impl PostProcessingConfig {
    /// Create cinematic post-processing preset.
    pub fn cinematic() -> Self {
        Self {
            bloom_enabled: true,
            bloom: BloomSettings {
                intensity: 1.0,
                threshold: 0.85,
                knee: 0.25,
                num_passes: 6,
            },
            tone_mapping: ToneMapping::Aces,
            color_grading: ColorGradingConfig {
                exposure: 0.1,
                contrast: 1.1,
                saturation: 1.1,
                ..default()
            },
            vignette_enabled: true,
            vignette_intensity: 0.4,
            film_grain_enabled: false,
            ..default()
        }
    }

    /// Create film look preset.
    pub fn film_look() -> Self {
        Self {
            bloom: BloomSettings {
                intensity: 0.6,
                threshold: 0.95,
                knee: 0.5,
                num_passes: 4,
            },
            color_grading: ColorGradingConfig {
                contrast: 1.2,
                saturation: 0.9,
                temperature: 0.1, // Slightly warm
                ..default()
            },
            film_grain_enabled: true,
            film_grain_intensity: 0.05,
            vignette_intensity: 0.5,
            ..default()
        }
    }

    /// Create vibrant game preset.
    pub fn vibrant() -> Self {
        Self {
            color_grading: ColorGradingConfig {
                contrast: 1.05,
                saturation: 1.3,
                ..default()
            },
            ..default()
        }
    }
}
