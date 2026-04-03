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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_viewport_camera_center_is_half() {
        let cam = Vec3::new(50.0, 0.0, 30.0);
        let result = world_to_viewport(cam, cam, 1.0);
        assert!((result.x - 0.5).abs() < 1e-4);
        assert!((result.y - 0.5).abs() < 1e-4);
    }

    #[test]
    fn world_to_viewport_offset_at_zoom_1() {
        let cam = Vec3::ZERO;
        let world = Vec3::new(50.0, 0.0, 0.0);
        let result = world_to_viewport(world, cam, 1.0);
        // offset.x = 50, scale = 1/100 = 0.01, so 50*0.01 + 0.5 = 1.0
        assert!((result.x - 1.0).abs() < 1e-4);
        assert!((result.y - 0.5).abs() < 1e-4);
    }

    #[test]
    fn world_to_viewport_zoom_halves_scale() {
        let cam = Vec3::ZERO;
        let world = Vec3::new(50.0, 0.0, 0.0);
        let result = world_to_viewport(world, cam, 2.0);
        // scale = 1/(2*100) = 0.005, so 50*0.005 + 0.5 = 0.75
        assert!((result.x - 0.75).abs() < 1e-4);
    }

    #[test]
    fn compute_frustum_bounds_zoom_1_square() {
        let (center, size) = compute_frustum_bounds(1.0, 1.0);
        assert_eq!(center, Vec2::splat(0.5));
        assert_eq!(size, Vec2::ONE);
    }

    #[test]
    fn compute_frustum_bounds_zoom_2_halves_size() {
        let (center, size) = compute_frustum_bounds(2.0, 1.0);
        assert_eq!(center, Vec2::splat(0.5));
        assert_eq!(size, Vec2::splat(0.5));
    }

    #[test]
    fn compute_frustum_bounds_wide_aspect() {
        let (center, size) = compute_frustum_bounds(1.0, 16.0 / 9.0);
        assert_eq!(center, Vec2::splat(0.5));
        assert!((size.x - 1.0).abs() < 1e-4);
        assert!((size.y - 9.0 / 16.0).abs() < 1e-4);
    }
}
