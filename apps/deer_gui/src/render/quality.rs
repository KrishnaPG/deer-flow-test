//! Quality settings — configurable rendering presets.
//!
//! Provides presets for different quality levels from low to cinematic.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Quality preset levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityPreset {
    /// Low quality for older hardware.
    Low,
    /// Medium quality for balanced performance.
    Medium,
    /// High quality for modern gaming PCs.
    High,
    /// Ultra quality for high-end systems.
    Ultra,
    /// Cinematic quality for screenshots and demos.
    Cinematic,
}

impl Default for QualityPreset {
    fn default() -> Self {
        Self::High
    }
}

/// Comprehensive quality settings for rendering.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    // =========================================================================
    // Shadow Settings
    // =========================================================================
    /// Enable shadow mapping.
    pub shadows_enabled: bool,
    /// Number of shadow cascades (1-4).
    pub shadow_cascades: u32,
    /// First cascade far bound in world units.
    pub shadow_first_cascade_bound: f32,
    /// Maximum shadow distance.
    pub shadow_maximum_distance: f32,
    /// Shadow map resolution per cascade.
    pub shadow_map_resolution: u32,

    // =========================================================================
    // Ambient Occlusion
    // =========================================================================
    /// Enable Screen-Space Ambient Occlusion.
    pub ssao_enabled: bool,
    /// SSAO quality level (0-4).
    pub ssao_quality: u32,
    /// SSAO sampling radius.
    pub ssao_radius: f32,
    /// SSAO intensity multiplier.
    pub ssao_intensity: f32,

    // =========================================================================
    // Bloom / Post-Processing
    // =========================================================================
    /// Enable bloom effect.
    pub bloom_enabled: bool,
    /// Bloom intensity (0.0 - 2.0).
    pub bloom_intensity: f32,
    /// Bloom threshold (brightness that triggers bloom).
    pub bloom_threshold: f32,
    /// Bloom knee (soft threshold transition).
    pub bloom_knee: f32,

    // =========================================================================
    // Fog / Atmosphere
    // =========================================================================
    /// Enable atmospheric fog.
    pub fog_enabled: bool,
    /// Fog color.
    pub fog_color: (f32, f32, f32),
    /// Fog density.
    pub fog_density: f32,
    /// Fog start distance.
    pub fog_start: f32,
    /// Fog end distance.
    pub fog_end: f32,

    // =========================================================================
    // Lighting
    // =========================================================================
    /// Ambient light brightness.
    pub ambient_brightness: f32,
    /// Enable light scattering (god rays).
    pub light_scattering_enabled: bool,
    /// Number of dynamic lights supported.
    pub max_dynamic_lights: u32,

    // =========================================================================
    // Materials / Textures
    // =========================================================================
    /// Enable normal maps.
    pub normal_maps_enabled: bool,
    /// Enable parallax occlusion mapping.
    pub parallax_mapping_enabled: bool,
    /// Texture quality (0 = low, 1 = medium, 2 = high, 3 = ultra).
    pub texture_quality: u32,
    /// Anisotropic filtering level (1, 2, 4, 8, 16).
    pub anisotropic_filtering: u32,

    // =========================================================================
    // Terrain / Vegetation
    // =========================================================================
    /// Terrain LOD distance multiplier.
    pub terrain_lod_multiplier: f32,
    /// Vegetation draw distance in meters.
    pub vegetation_draw_distance: f32,
    /// Vegetation density multiplier (0.0 - 2.0).
    pub vegetation_density: f32,

    // =========================================================================
    // Water
    // =========================================================================
    /// Enable screen-space reflections for water.
    pub water_ssr_enabled: bool,
    /// Water quality (affects tessellation and flow maps).
    pub water_quality: u32,

    // =========================================================================
    // Anti-Aliasing
    // =========================================================================
    /// Anti-aliasing mode.
    pub antialiasing: AntiAliasing,
}

/// Anti-aliasing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AntiAliasing {
    /// No anti-aliasing.
    None,
    /// FXAA (fast approximate).
    Fx,
    /// MSAA 4x (multi-sample).
    Msaa4x,
    /// MSAA 8x (multi-sample).
    Msaa8x,
    /// TAA (temporal anti-aliasing).
    Taa,
}

