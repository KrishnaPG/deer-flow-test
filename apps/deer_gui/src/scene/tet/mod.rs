//! TET scene — the central tetrahedral structure environment.

pub mod particles;
pub mod setup;
pub mod systems;

pub use setup::tet_scene_setup_system;
pub use systems::{data_trail_system, tet_glow_system};

use crate::constants::visual::TET_STRUCTURE_RADIUS;

// ---------------------------------------------------------------------------
// Shared geometry helpers
// ---------------------------------------------------------------------------

/// Compute a spiral position for a data trail at parametric position `t`.
///
/// Shared between `setup` (initial placement) and `systems` (per-frame motion)
/// to keep the spiral formula in one place.
pub(crate) fn spiral_position(t: f32, index: usize) -> bevy::math::Vec3 {
    let phase = index as f32 * 0.618; // golden ratio offset per trail
    let angle = t * std::f32::consts::TAU * 3.0 + phase;
    let r = TET_STRUCTURE_RADIUS * 1.5 + t * TET_STRUCTURE_RADIUS;
    let y = (t - 0.5) * TET_STRUCTURE_RADIUS * 2.0;

    bevy::math::Vec3::new(r * angle.cos(), y, r * angle.sin())
}
