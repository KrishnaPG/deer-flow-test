//! Path travellers generator — diagonal parametric flow entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform, InheritedVisibility, Visibility};

use crate::scene::descriptor::GeneratorParams;

#[derive(Component, Debug, Clone)]
pub struct Traveller {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
    pub path_radius: f32,
}

pub fn gen_path_travellers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::PathTravellers {
        count,
        speed,
        path_radius,
        emissive,
    } = params
    else {
        warn!("gen_path_travellers: expected PathTravellers params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = traveller_position(t, i, *path_radius);
        commands.spawn((
            Traveller {
                t,
                index: i,
                speed: *speed,
                path_radius: *path_radius,
            },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }
    debug!("gen_path_travellers: count={count} speed={speed}");
}

pub fn traveller_position(t: f32, index: usize, path_radius: f32) -> Vec3 {
    let phase = index as f32 * 0.53;
    let angle = t * std::f32::consts::TAU + phase;
    let r = path_radius * (0.7 + 0.3 * t);
    let y = (t - 0.5) * path_radius * 0.5;
    Vec3::new(r * angle.cos(), y, r * angle.sin())
}
