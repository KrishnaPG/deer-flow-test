//! Per-frame TET scene systems — glow pulsing and data trail movement.
//!
//! All constants sourced from [`crate::constants`].

use bevy::asset::Assets;
use bevy::log::trace;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{MeshMaterial3d, Query, Res, ResMut, Time, Transform, With};

use super::setup::{DataTrail, TetMonolith};
use crate::constants::timing::BEACON_PULSE_HZ;
use crate::constants::visual::{
    DATA_TRAIL_SPEED, TET_GLOW_MAX, TET_GLOW_MIN, TET_STRUCTURE_RADIUS,
};
use crate::theme::ThemeManager;

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
    theme: Option<Res<ThemeManager>>,
) {
    // Gracefully no-op when AssetPlugin is absent (e.g. headless / test).
    let Some(mut materials) = materials else {
        return;
    };

    let glow_channels = theme
        .as_ref()
        .map(|tm| tm.current().monolith_glow_channels)
        .unwrap_or([0.3, 0.5, 1.0]);

    let elapsed = time.elapsed_secs();
    let phase = (elapsed * BEACON_PULSE_HZ * std::f32::consts::TAU).sin();
    // Map sin [-1,1] to [TET_GLOW_MIN, TET_GLOW_MAX]
    let intensity = TET_GLOW_MIN + (TET_GLOW_MAX - TET_GLOW_MIN) * (phase * 0.5 + 0.5);

    for mat_handle in monolith_query.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.emissive = bevy::color::LinearRgba::new(
                glow_channels[0] * intensity,
                glow_channels[1] * intensity,
                glow_channels[2] * intensity,
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

/// Compute a spiral position — delegates to the shared helper
/// in [`super::spiral_position`] to avoid duplication.
fn spiral_position(t: f32, index: usize) -> bevy::math::Vec3 {
    super::spiral_position(t, index)
}

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
