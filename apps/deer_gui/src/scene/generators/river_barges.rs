//! River barges generator — spawns parametric horizontal flow entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    ChildOf, Component, Entity, InheritedVisibility, Mesh, Mesh3d, MeshMaterial3d, Sphere,
    Transform, Visibility,
};

use crate::scene::descriptor::GeneratorParams;

/// Parametric river barge entity.
#[derive(Component, Debug, Clone)]
pub struct Barge {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
    pub river_radius: f32,
}

/// Spawn river barge entities along a horizontal parametric path.
pub fn gen_river_barges(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::RiverBarges {
        count,
        speed,
        river_radius,
        emissive,
    } = params
    else {
        warn!("gen_river_barges: expected RiverBarges params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.4)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = barge_position(t, i, *river_radius);

        commands.spawn((
            Barge {
                t,
                index: i,
                speed: *speed,
                river_radius: *river_radius,
            },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }

    debug!("gen_river_barges: count={count} speed={speed} radius={river_radius}");
}

/// Compute barge position along a sinusoidal river path.
pub fn barge_position(t: f32, index: usize, river_radius: f32) -> Vec3 {
    let phase = index as f32 * 0.37;
    let x = (t * 2.0 - 1.0) * river_radius;
    let z = (t * std::f32::consts::TAU + phase).sin() * river_radius * 0.15;
    let y = -50.0 + (index as f32 * 1.7).sin() * 5.0;
    Vec3::new(x, y, z)
}
