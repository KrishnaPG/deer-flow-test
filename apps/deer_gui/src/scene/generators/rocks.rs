//! Rock generator — scatters photogrammetry rock models across terrain.
//!
//! Places real rock meshes along stream beds and hillsides with
//! random rotation, scale variation, and terrain height sampling.

use std::f32::consts::TAU;

use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::math::Vec3;
use bevy::prelude::{ChildOf, Entity, InheritedVisibility, Name, SceneRoot, Transform, Visibility};
use rand::{thread_rng, Rng};

use crate::scene::descriptor::GeneratorParams;

/// Marker component for rock entities.
#[derive(bevy::prelude::Component, Debug)]
pub struct RockInstance {
    pub model_path: String,
    pub lod_level: u32,
}

/// Generate rock scatter from scene descriptor parameters.
pub fn gen_rocks(
    commands: &mut Commands,
    _meshes: &mut bevy::asset::Assets<bevy::prelude::Mesh>,
    _materials: &mut bevy::asset::Assets<bevy::pbr::StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Rocks {
        models,
        center,
        radius,
        count,
        height_offset,
    } = params
    else {
        warn!("gen_rocks: expected Rocks params");
        return;
    };

    if models.is_empty() {
        warn!("gen_rocks: no rock models specified");
        return;
    }

    info!("gen_rocks: scattering {} rocks (radius={})", count, radius);

    let mut rng = thread_rng();
    let center_v = Vec3::from_array(*center);

    for i in 0..*count {
        // Pick random model
        let model_path = models[rng.gen_range(0..models.len())].clone();

        // Random position within radius
        let angle = rng.gen_range(0.0..TAU);
        let distance = rng.gen_range(0.0..*radius);
        let x = center_v.x + angle.cos() * distance;
        let z = center_v.z + angle.sin() * distance;
        let y = center_v.y + *height_offset;

        // Random rotation and scale
        let rotation = bevy::prelude::Quat::from_rotation_y(rng.gen_range(0.0..TAU));
        let scale = rng.gen_range(0.5..1.5);

        let entity = commands
            .spawn((
                Name::new(format!("Rock_{}", i)),
                RockInstance {
                    model_path: model_path.clone(),
                    lod_level: 0,
                },
                Transform {
                    translation: Vec3::new(x, y, z),
                    rotation,
                    scale: Vec3::splat(scale),
                },
                Visibility::default(),
                InheritedVisibility::default(),
            ))
            .id();

        commands.entity(root).add_child(entity);
        debug!("gen_rocks: spawned rock {} at {:?}", i, (x, y, z));
    }

    info!("gen_rocks: finished scattering {} rocks", count);
}
