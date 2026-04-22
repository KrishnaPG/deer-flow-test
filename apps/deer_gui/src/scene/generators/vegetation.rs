//! Vegetation generator for biome-based foliage scattering.
//!
//! Generates vegetation clusters using the FoliagePlugin configuration
//! based on biome type and scatter parameters.

use bevy::asset::{AssetServer, Assets};
use bevy::log::{debug, info, trace, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use crate::scene::descriptor::GeneratorParams;
use crate::world::foliage::{
    BiomeType, BiomeVegetationConfig, FoliageGlobalConfig, FoliageRoot, FoliageType, WindConfig,
};

/// Vegetation generator configuration.
#[derive(Debug, Clone)]
pub struct VegetationGenerator {
    /// Biome type for vegetation selection.
    pub biome: BiomeType,
    /// Center position for scatter.
    pub center: Vec3,
    /// Scatter radius in meters.
    pub radius: f32,
    /// Density multiplier.
    pub density: f32,
    /// Enable wind animation.
    pub enable_wind: bool,
    /// Draw distance in meters.
    pub draw_distance: f32,
}

impl VegetationGenerator {
    /// Create a new vegetation generator from scene descriptor params.
    pub fn from_params(params: &GeneratorParams) -> Option<Self> {
        match params {
            GeneratorParams::Vegetation {
                biome,
                center,
                radius,
                density,
                enable_wind,
                draw_distance,
            } => {
                let biome_type = match biome.as_str() {
                    "Meadow" => BiomeType::Meadow,
                    "Forest" => BiomeType::Forest,
                    "Rocky" => BiomeType::Rocky,
                    "Coastal" => BiomeType::Coastal,
                    "Alpine" => BiomeType::Alpine,
                    "Riverbank" => BiomeType::Riverbank,
                    _ => {
                        warn!("Unknown biome type '{}', defaulting to Meadow", biome);
                        BiomeType::Meadow
                    }
                };

                Some(Self {
                    biome: biome_type,
                    center: Vec3::from_array(*center),
                    radius: *radius,
                    density: *density,
                    enable_wind: *enable_wind,
                    draw_distance: *draw_distance,
                })
            }
            _ => None,
        }
    }

    /// Get the default foliage config for this biome.
    pub fn get_foliage_config(&self) -> FoliageGlobalConfig {
        let biome_config = match self.biome {
            BiomeType::Meadow => BiomeVegetationConfig {
                biome: BiomeType::Meadow,
                foliage: vec![
                    FoliageType::Tree {
                        model_path: "models/foliage/oak.glb".to_string(),
                        scale_range: (0.8, 1.2),
                        spacing: 8.0 / self.density,
                    },
                    FoliageType::Bush {
                        model_path: "models/foliage/bush.glb".to_string(),
                        scale_range: (0.5, 1.0),
                        spacing: 3.0 / self.density,
                    },
                    FoliageType::Grass {
                        model_path: "models/foliage/grass.glb".to_string(),
                        density: 5.0 * self.density,
                    },
                ],
                ground_cover_density: self.density,
            },
            BiomeType::Forest => BiomeVegetationConfig {
                biome: BiomeType::Forest,
                foliage: vec![
                    FoliageType::Tree {
                        model_path: "models/foliage/pine.glb".to_string(),
                        scale_range: (0.9, 1.5),
                        spacing: 4.0 / self.density,
                    },
                    FoliageType::Tree {
                        model_path: "models/foliage/birch.glb".to_string(),
                        scale_range: (0.8, 1.3),
                        spacing: 6.0 / self.density,
                    },
                    FoliageType::Bush {
                        model_path: "models/foliage/shrub.glb".to_string(),
                        scale_range: (0.3, 0.7),
                        spacing: 2.0 / self.density,
                    },
                ],
                ground_cover_density: 0.5 * self.density,
            },
            _ => {
                // Generic fallback
                BiomeVegetationConfig {
                    biome: self.biome,
                    foliage: vec![FoliageType::Tree {
                        model_path: "models/foliage/oak.glb".to_string(),
                        scale_range: (0.8, 1.2),
                        spacing: 10.0 / self.density,
                    }],
                    ground_cover_density: self.density,
                }
            }
        };

        FoliageGlobalConfig {
            biomes: vec![biome_config],
            wind: if self.enable_wind {
                WindConfig::default()
            } else {
                WindConfig {
                    speed: 0.0,
                    turbulence: 0.0,
                    gust_frequency: 0.0,
                    ..Default::default()
                }
            },
            draw_distance: self.draw_distance,
            ..Default::default()
        }
    }
}

/// Spawn vegetation in the scene.
pub fn spawn_vegetation(
    commands: &mut Commands,
    generator: &VegetationGenerator,
    _asset_server: &AssetServer,
) -> Entity {
    debug!(
        "spawn_vegetation: spawning {:?} biome at {:?} radius={}",
        generator.biome, generator.center, generator.radius
    );

    let config = generator.get_foliage_config();
    let biome_count = config.biomes.len();

    // Spawn vegetation root with config
    let entity = commands
        .spawn((
            Name::new(format!("Vegetation_{:?}", generator.biome)),
            Transform::from_translation(generator.center),
            FoliageRoot,
        ))
        .insert(config)
        .id();

    info!(
        "spawn_vegetation: created vegetation entity {:?} with {} biome configs",
        entity, biome_count
    );

    entity
}

/// Generator factory function for vegetation scattering.
pub fn gen_vegetation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    trace!("gen_vegetation: called with params {:?}", params);

    if let Some(generator) = VegetationGenerator::from_params(params) {
        // Note: In a real implementation, we would get the AssetServer from the World
        // For now, we'll create a placeholder entity
        let entity = commands
            .spawn((
                Name::new(format!("Vegetation_{:?}", generator.biome)),
                Transform::from_translation(generator.center),
                crate::world::foliage::FoliageRoot,
            ))
            .id();

        debug!(
            "gen_vegetation: created placeholder vegetation entity {:?}",
            entity
        );
    } else {
        warn!("gen_vegetation: invalid params for vegetation generator");
    }
}
