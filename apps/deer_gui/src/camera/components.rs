//! Camera component definitions.
//!
//! The [`CinematicCamera`] component stores orbital state (yaw, pitch, zoom),
//! smooth-follow targets, shake amplitude, and an optional focus point.

use bevy::log::trace;
use bevy::math::Vec3;
use bevy::prelude::Component;

use crate::constants::camera::{DEFAULT_PITCH, DEFAULT_YAW, DEFAULT_ZOOM};

// ---------------------------------------------------------------------------
// CinematicCamera
// ---------------------------------------------------------------------------

/// Orbital camera state attached to the main camera entity.
///
/// Current values (`yaw_deg`, `pitch_deg`, `zoom`) are interpolated each
/// frame toward their `target_*` counterparts, producing smooth motion.
#[derive(Component, Debug, Clone)]
pub struct CinematicCamera {
    /// Current horizontal rotation in degrees.
    pub yaw_deg: f32,
    /// Desired horizontal rotation in degrees.
    pub target_yaw: f32,

    /// Current vertical tilt in degrees (negative = looking down).
    pub pitch_deg: f32,
    /// Desired vertical tilt in degrees.
    pub target_pitch: f32,

    /// Current zoom multiplier (affects orbit distance).
    pub zoom: f32,
    /// Desired zoom multiplier.
    pub target_zoom: f32,

    /// Remaining shake amplitude (decays over time).
    pub shake: f32,

    /// Optional world-space point to smoothly rotate toward.
    pub focus_target: Option<Vec3>,
}

impl Default for CinematicCamera {
    fn default() -> Self {
        trace!("CinematicCamera::default — creating with constants");
        Self {
            yaw_deg: DEFAULT_YAW,
            target_yaw: DEFAULT_YAW,
            pitch_deg: DEFAULT_PITCH,
            target_pitch: DEFAULT_PITCH,
            zoom: DEFAULT_ZOOM,
            target_zoom: DEFAULT_ZOOM,
            shake: 0.0,
            focus_target: None,
        }
    }
}

impl CinematicCamera {
    /// Convert the current spherical coordinates to a Cartesian position.
    ///
    /// Uses standard spherical-to-Cartesian conversion:
    /// - `x = r * cos(pitch) * sin(yaw)`
    /// - `y = r * sin(pitch)`
    /// - `z = r * cos(pitch) * cos(yaw)`
    ///
    /// The effective radius is `orbit_radius * self.zoom`.
    pub fn compute_position(&self, orbit_radius: f32) -> Vec3 {
        let r = orbit_radius * self.zoom;
        let yaw_rad = self.yaw_deg.to_radians();
        let pitch_rad = self.pitch_deg.to_radians();

        let x = r * pitch_rad.cos() * yaw_rad.sin();
        let y = r * pitch_rad.sin();
        let z = r * pitch_rad.cos() * yaw_rad.cos();

        let pos = Vec3::new(x, y, z);
        trace!(
            "compute_position: yaw={:.2} pitch={:.2} zoom={:.2} r={:.2} → {:?}",
            self.yaw_deg,
            self.pitch_deg,
            self.zoom,
            r,
            pos
        );
        pos
    }

    /// Returns the world-space point the camera should look at.
    ///
    /// Defaults to [`Vec3::ZERO`] when no explicit focus target is set.
    pub fn compute_look_at(&self) -> Vec3 {
        let target = self.focus_target.unwrap_or(Vec3::ZERO);
        trace!("compute_look_at: {:?}", target);
        target
    }

    /// Add impulse shake amplitude (cumulative).
    pub fn add_shake(&mut self, amplitude: f32) {
        self.shake += amplitude;
        trace!("add_shake: +{:.4} → total {:.4}", amplitude, self.shake);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_constants() {
        let cam = CinematicCamera::default();
        assert_eq!(cam.yaw_deg, DEFAULT_YAW);
        assert_eq!(cam.pitch_deg, DEFAULT_PITCH);
        assert_eq!(cam.zoom, DEFAULT_ZOOM);
        assert!(cam.focus_target.is_none());
    }

    #[test]
    fn compute_position_at_zero_angles() {
        let cam = CinematicCamera {
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            zoom: 1.0,
            ..Default::default()
        };
        let pos = cam.compute_position(100.0);
        // yaw=0, pitch=0 → x≈0, y=0, z=100
        assert!(pos.x.abs() < 1e-4);
        assert!(pos.y.abs() < 1e-4);
        assert!((pos.z - 100.0).abs() < 1e-4);
    }

    #[test]
    fn compute_position_respects_zoom() {
        let cam = CinematicCamera {
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            zoom: 2.0,
            ..Default::default()
        };
        let pos = cam.compute_position(100.0);
        assert!((pos.z - 200.0).abs() < 1e-4);
    }

    #[test]
    fn compute_look_at_defaults_to_origin() {
        let cam = CinematicCamera::default();
        assert_eq!(cam.compute_look_at(), Vec3::ZERO);
    }

    #[test]
    fn compute_look_at_returns_focus() {
        let cam = CinematicCamera {
            focus_target: Some(Vec3::new(1.0, 2.0, 3.0)),
            ..Default::default()
        };
        assert_eq!(cam.compute_look_at(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn add_shake_accumulates() {
        let mut cam = CinematicCamera::default();
        cam.add_shake(1.0);
        cam.add_shake(0.5);
        assert!((cam.shake - 1.5).abs() < 1e-6);
    }
}
