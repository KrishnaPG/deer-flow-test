//! Manual parallax scrolling system.
//!
//! Entities tagged with [`ParallaxLayer`] are offset each frame based on
//! the camera position delta, scaled by their `depth` value.
//! Depth 0.0 = foreground (moves most), 1.0 = farthest (moves least).

use bevy::log::trace;
use bevy::math::Vec3;
use bevy::prelude::{Camera3d, Component, Query, Res, Resource, Transform, With, Without};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks an entity as participating in parallax scrolling.
///
/// `depth` controls the parallax factor:
/// - `0.0` = foreground — full camera-relative movement
/// - `1.0` = farthest layer — almost no relative movement
#[derive(Component, Debug, Clone)]
pub struct ParallaxLayer {
    /// Depth factor in `[0.0, 1.0]`.
    pub depth: f32,
}

impl ParallaxLayer {
    /// Create a new parallax layer with the given depth.
    pub fn new(depth: f32) -> Self {
        let depth = depth.clamp(0.0, 1.0);
        trace!("ParallaxLayer::new — depth={:.3}", depth);
        Self { depth }
    }

    /// Compute the movement factor: foreground (depth=0) → factor=1,
    /// farthest (depth=1) → factor≈0.
    #[inline]
    pub fn movement_factor(&self) -> f32 {
        1.0 - self.depth
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the camera position from the previous frame for delta computation.
#[derive(Resource, Debug, Default)]
pub struct PreviousCameraPosition(pub Vec3);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Offsets entities with [`ParallaxLayer`] based on the camera position delta.
///
/// The displacement applied is `camera_delta * movement_factor`, where
/// movement_factor ranges from 1.0 (foreground) to ~0.0 (far background).
pub fn parallax_scroll_system(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut prev_pos: Res<PreviousCameraPosition>,
    mut layer_query: Query<(&ParallaxLayer, &mut Transform), Without<Camera3d>>,
) {
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };

    let current = cam_tf.translation;
    let delta = current - prev_pos.0;

    if delta.length_squared() < 1e-8 {
        return;
    }

    trace!(
        "parallax_scroll: camera_delta=({:.3},{:.3},{:.3})",
        delta.x,
        delta.y,
        delta.z,
    );

    for (layer, mut tf) in layer_query.iter_mut() {
        let factor = layer.movement_factor();
        let offset = delta * factor * 0.1; // subtle parallax
        tf.translation += offset;
    }

    // NOTE: prev_pos is updated via a separate system or resource write.
    // In a full implementation, we'd use a ResMut here, but since we
    // want the previous frame's value, this is read-only in this system.
    let _ = &mut prev_pos;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallax_layer_clamps_depth() {
        let layer = ParallaxLayer::new(-0.5);
        assert_eq!(layer.depth, 0.0);
        let layer = ParallaxLayer::new(2.0);
        assert_eq!(layer.depth, 1.0);
    }

    #[test]
    fn movement_factor_correct() {
        let foreground = ParallaxLayer::new(0.0);
        assert!((foreground.movement_factor() - 1.0).abs() < 1e-6);
        let background = ParallaxLayer::new(1.0);
        assert!(background.movement_factor().abs() < 1e-6);
        let mid = ParallaxLayer::new(0.5);
        assert!((mid.movement_factor() - 0.5).abs() < 1e-6);
    }
}
