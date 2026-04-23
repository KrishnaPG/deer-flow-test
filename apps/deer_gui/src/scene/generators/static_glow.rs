//! Static glow cluster generator — fixed emissive entities.

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
use crate::scene::primitives;

#[derive(Component, Debug, Default)]
pub struct GlowEntity;

pub fn gen_static_glow_cluster(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::StaticGlowCluster {
        count,
        emissive,
        position,
        spread,
    } = params
    else {
        warn!("gen_static_glow_cluster: expected StaticGlowCluster params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(1.0)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    let center = Vec3::new(position[0], position[1], position[2]);

    for i in 0..*count {
        let offset = primitives::fibonacci_sphere_point(i, *count, *spread);
        let pos = center + offset;
        let scale = primitives::entity_scale(i) * 0.8;

        commands.spawn((
            GlowEntity,
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }
    debug!("gen_static_glow_cluster: count={count} spread={spread}");
}
