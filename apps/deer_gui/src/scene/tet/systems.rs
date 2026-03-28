//! Per-frame TET scene systems — glow pulsing and data trail movement.
//!
//! All constants sourced from [`crate::constants`].

use bevy::asset::Assets;
use bevy::log::trace;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Query, Res, ResMut, Time, Transform};

use super::setup::{DataTrail, TetMonolith};
use crate::constants::timing::BEACON_PULSE_HZ;
use crate::constants::visual::{
    DATA_TRAIL_SPEED, TET_GLOW_MAX, TET_GLOW_MIN, TET_STRUCTURE_RADIUS,
};

// ---------------------------------------------------------------------------
// TET glow
// ---------------------------------------------------------------------------

/// Pulses the TET monolith's emissive intensity between
/// [`TET_GLOW_MIN`] and [`TET_GLOW_MAX`] using a sine wave
/// at [`BEACON_PULSE_HZ`].
pub fn tet_glow_system(
    time: Res<Time>,
    monolith_query: Query<&MeshMaterial3d<StandardMaterial>, With<TetMonolith>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    // Gracefully no-op when AssetPlugin is absent (e.g. headless / test).
    let Some(mut materials) = materials else {
        return;
    };

    let elapsed = time.elapsed_secs();
    let phase = (elapsed * BEACON_PULSE_HZ * std::f32::consts::TAU).sin();
    // Map sin [-1,1] to [TET_GLOW_MIN, TET_GLOW_MAX]
    let intensity = TET_GLOW_MIN + (TET_GLOW_MAX - TET_GLOW_MIN) * (phase * 0.5 + 0.5);

    for mat_handle in monolith_query.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.emissive = bevy::color::LinearRgba::new(
                0.3 * intensity,
                0.5 * intensity,
                1.0 * intensity,
                1.0,
            );

            trace!(
                "tet_glow: elapsed={:.2} phase={:.3} intensity={:.3}",
                elapsed,
                phase,
                intensity,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Data trails
// ---------------------------------------------------------------------------

/// Advances [`DataTrail`] entities along their spiral paths at
/// [`DATA_TRAIL_SPEED`], wrapping `t` back to 0.0 on overflow.
pub fn data_trail_system(
    time: Res<Time>,
    mut trail_query: Query<(&mut DataTrail, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let advance = DATA_TRAIL_SPEED * dt * 0.01; // normalize speed to [0,1] range

    for (mut trail, mut tf) in trail_query.iter_mut() {
        trail.t = (trail.t + advance) % 1.0;
        tf.translation = spiral_position(trail.t, trail.index);

        trace!(
            "data_trail: index={} t={:.4} pos=({:.1},{:.1},{:.1})",
            trail.index,
            trail.t,
            tf.translation.x,
            tf.translation.y,
            tf.translation.z,
        );
    }
}

/// Compute a spiral position (mirrors setup::spiral_position).
///
/// Duplicated here intentionally to keep modules loosely coupled —
/// setup owns spawning, systems owns per-frame motion.
fn spiral_position(t: f32, index: usize) -> bevy::math::Vec3 {
    let phase = index as f32 * 0.618;
    let angle = t * std::f32::consts::TAU * 3.0 + phase;
    let r = TET_STRUCTURE_RADIUS * 1.5 + t * TET_STRUCTURE_RADIUS;
    let y = (t - 0.5) * TET_STRUCTURE_RADIUS * 2.0;

    bevy::math::Vec3::new(r * angle.cos(), y, r * angle.sin())
}

// ---------------------------------------------------------------------------
// Imports for Bevy component queries
// ---------------------------------------------------------------------------

use bevy::prelude::{MeshMaterial3d, With};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spiral_wraps_smoothly() {
        let a = spiral_position(0.99, 0);
        let b = spiral_position(0.01, 0);
        // They should be near each other (wrap-around)
        // but not identical since it's a multi-turn spiral.
        let dist = (a - b).length();
        assert!(dist < TET_STRUCTURE_RADIUS * 5.0);
    }

    #[test]
    fn glow_range() {
        // Verify constant ordering
        assert!(TET_GLOW_MIN < TET_GLOW_MAX);
        assert!(TET_GLOW_MIN >= 0.0);
    }
}
