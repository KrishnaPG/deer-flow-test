//! Shared scene primitives — reusable spawn helpers for all scenes.
//!
//! Scene-independent functions for spawning starfields, ambient lights,
//! and computing Fibonacci-sphere distributions.

use bevy::asset::Assets;
use bevy::log::{debug, trace};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    ChildOf, Component, Entity, InheritedVisibility, Mesh, Mesh3d, MeshMaterial3d, Sphere,
    Transform, Visibility,
};

use crate::scene::common::parallax::ParallaxLayer;
use crate::scene::manager::SceneRoot;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tag for starfield entities. Scene-independent — used by any scene
/// that spawns a Fibonacci-sphere star distribution.
#[derive(Component, Debug, Default)]
pub struct Star;

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Spawn a [`SceneRoot`] entity and return its [`Entity`] id.
///
/// The root entity serves as the parent for all scene-specific entities.
/// Despawning it recursively removes the entire scene.
pub fn spawn_root(commands: &mut bevy::ecs::system::Commands) -> Entity {
    let root = commands
        .spawn((
            SceneRoot,
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id();
    debug!("primitives::spawn_root — root={root:?}");
    root
}

/// Spawn `count` stars distributed on a Fibonacci sphere of `radius`.
///
/// Each star is an emissive sphere mesh with a [`ParallaxLayer`] for
/// depth-based parallax scrolling. All entities are `ChildOf(root)`.
pub fn spawn_starfield(
    commands: &mut bevy::ecs::system::Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
    count: usize,
    radius: f32,
) {
    let star_mesh = meshes.add(Mesh::from(Sphere::new(0.5)));
    let star_material = materials.add(StandardMaterial {
        emissive,
        ..Default::default()
    });

    for i in 0..count {
        let pos = fibonacci_sphere_point(i, count, radius);
        let depth = (pos.length() / radius).clamp(0.3, 1.0);

        commands.spawn((
            Star,
            ChildOf(root),
            ParallaxLayer::new(depth),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(entity_scale(i))),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }

    debug!("primitives::spawn_starfield — count={count}, radius={radius}, root={root:?}");
}

/// Spawn a single ambient-light entity parented to `root`.
pub fn spawn_scene_ambient_light(
    commands: &mut bevy::ecs::system::Commands,
    root: Entity,
    brightness: f32,
    color: bevy::color::LinearRgba,
) {
    use bevy::prelude::AmbientLight;
    commands.spawn((
        ChildOf(root),
        AmbientLight {
            color: bevy::color::Color::LinearRgba(color),
            brightness,
            affects_lightmapped_meshes: true,
        },
    ));
    trace!("primitives::spawn_scene_ambient_light — brightness={brightness}, root={root:?}");
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Deterministic Fibonacci-sphere point at index `i` of `total`.
///
/// Uses the golden-ratio distribution for uniform coverage.
pub fn fibonacci_sphere_point(index: usize, total: usize, radius: f32) -> Vec3 {
    let golden = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let i = index as f32;
    let n = total as f32;

    let theta = 2.0 * std::f32::consts::PI * i / golden;
    let phi = (1.0 - 2.0 * (i + 0.5) / n).acos();

    // Vary the radius slightly for depth.
    let r = radius * (0.6 + 0.4 * ((i * 7.13).sin() * 0.5 + 0.5));

    Vec3::new(
        r * phi.sin() * theta.cos(),
        r * phi.sin() * theta.sin(),
        r * phi.cos(),
    )
}

/// Deterministic per-entity scale in `[0.3, 1.0]` using the index.
pub fn entity_scale(index: usize) -> f32 {
    let i = index as f32;
    0.3 + 0.7 * ((i * 3.17).sin() * 0.5 + 0.5)
}
