//! Water generator — configures water bodies for a scene.
//!
//! Spawns water body markers and configures bevy_water settings.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::prelude::{ChildOf, Entity, InheritedVisibility, Name, Transform, Visibility};

use bevy_water::WaterSettings;

use crate::scene::descriptor::GeneratorParams;
use crate::world::water::{WaterBody, WaterBodyDef, WaterGlobalConfig, WaterType};

/// Generate water bodies from scene descriptor parameters.
pub fn gen_water(
    commands: &mut Commands,
    _meshes: &mut bevy::asset::Assets<bevy::prelude::Mesh>,
    _materials: &mut bevy::asset::Assets<bevy::pbr::StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Water { bodies } = params else {
        warn!("gen_water: expected Water params");
        return;
    };

    info!("gen_water: spawning {} water bodies", bodies.len());

    // Insert global config
    let water_level = bodies.first().map(|b| b.position.1).unwrap_or(0.0);
    commands.insert_resource(WaterGlobalConfig {
        bodies: bodies.clone(),
        water_level,
        enable_splash: true,
        enable_swimming: true,
    });

    commands.insert_resource(WaterSettings {
        height: water_level,
        ..Default::default()
    });

    for body in bodies {
        let entity = commands
            .spawn((
                WaterBody {
                    id: body.id.clone(),
                    water_type: body.water_type,
                    wave_time: 0.0,
                },
                Name::new(format!("Water:{}", body.name)),
                Transform::from_translation(bevy::math::Vec3::new(
                    body.position.0,
                    body.position.1,
                    body.position.2,
                )),
                Visibility::default(),
                InheritedVisibility::default(),
            ))
            .id();

        commands.entity(root).add_child(entity);
        debug!("gen_water: spawned '{}' at {:?}", body.name, body.position);
    }
}
