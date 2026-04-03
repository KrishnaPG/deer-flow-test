//! Viewport projection helpers for the world camera.
//!
//! Provides utilities for projecting world coordinates into viewport
//! space and computing frustum bounds.

use bevy::math::{Vec2, Vec3};

/// Projects a world position into normalized viewport coordinates.
///
/// Returns a `Vec2` in the 0..1 range relative to the camera's view center.
pub fn world_to_viewport(world_pos: Vec3, camera_translation: Vec3, zoom: f32) -> Vec2 {
    let offset = world_pos - camera_translation;
    let scale = 1.0 / (zoom * 100.0);
    Vec2::new(offset.x * scale, offset.z * scale) + Vec2::splat(0.5)
}

/// Computes the viewport frustum bounds from a camera snapshot.
///
/// Returns `(center, size)` in normalized coordinates.
pub fn compute_frustum_bounds(zoom: f32, aspect_ratio: f32) -> (Vec2, Vec2) {
    let base_size = 1.0 / zoom;
    let width = base_size;
    let height = base_size / aspect_ratio;
    (Vec2::splat(0.5), Vec2::new(width, height))
}
