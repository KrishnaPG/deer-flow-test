//! TET scene startup — spawns starfield, monolith, and data trails.
//!
//! All entity counts and radii are sourced from [`crate::constants::visual`].

use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::constants::visual::{
    DATA_TRAIL_COUNT, DATA_TRAIL_SPEED, STARFIELD_COUNT, STARFIELD_RADIUS, TET_STRUCTURE_RADIUS,
};
use crate::scene::common::parallax::ParallaxLayer;
use crate::scene::manager::SceneRoot;
use crate::theme::ThemeManager;

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

/// Top-level startup system (legacy entry-point, kept for compatibility).
///
/// Delegates to [`spawn_tet_environment`] with fresh asset stores.
pub fn tet_scene_setup_system(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    theme: Option<Res<ThemeManager>>,
) {
    // Gracefully no-op when AssetPlugin is absent (e.g. headless / test).
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };

    info!("tet_scene_setup_system — begin");
    let theme_ref = theme.as_deref();
    let root = spawn_tet_environment(&mut commands, &mut meshes, &mut materials, theme_ref);
    info!("tet_scene_setup_system — complete, root={root:?}");
}

/// Spawn all TET scene entities: starfield, monolith, and data trails.
///
/// Returns the root [`Entity`] tagged with [`SceneRoot`] that parents
/// all spawned entities. Despawning this entity removes the entire scene.
///
/// When `theme` is `Some`, material colours are read from the active
/// theme; otherwise sensible defaults (matching the TET palette) are used.
pub fn spawn_tet_environment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    theme: Option<&ThemeManager>,
) -> Entity {
    let root = commands.spawn((SceneRoot, Transform::default())).id();
    debug!("spawn_tet_environment: created scene root={root:?}");

    // Extract colours from theme or fall back to TET defaults.
    let (star_em, mono_em, trail_em, trail_base) = extract_world_colors(theme);

    spawn_starfield(commands, meshes, materials, root, star_em);
    spawn_monolith(commands, meshes, materials, root, mono_em);
    spawn_data_trails(commands, meshes, materials, root, trail_em, trail_base);

    root
}

/// Extracts world-material colours from the active theme, or returns
/// hard-coded TET defaults when no [`ThemeManager`] is available.
fn extract_world_colors(
    theme: Option<&ThemeManager>,
) -> (
    bevy::color::LinearRgba,
    bevy::color::LinearRgba,
    bevy::color::LinearRgba,
    Color,
) {
    match theme {
        Some(tm) => {
            let t = tm.current();
            debug!("extract_world_colors: using theme '{}'", t.name);
            (
                t.star_emissive,
                t.monolith_emissive,
                t.trail_emissive,
                t.trail_base_color,
            )
        }
        None => {
            debug!("extract_world_colors: no ThemeManager, using TET defaults");
            (
                bevy::color::LinearRgba::new(2.0, 2.0, 2.0, 1.0),
                bevy::color::LinearRgba::new(0.3, 0.5, 1.0, 1.0),
                bevy::color::LinearRgba::new(0.0, 1.5, 0.8, 1.0),
                Color::srgb(0.0, 0.8, 0.5),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Starfield
// ---------------------------------------------------------------------------

/// Spawns [`STARFIELD_COUNT`] star entities at pseudo-random positions
/// within a sphere of [`STARFIELD_RADIUS`].
fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
) {
    let star_mesh = meshes.add(Mesh::from(Sphere::new(0.5)));
    let star_material = materials.add(StandardMaterial {
        emissive,
        ..Default::default()
    });

    for i in 0..STARFIELD_COUNT {
        let pos = pseudo_random_sphere_point(i, STARFIELD_COUNT, STARFIELD_RADIUS);
        let depth = (pos.length() / STARFIELD_RADIUS).clamp(0.3, 1.0);

        commands.spawn((
            Star,
            ChildOf(root),
            ParallaxLayer::new(depth),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(random_scale(i))),
        ));
    }

    debug!(
        "spawn_starfield: spawned {} stars under root={root:?}",
        STARFIELD_COUNT,
    );
}

// ---------------------------------------------------------------------------
// Monolith
// ---------------------------------------------------------------------------

/// Spawns the central TET monolith — an icosphere with emissive material.
fn spawn_monolith(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
) {
    let monolith_mesh = meshes.add(Mesh::from(Sphere::new(TET_STRUCTURE_RADIUS)));
    let monolith_material = materials.add(StandardMaterial {
        emissive,
        ..Default::default()
    });

    commands.spawn((
        TetMonolith,
        ChildOf(root),
        Mesh3d(monolith_mesh),
        MeshMaterial3d(monolith_material),
        Transform::from_translation(Vec3::ZERO),
    ));

    debug!(
        "spawn_monolith: TET at origin, radius={}, root={root:?}",
        TET_STRUCTURE_RADIUS,
    );
}

// ---------------------------------------------------------------------------
// Data Trails
// ---------------------------------------------------------------------------

/// Spawns [`DATA_TRAIL_COUNT`] data-trail markers along spiral paths.
fn spawn_data_trails(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
    base_color: Color,
) {
    let trail_mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let trail_material = materials.add(StandardMaterial {
        emissive,
        base_color,
        ..Default::default()
    });

    for i in 0..DATA_TRAIL_COUNT {
        let t = i as f32 / DATA_TRAIL_COUNT as f32;
        let pos = spiral_position(t, i);

        commands.spawn((
            DataTrail { t, index: i },
            ChildOf(root),
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(trail_material.clone()),
            Transform::from_translation(pos),
        ));
    }

    debug!(
        "spawn_data_trails: spawned {} trails, speed={}, root={root:?}",
        DATA_TRAIL_COUNT, DATA_TRAIL_SPEED,
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
///
/// Delegates to the shared helper in [`super::spiral_position`].
fn spiral_position(t: f32, index: usize) -> Vec3 {
    super::spiral_position(t, index)
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
