//! Foliage generator — configures biome vegetation for a scene.
//!
//! Spawns foliage root and sets up biome configuration.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::prelude::{ChildOf, Entity, InheritedVisibility, Name, Transform, Visibility};

use crate::scene::descriptor::GeneratorParams;
use crate::world::foliage::{
    BiomeType, BiomeVegetationConfig, FoliageGlobalConfig, FoliageRoot, FoliageType, WindConfig,
};

/// Generate foliage configuration from scene descriptor parameters.
pub fn gen_foliage(
    commands: &mut Commands,
    _meshes: &mut bevy::asset::Assets<bevy::prelude::Mesh>,
    _materials: &mut bevy::asset::Assets<bevy::pbr::StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Foliage {
        biome,
        wind_speed,
        draw_distance,
    } = params
    else {
        warn!("gen_foliage: expected Foliage params");
        return;
    };

    info!("gen_foliage: configuring biome='{}'", biome);

    let foliage_config = FoliageGlobalConfig {
        biomes: vec![create_biome_config(biome)],
        wind: WindConfig {
            speed: *wind_speed,
            ..Default::default()
        },
        draw_distance: *draw_distance,
        ..Default::default()
    };

    commands.insert_resource(foliage_config);

    let foliage_root = commands
        .spawn((
            FoliageRoot,
            Name::new("FoliageRoot"),
            Visibility::default(),
            Transform::default(),
        ))
        .id();

    commands.entity(root).add_child(foliage_root);
    debug!("gen_foliage: spawned foliage root under scene");
}

fn create_biome_config(biome: &str) -> BiomeVegetationConfig {
    match biome {
        "Meadow" => BiomeVegetationConfig {
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
            ],
            ground_cover_density: 1.0,
        },
        "Forest" => BiomeVegetationConfig {
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
        },
        _ => BiomeVegetationConfig {
            biome: BiomeType::Meadow,
            foliage: vec![],
            ground_cover_density: 0.0,
        },
    }
}
