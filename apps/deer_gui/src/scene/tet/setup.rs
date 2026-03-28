//! TET scene startup — spawns starfield, monolith, and data trails.
//!
//! All entity counts and radii are sourced from [`crate::constants::visual`].

use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::{Commands, ResMut};
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Component, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::constants::visual::{
    DATA_TRAIL_COUNT, DATA_TRAIL_SPEED, STARFIELD_COUNT, STARFIELD_RADIUS, TET_STRUCTURE_RADIUS,
};
use crate::scene::common::parallax::ParallaxLayer;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tag for starfield entities.
#[derive(Component, Debug, Default)]
pub struct Star;

/// Tag for the central TET monolith entity.
#[derive(Component, Debug, Default)]
pub struct TetMonolith;

/// Tag + path data for data-trail entities.
#[derive(Component, Debug, Clone)]
pub struct DataTrail {
    /// Parametric position along the spiral path `[0.0, 1.0)`.
    pub t: f32,
    /// Index of this trail entity (for spiral phase offset).
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Startup system
// ---------------------------------------------------------------------------

/// Top-level startup system registered by [`ScenePlugin`].
///
/// Delegates to [`spawn_tet_environment`] with fresh asset stores.
pub fn tet_scene_setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("tet_scene_setup_system — begin");
    spawn_tet_environment(&mut commands, &mut meshes, &mut materials);
    info!("tet_scene_setup_system — complete");
}

/// Spawn all TET scene entities: starfield, monolith, and data trails.
pub fn spawn_tet_environment(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    spawn_starfield(commands, meshes, materials);
    spawn_monolith(commands, meshes, materials);
    spawn_data_trails(commands, meshes, materials);
}

// ---------------------------------------------------------------------------
// Starfield
// ---------------------------------------------------------------------------

/// Spawns [`STARFIELD_COUNT`] star entities at pseudo-random positions
/// within a sphere of [`STARFIELD_RADIUS`].
fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let star_mesh = meshes.add(Mesh::from(Sphere::new(0.5)));
    let star_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(2.0, 2.0, 2.0, 1.0),
        ..Default::default()
    });

    for i in 0..STARFIELD_COUNT {
        let pos = pseudo_random_sphere_point(i, STARFIELD_COUNT, STARFIELD_RADIUS);
        let depth = (pos.length() / STARFIELD_RADIUS).clamp(0.3, 1.0);

        commands.spawn((
            Star,
            ParallaxLayer::new(depth),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(random_scale(i))),
        ));
    }

    debug!("spawn_starfield: spawned {} stars", STARFIELD_COUNT);
}

// ---------------------------------------------------------------------------
// Monolith
// ---------------------------------------------------------------------------

/// Spawns the central TET monolith — an icosphere with emissive material.
fn spawn_monolith(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let monolith_mesh = meshes.add(Mesh::from(Sphere::new(TET_STRUCTURE_RADIUS)));
    let monolith_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(0.3, 0.5, 1.0, 1.0),
        ..Default::default()
    });

    commands.spawn((
        TetMonolith,
        Mesh3d(monolith_mesh),
        MeshMaterial3d(monolith_material),
        Transform::from_translation(Vec3::ZERO),
    ));

    debug!(
        "spawn_monolith: TET at origin, radius={}",
        TET_STRUCTURE_RADIUS
    );
}

// ---------------------------------------------------------------------------
// Data Trails
// ---------------------------------------------------------------------------

/// Spawns [`DATA_TRAIL_COUNT`] data-trail markers along spiral paths.
fn spawn_data_trails(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let trail_mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let trail_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(0.0, 1.5, 0.8, 1.0),
        base_color: Color::srgb(0.0, 0.8, 0.5),
        ..Default::default()
    });

    for i in 0..DATA_TRAIL_COUNT {
        let t = i as f32 / DATA_TRAIL_COUNT as f32;
        let pos = spiral_position(t, i);

        commands.spawn((
            DataTrail { t, index: i },
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(trail_material.clone()),
            Transform::from_translation(pos),
        ));
    }

    debug!(
        "spawn_data_trails: spawned {} trails, speed={}",
        DATA_TRAIL_COUNT, DATA_TRAIL_SPEED
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random point on a sphere using golden-ratio
/// distribution (Fibonacci sphere).
fn pseudo_random_sphere_point(index: usize, total: usize, radius: f32) -> Vec3 {
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

/// Deterministic per-star scale variation.
fn random_scale(index: usize) -> f32 {
    let i = index as f32;
    0.3 + 0.7 * ((i * 3.17).sin() * 0.5 + 0.5)
}

/// Compute a spiral position for a data trail at parametric position `t`.
fn spiral_position(t: f32, index: usize) -> Vec3 {
    let phase = index as f32 * 0.618; // golden ratio offset per trail
    let angle = t * std::f32::consts::TAU * 3.0 + phase;
    let r = TET_STRUCTURE_RADIUS * 1.5 + t * TET_STRUCTURE_RADIUS;
    let y = (t - 0.5) * TET_STRUCTURE_RADIUS * 2.0;

    Vec3::new(r * angle.cos(), y, r * angle.sin())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pseudo_random_sphere_within_radius() {
        for i in 0..100 {
            let p = pseudo_random_sphere_point(i, 100, 800.0);
            assert!(p.length() <= 800.0 + 1e-3);
        }
    }

    #[test]
    fn spiral_position_varies() {
        let a = spiral_position(0.0, 0);
        let b = spiral_position(0.5, 0);
        assert!((a - b).length() > 1.0);
    }

    #[test]
    fn random_scale_in_range() {
        for i in 0..200 {
            let s = random_scale(i);
            assert!(s >= 0.29 && s <= 1.01, "scale {} out of range", s);
        }
    }
}
