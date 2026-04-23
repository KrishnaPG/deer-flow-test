//! Camera component definitions.
//!
//! The [`CinematicCamera`] component stores orbital state (yaw, pitch, zoom),
//! smooth-follow targets, shake amplitude, and an optional focus point.
//!
//! Supports multiple camera modes: FirstPerson, ThirdPerson, Orbital, Cinematic.

use bevy::log::trace;
use bevy::math::Vec3;
use bevy::prelude::Component;

use crate::constants::camera::{DEFAULT_PITCH, DEFAULT_YAW, DEFAULT_ZOOM};

// ---------------------------------------------------------------------------
// CameraMode
// ---------------------------------------------------------------------------

/// Camera mode enumeration for hybrid FPS/RTS camera system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CameraMode {
    /// First-person exploration with WASD movement and mouse look.
    FirstPerson,
    /// Third-person camera that follows a target entity.
    ThirdPerson,
    /// Classic RTS-style orbital camera (default).
    #[default]
    Orbital,
    /// Waypoint-based cinematic sequences.
    Cinematic,
}

/// Configuration for FirstPerson camera mode.
#[derive(Debug, Clone)]
pub struct FirstPersonConfig {
    /// Eye height above ground (meters).
    pub eye_height: f32,
    /// Field of view in degrees.
    pub fov: f32,
    /// Mouse sensitivity multiplier.
    pub mouse_sensitivity: f32,
    /// Movement speed (meters/second).
    pub move_speed: f32,
    /// Whether to enable head bob animation.
    pub head_bob: bool,
}

impl Default for FirstPersonConfig {
    fn default() -> Self {
        Self {
            eye_height: 1.7,
            fov: 75.0,
            mouse_sensitivity: 0.3,
            move_speed: 5.0,
            head_bob: false,
        }
    }
}

/// Configuration for ThirdPerson camera mode.
#[derive(Debug, Clone)]
pub struct ThirdPersonConfig {
    /// Camera distance behind target (meters).
    pub distance: f32,
    /// Height offset above target (meters).
    pub height_offset: f32,
    /// Minimum pitch angle (degrees).
    pub min_pitch: f32,
    /// Maximum pitch angle (degrees).
    pub max_pitch: f32,
}

impl Default for ThirdPersonConfig {
    fn default() -> Self {
        Self {
            distance: 3.0,
            height_offset: 1.5,
            min_pitch: -30.0,
            max_pitch: 60.0,
        }
    }
}

/// Camera keyframe for cinematic sequences.
#[derive(Debug, Clone)]
pub struct CameraKeyframe {
    /// Position in world space.
    pub position: Vec3,
    /// Look-at target point.
    pub look_at: Vec3,
    /// Duration to hold this keyframe (seconds).
    pub duration: f32,
}

// ---------------------------------------------------------------------------
// CinematicCamera
// ---------------------------------------------------------------------------

/// Orbital camera state attached to the main camera entity.
///
/// Current values (`yaw_deg`, `pitch_deg`, `zoom`) are interpolated each
/// frame toward their `target_*` counterparts, producing smooth motion.
///
/// Supports multiple camera modes via [`CameraMode`].
#[derive(Component, Debug, Clone)]
pub struct CinematicCamera {
    /// Current camera mode.
    pub mode: CameraMode,
    /// Target camera mode (for smooth transitions).
    pub target_mode: CameraMode,
    /// Transition progress (0.0 = at source, 1.0 = at target).
    pub mode_transition: f32,

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

    /// FirstPerson mode configuration.
    pub first_person: FirstPersonConfig,
    /// ThirdPerson mode configuration.
    pub third_person: ThirdPersonConfig,

    /// Cinematic waypoints (if in cinematic mode).
    pub cinematic_waypoints: Vec<CameraKeyframe>,
    /// Current waypoint index in cinematic sequence.
    pub cinematic_waypoint_index: usize,
    /// Time spent on current waypoint (seconds).
    pub cinematic_waypoint_time: f32,
}

