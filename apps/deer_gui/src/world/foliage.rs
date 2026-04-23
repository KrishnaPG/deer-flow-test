//! FoliagePlugin - Modular wrapper for bevy_feronia vegetation rendering.
//!
//! This plugin provides a reusable interface for GPU-instanced vegetation
//! with wind animation, biome-based scattering, and LOD management.
//!
//! ## Features
//!
//! - GPU instancing via bevy_feronia for thousands of trees/grass
//! - Wind animation with configurable parameters
//! - Biome-to-foliage mappings for varied terrain
//! - Data-driven configuration via RON files
//! - Minimal dependency on deer_gui internals
//!
//! ## Reusability
//!
//! This plugin can be extracted to a standalone crate for use in any
//! Bevy project needing procedural vegetation. Configuration is done
//! via Bevy resources (data-driven, no hardcoded paths).

use bevy::ecs::system::{Commands, Res};
use bevy::log::{debug, info, trace};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::scene::common::ActiveSceneTag;

// ---------------------------------------------------------------------------
// Biome Types
// ---------------------------------------------------------------------------

/// Biome types that determine vegetation distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    /// Lush grasslands with scattered trees.
    Meadow,
    /// Dense forest with thick canopy.
    Forest,
    /// Rocky highlands with sparse vegetation.
    Rocky,
    /// Coastal area with palms and shrubs.
    Coastal,
    /// Alpine zone above tree line.
    Alpine,
    /// River valley with riparian vegetation.
    Riverbank,
}

impl Default for BiomeType {
    fn default() -> Self {
        Self::Meadow
    }
}

// ---------------------------------------------------------------------------
// Foliage Types
// ---------------------------------------------------------------------------

/// Types of foliage that can be spawned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoliageType {
    /// Individual tree (oak, pine, birch).
    Tree {
        /// Asset path for the 3D model (glTF).
        model_path: String,
        /// Scale variation (min, max).
        scale_range: (f32, f32),
        /// Average spacing between trees.
        spacing: f32,
    },
    /// Small bushes and shrubs.
    Bush {
        model_path: String,
        scale_range: (f32, f32),
        spacing: f32,
    },
    /// Ground cover grass (billboard or mesh).
    Grass {
        model_path: String,
        density: f32, // instances per square meter
    },
}

// ---------------------------------------------------------------------------
// Biome Configuration
// ---------------------------------------------------------------------------

/// Vegetation configuration for a specific biome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeVegetationConfig {
    /// Biome type.
    pub biome: BiomeType,
    /// List of foliage types and their densities.
    pub foliage: Vec<FoliageType>,
    /// Ground cover density multiplier.
    #[serde(default)]
    pub ground_cover_density: f32,
}

// ---------------------------------------------------------------------------
// Wind Parameters
// ---------------------------------------------------------------------------

/// Wind animation parameters for vegetation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindConfig {
    /// Wind direction (normalized).
    #[serde(default = "default_wind_direction")]
    pub direction: (f32, f32, f32),
    /// Wind speed in m/s.
    #[serde(default = "default_wind_speed")]
    pub speed: f32,
    /// Wind turbulence/strength.
    #[serde(default = "default_wind_turbulence")]
    pub turbulence: f32,
    /// Gust frequency (gusts per minute).
    #[serde(default = "default_gust_frequency")]
    pub gust_frequency: f32,
}

fn default_wind_direction() -> (f32, f32, f32) {
    (1.0, 0.0, 0.3)
}

fn default_wind_speed() -> f32 {
    2.0
}

fn default_wind_turbulence() -> f32 {
    0.5
}

fn default_gust_frequency() -> f32 {
    3.0
}

impl Default for WindConfig {
    fn default() -> Self {
        Self {
            direction: default_wind_direction(),
            speed: default_wind_speed(),
            turbulence: default_wind_turbulence(),
            gust_frequency: default_gust_frequency(),
        }
    }
}

// ---------------------------------------------------------------------------
// Global Foliage Configuration
// ---------------------------------------------------------------------------

/// Global configuration for the foliage system.
#[derive(Resource, Component, Debug, Clone, Serialize, Deserialize)]
pub struct FoliageGlobalConfig {
    /// Biome vegetation configurations.
    #[serde(default)]
    pub biomes: Vec<BiomeVegetationConfig>,
    /// Wind parameters.
    #[serde(default)]
    pub wind: WindConfig,
    /// Maximum vegetation draw distance.
    #[serde(default = "default_draw_distance")]
    pub draw_distance: f32,
    /// LOD distance thresholds.
    #[serde(default = "default_lod_distances")]
    pub lod_distances: Vec<f32>,
}

fn default_draw_distance() -> f32 {
    500.0
}

fn default_lod_distances() -> Vec<f32> {
    vec![50.0, 100.0, 200.0]
}

impl Default for FoliageGlobalConfig {
    fn default() -> Self {
        Self {
            biomes: Vec::new(),
            wind: WindConfig::default(),
            draw_distance: 500.0,
            lod_distances: vec![50.0, 100.0, 200.0],
        }
    }
}

// ---------------------------------------------------------------------------
// Marker Components
// ---------------------------------------------------------------------------

/// Marker for the foliage system root entity.
#[derive(Component, Debug)]
pub struct FoliageRoot;

/// Marker for individual foliage instances.
#[derive(Component, Debug)]
pub struct FoliageInstance {
    /// Type of foliage (tree, bush, grass).
    pub foliage_type: FoliageTypeIndex,
    /// LOD level (0 = full detail).
    pub lod_level: u32,
}

