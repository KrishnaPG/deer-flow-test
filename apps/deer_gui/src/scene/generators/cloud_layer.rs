//! Cloud layer generator — vertical rising particle entities.

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

#[derive(Component, Debug, Clone)]
pub struct CloudParticle {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

pub fn gen_cloud_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::CloudLayer {
        count,
        speed,
        radius,
        emissive,
    } = params
    else {
        warn!("gen_cloud_layer: expected CloudLayer params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(1.5)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        alpha_mode: bevy::prelude::AlphaMode::Blend,
        base_color: bevy::color::Color::srgba(0.8, 0.85, 0.9, 0.3),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = cloud_position(t, i, *radius);
        commands.spawn((
            CloudParticle {
                t,
                index: i,
                speed: *speed,
            },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::new(3.0, 0.5, 3.0)),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }
    debug!("gen_cloud_layer: count={count} speed={speed} radius={radius}");
}

pub fn cloud_position(t: f32, index: usize, radius: f32) -> Vec3 {
    let phase = index as f32 * 2.39; // golden angle
    let x = (phase).cos() * radius * (0.3 + 0.7 * t);
    let z = (phase).sin() * radius * (0.3 + 0.7 * t);
    let y = (t - 0.5) * radius * 2.0;
    Vec3::new(x, y, z)
}
