//! Spiral trails generator — spawns parametric spiral path entities.

use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    ChildOf, Component, Entity, InheritedVisibility, Mesh, Mesh3d, MeshMaterial3d, Sphere,
    Transform, Visibility,
};

use crate::constants::visual::TET_STRUCTURE_RADIUS;
use crate::scene::descriptor::GeneratorParams;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Tag + parametric data for spiral trail entities.
#[derive(Component, Debug, Clone)]
pub struct SpiralTrail {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Spawn spiral trail entities along parametric paths.
pub fn gen_spiral_trails(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::SpiralTrails {
        count,
        speed,
        emissive,
        base_color,
    } = params
    else {
        warn!("gen_spiral_trails: expected SpiralTrails params");
        return;
    };

    let trail_mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let trail_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], base_color[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = spiral_position(t, i);

        commands.spawn((
            SpiralTrail {
                t,
                index: i,
                speed: *speed,
            },
            ChildOf(root),
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(trail_material.clone()),
            Transform::from_translation(pos),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }

    debug!("gen_spiral_trails: count={count} speed={speed}");
}

/// Compute spiral position (same formula as TET's `spiral_position`).
pub fn spiral_position(t: f32, index: usize) -> Vec3 {
    let phase = index as f32 * 0.618;
    let angle = t * std::f32::consts::TAU * 3.0 + phase;
    let r = TET_STRUCTURE_RADIUS * 1.5 + t * TET_STRUCTURE_RADIUS;
    let y = (t - 0.5) * TET_STRUCTURE_RADIUS * 2.0;
    Vec3::new(r * angle.cos(), y, r * angle.sin())
}
