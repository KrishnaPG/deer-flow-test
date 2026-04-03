//! Camera synchronization and viewport navigation types.
//!
//! Provides data structures for synchronizing camera state across
//! viewport/minimap panels and issuing navigation requests.

use bevy::prelude::*;

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

/// A request to navigate the viewport to a new center point.
///
/// Emitted by minimap interaction and consumed by the camera system.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ViewportNavigationRequest {
    /// Target center in normalized coordinates (0..1 range).
    pub target_center: Vec2,
}
