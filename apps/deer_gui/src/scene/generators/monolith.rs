//! Monolith generator — spawns a central emissive sphere structure.
//!
//! Used for the TET scene's central tetrahedral structure.

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

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marker component for monolith entities.
#[derive(Component, Debug, Default)]
pub struct Monolith;

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Spawn a central monolith sphere with emissive material.
pub fn gen_monolith(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Monolith { radius, emissive } = params else {
        warn!("gen_monolith: expected Monolith params");
        return;
    };

    let monolith_mesh = meshes.add(Mesh::from(Sphere::new(*radius)));
    let monolith_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    commands.spawn((
        Monolith,
        ChildOf(root),
        Mesh3d(monolith_mesh),
        MeshMaterial3d(monolith_material),
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
        InheritedVisibility::default(),
    ));

    debug!("gen_monolith: radius={radius} emissive={emissive:?}");
}