impl Default for AntiAliasing {
    fn default() -> Self {
        Self::Taa
    }
}

impl QualitySettings {
    /// Create cinematic quality preset (maximum visuals).
    ///
    /// Target: RTX 4080 / RX 7900 XT or better
    pub fn cinematic() -> Self {
        Self {
            // Shadows
            shadows_enabled: true,
            shadow_cascades: 4,
            shadow_first_cascade_bound: 10.0,
            shadow_maximum_distance: 200.0,
            shadow_map_resolution: 2048,

            // SSAO
            ssao_enabled: true,
            ssao_quality: 4,
            ssao_radius: 0.5,
            ssao_intensity: 1.0,

            // Bloom
            bloom_enabled: true,
            bloom_intensity: 0.8,
            bloom_threshold: 0.9,
            bloom_knee: 0.3,

            // Fog
            fog_enabled: true,
            fog_color: (0.7, 0.75, 0.85),
            fog_density: 0.002,
            fog_start: 50.0,
            fog_end: 500.0,

            // Lighting
            ambient_brightness: 0.05,
            light_scattering_enabled: true,
            max_dynamic_lights: 64,

            // Materials
            normal_maps_enabled: true,
            parallax_mapping_enabled: true,
            texture_quality: 3,
            anisotropic_filtering: 16,

            // Terrain/Vegetation
            terrain_lod_multiplier: 1.0,
            vegetation_draw_distance: 500.0,
            vegetation_density: 1.5,

            // Water
            water_ssr_enabled: true,
            water_quality: 3,

            // Anti-aliasing
            antialiasing: AntiAliasing::Taa,
        }
    }

    /// Create ultra quality preset.
    ///
    /// Target: RTX 3080 / RX 6800 XT or better
    pub fn ultra() -> Self {
        Self {
            shadow_cascades: 4,
            shadow_map_resolution: 1024,
            shadow_maximum_distance: 150.0,
            vegetation_draw_distance: 400.0,
            vegetation_density: 1.2,
            ..Self::cinematic()
        }
    }

    /// Create high quality preset.
    ///
    /// Target: RTX 3060 / RX 6700 XT
    pub fn high() -> Self {
        Self {
            shadow_cascades: 3,
            shadow_map_resolution: 512,
            shadow_maximum_distance: 100.0,
            ssao_quality: 2,
            vegetation_draw_distance: 300.0,
            vegetation_density: 1.0,
            anisotropic_filtering: 8,
            water_ssr_enabled: false,
            parallax_mapping_enabled: false,
            ..Self::ultra()
        }
    }

    /// Create medium quality preset.
    ///
    /// Target: GTX 1660 / RX 580
    pub fn medium() -> Self {
        Self {
            shadow_cascades: 2,
            shadow_map_resolution: 256,
            shadow_maximum_distance: 75.0,
            ssao_enabled: false,
            vegetation_draw_distance: 200.0,
            vegetation_density: 0.7,
            anisotropic_filtering: 4,
            bloom_enabled: false,
            light_scattering_enabled: false,
            ..Self::high()
        }
    }

    /// Create low quality preset.
    ///
    /// Target: Integrated graphics / older GPUs
    pub fn low() -> Self {
        Self {
            shadows_enabled: false,
            shadow_cascades: 1,
            ssao_enabled: false,
            bloom_enabled: false,
            fog_enabled: false,
            normal_maps_enabled: false,
            vegetation_draw_distance: 100.0,
            vegetation_density: 0.4,
            anisotropic_filtering: 1,
            texture_quality: 0,
            antialiasing: AntiAliasing::Fx,
            ..Self::medium()
        }
    }

    /// Get preset by enum.
    pub fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Low => Self::low(),
            QualityPreset::Medium => Self::medium(),
            QualityPreset::High => Self::high(),
            QualityPreset::Ultra => Self::ultra(),
            QualityPreset::Cinematic => Self::cinematic(),
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self::high()
    }
}
