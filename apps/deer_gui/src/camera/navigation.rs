//! Camera synchronization and viewport navigation types.
//!
//! Provides data structures for synchronizing camera state across
//! viewport/minimap panels and issuing navigation requests.

use bevy::prelude::*;

use super::components::CinematicCamera;

/// A snapshot of the camera's current viewport state.
///
/// Used to synchronize minimap and viewport panels with the main camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSyncSnapshot {
    /// World-space position of the camera.
    pub camera_translation: Vec3,
    /// Current zoom level.
    pub zoom: f32,
    /// Normalized center of the visible frustum (0..1 range).
    pub frustum_center: Vec2,
    /// Normalized size of the visible frustum (0..1 range).
    pub frustum_size: Vec2,
}

impl Default for CameraSyncSnapshot {
    fn default() -> Self {
        Self {
            camera_translation: Vec3::ZERO,
            zoom: 1.0,
            frustum_center: Vec2::splat(0.5),
            frustum_size: Vec2::ONE,
        }
    }
}

/// A request to navigate the viewport to a new center point.
///
/// Emitted by minimap interaction and consumed by the camera system.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ViewportNavigationRequest {
    /// Target center in normalized coordinates (0..1 range).
    pub target_center: Vec2,
}

/// Resource that holds the latest camera sync snapshot.
///
/// Updated each frame by `camera_sync_snapshot_system`.
#[derive(Resource, Default)]
pub struct CameraSyncState {
    pub snapshot: CameraSyncSnapshot,
}

/// System that reads the current camera state and updates the sync snapshot.
///
/// Runs after interpolation so the snapshot reflects the current frame's position.
pub fn camera_sync_snapshot_system(
    mut sync_state: ResMut<CameraSyncState>,
    query: Query<(&CinematicCamera, &Transform)>,
) {
    for (cam, transform) in &query {
        sync_state.snapshot = CameraSyncSnapshot {
            camera_translation: transform.translation,
            zoom: cam.zoom,
            frustum_center: Vec2::splat(0.5),
            frustum_size: Vec2::new(1.0 / cam.zoom, 1.0 / cam.zoom),
        };
    }
}