impl Default for CinematicCamera {
    fn default() -> Self {
        trace!("CinematicCamera::default — creating with constants");
        Self {
            mode: CameraMode::default(),
            target_mode: CameraMode::default(),
            mode_transition: 1.0,
            yaw_deg: DEFAULT_YAW,
            target_yaw: DEFAULT_YAW,
            pitch_deg: DEFAULT_PITCH,
            target_pitch: DEFAULT_PITCH,
            zoom: DEFAULT_ZOOM,
            target_zoom: DEFAULT_ZOOM,
            shake: 0.0,
            focus_target: None,
            first_person: FirstPersonConfig::default(),
            third_person: ThirdPersonConfig::default(),
            cinematic_waypoints: Vec::new(),
            cinematic_waypoint_index: 0,
            cinematic_waypoint_time: 0.0,
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
    /// Defaults to a point above water (Y=10m) to avoid looking into the lake.
    pub fn compute_look_at(&self) -> Vec3 {
        let target = self.focus_target.unwrap_or(Vec3::new(0.0, 10.0, 0.0));
        trace!("compute_look_at: {:?}", target);
        target
    }

    /// Add impulse shake amplitude (cumulative).
    pub fn add_shake(&mut self, amplitude: f32) {
        self.shake += amplitude;
        trace!("add_shake: +{:.4} → total {:.4}", amplitude, self.shake);
    }

    /// Start a transition to a new camera mode.
    ///
    /// The transition takes `duration` seconds (default: 0.4s).
    pub fn transition_to_mode(&mut self, new_mode: CameraMode) {
        if new_mode != self.target_mode {
            trace!("transition_to_mode: {:?} → {:?}", self.mode, new_mode);
            self.target_mode = new_mode;
            self.mode_transition = 0.0;
        }
    }

    /// Update the mode transition progress.
    ///
    /// Returns true if the transition is complete.
    pub fn update_mode_transition(&mut self, delta_secs: f32, transition_duration: f32) -> bool {
        if self.mode_transition >= 1.0 {
            return true;
        }

        self.mode_transition = (self.mode_transition + delta_secs / transition_duration).min(1.0);

        // Smooth easing (ease-in-out)
        let t = self.mode_transition;
        let eased = if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        };

        // When transition reaches 50%, switch modes
        if eased >= 0.5 && self.mode != self.target_mode {
            self.mode = self.target_mode;
            trace!("mode_transition: switched to {:?}", self.mode);
        }

        self.mode_transition >= 1.0
    }

    /// Check if currently transitioning between modes.
    pub fn is_transitioning(&self) -> bool {
        self.mode_transition < 1.0
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
        assert_eq!(cam.mode, CameraMode::Orbital);
        assert_eq!(cam.target_mode, CameraMode::Orbital);
        assert!(cam.mode_transition >= 1.0);
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

    #[test]
    fn mode_transition_starts_correctly() {
        let mut cam = CinematicCamera::default();
        cam.transition_to_mode(CameraMode::FirstPerson);
        assert_eq!(cam.target_mode, CameraMode::FirstPerson);
        assert!(cam.mode_transition < 1.0);
        assert!(cam.is_transitioning()); // Should report transitioning
    }

    #[test]
    fn mode_transition_completes() {
        let mut cam = CinematicCamera::default();
        cam.transition_to_mode(CameraMode::ThirdPerson);

        // Simulate full transition
        let complete = cam.update_mode_transition(0.5, 0.4);
        assert!(complete);
        assert_eq!(cam.mode, CameraMode::ThirdPerson);
        assert_eq!(cam.target_mode, CameraMode::ThirdPerson);
    }

    #[test]
    fn camera_mode_defaults() {
        assert_eq!(CameraMode::default(), CameraMode::Orbital);
    }

    #[test]
    fn first_person_config_defaults() {
        let config = FirstPersonConfig::default();
        assert_eq!(config.eye_height, 1.7);
        assert_eq!(config.fov, 75.0);
        assert_eq!(config.move_speed, 5.0);
    }

    #[test]
    fn third_person_config_defaults() {
        let config = ThirdPersonConfig::default();
        assert_eq!(config.distance, 3.0);
        assert_eq!(config.height_offset, 1.5);
    }
}