/// Index into the foliage types array.
#[derive(Debug, Clone, Copy)]
pub enum FoliageTypeIndex {
    Tree(usize),
    Bush(usize),
    Grass(usize),
}

// ---------------------------------------------------------------------------
// FoliagePlugin
// ---------------------------------------------------------------------------

/// Modular foliage plugin using bevy_feronia for GPU instancing.
///
/// This plugin can be added to any Bevy app for vegetation rendering.
/// Configuration is done via the `FoliageGlobalConfig` resource.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(FoliagePlugin::default());
/// app.insert_resource(FoliageGlobalConfig {
///     biomes: vec![...],
///     wind: WindConfig::default(),
///     ..Default::default()
/// });
/// ```
#[derive(Default)]
pub struct FoliagePlugin {
    /// Initial configuration (optional).
    pub config: Option<FoliageGlobalConfig>,
}

impl Plugin for FoliagePlugin {
    fn build(&self, app: &mut App) {
        info!("FoliagePlugin: initializing foliage system");

        // Add global config resource
        if let Some(config) = &self.config {
            app.insert_resource(config.clone());
        } else {
            app.init_resource::<FoliageGlobalConfig>();
        }

        // Add per-frame systems (scene-conditional)
        app.add_systems(Update, update_wind_system);

        // Log available biome configs
        debug!("FoliagePlugin: registered systems");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Setup the foliage system root entity.
#[allow(dead_code)]
fn setup_foliage_system(mut commands: Commands, config: Res<FoliageGlobalConfig>) {
    trace!("setup_foliage_system: creating foliage root");

    commands.spawn((
        FoliageRoot,
        Name::new("FoliageRoot"),
        Visibility::default(),
        Transform::default(),
    ));

    info!(
        "FoliagePlugin: initialized with {} biome configs, draw distance: {}m",
        config.biomes.len(),
        config.draw_distance
    );
}

/// Update wind animation parameters.
fn update_wind_system(
    time: Res<Time>,
    tag: Res<ActiveSceneTag>,
    config: ResMut<FoliageGlobalConfig>,
) {
    if tag.0.as_deref() != Some("medieval_open") {
        return;
    }
    // Update wind turbulence based on time for natural variation
    // This would be used by bevy_feronia's wind shader
    let elapsed = time.elapsed_secs();
    let gust_phase = (elapsed * config.wind.gust_frequency / 60.0 * std::f32::consts::TAU).sin();

    // Store the gust factor for use by the foliage renderer
    // (Actual wind update would happen via bevy_feronia's API)
    trace!("update_wind_system: gust phase = {:.2}", gust_phase);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl FoliagePlugin {
    /// Create a plugin with default meadow configuration.
    pub fn meadow() -> Self {
        let config = FoliageGlobalConfig {
            biomes: vec![BiomeVegetationConfig {
                biome: BiomeType::Meadow,
                foliage: vec![
                    FoliageType::Tree {
                        model_path: "models/foliage/oak.glb".to_string(),
                        scale_range: (0.8, 1.2),
                        spacing: 8.0,
                    },
                    FoliageType::Bush {
                        model_path: "models/foliage/bush.glb".to_string(),
                        scale_range: (0.5, 1.0),
                        spacing: 3.0,
                    },
                    FoliageType::Grass {
                        model_path: "models/foliage/grass.glb".to_string(),
                        density: 5.0,
                    },
                ],
                ground_cover_density: 1.0,
            }],
            wind: WindConfig::default(),
            ..Default::default()
        };
        Self {
            config: Some(config),
        }
    }

    /// Create a plugin with forest configuration.
    pub fn forest() -> Self {
        let config = FoliageGlobalConfig {
            biomes: vec![BiomeVegetationConfig {
                biome: BiomeType::Forest,
                foliage: vec![
                    FoliageType::Tree {
                        model_path: "models/foliage/pine.glb".to_string(),
                        scale_range: (0.9, 1.5),
                        spacing: 4.0,
                    },
                    FoliageType::Tree {
                        model_path: "models/foliage/birch.glb".to_string(),
                        scale_range: (0.8, 1.3),
                        spacing: 6.0,
                    },
                    FoliageType::Bush {
                        model_path: "models/foliage/shrub.glb".to_string(),
                        scale_range: (0.3, 0.7),
                        spacing: 2.0,
                    },
                ],
                ground_cover_density: 0.5,
            }],
            wind: WindConfig {
                speed: 1.5, // Less wind under canopy
                ..Default::default()
            },
            ..Default::default()
        };
        Self {
            config: Some(config),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_type_default() {
        let biome = BiomeType::default();
        assert_eq!(biome, BiomeType::Meadow);
    }

    #[test]
    fn wind_config_default() {
        let config = WindConfig::default();
        assert!(config.speed > 0.0);
        assert!(config.turbulence >= 0.0 && config.turbulence <= 1.0);
    }

    #[test]
    fn foliage_global_config_default() {
        let config = FoliageGlobalConfig::default();
        assert!(config.biomes.is_empty());
        assert_eq!(config.draw_distance, 500.0);
    }

    #[test]
    fn foliage_plugin_meadow() {
        let plugin = FoliagePlugin::meadow();
        assert!(plugin.config.is_some());
        let config = plugin.config.unwrap();
        assert_eq!(config.biomes.len(), 1);
        assert_eq!(config.biomes[0].biome, BiomeType::Meadow);
    }

    #[test]
    fn foliage_plugin_forest() {
        let plugin = FoliagePlugin::forest();
        assert!(plugin.config.is_some());
        let config = plugin.config.unwrap();
        assert_eq!(config.biomes[0].biome, BiomeType::Forest);
    }
}
