//! Camera systems for input, interpolation, shake, and focus tracking.
//!
//! Each system is a standalone Bevy system function designed for low coupling.
//! Chain them in Update order via [`super::plugin::CameraPlugin`].

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::input::ButtonInput;
use bevy::log::{debug, trace};
use bevy::math::Vec3;
use bevy::prelude::{MouseButton, Query, Res, Time, Transform};

use super::components::CinematicCamera;
use crate::constants::camera::{
    INTERPOLATION_SPEED, MAX_PITCH, MAX_ZOOM, MIN_PITCH, MIN_ZOOM, ORBIT_RADIUS, PITCH_SENSITIVITY,
    SHAKE_DECAY, YAW_SENSITIVITY, ZOOM_SENSITIVITY,
};

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Reads accumulated mouse motion (right-button held) to update `target_yaw` / `target_pitch`,
/// and accumulated mouse scroll to update `target_zoom`.
pub fn camera_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut CinematicCamera>,
) {
    let right_held = mouse_buttons.pressed(MouseButton::Right);

    let drag_delta = if right_held {
        accumulated_motion.delta
    } else {
        bevy::math::Vec2::ZERO
    };

    let scroll_y = accumulated_scroll.delta.y;

    if drag_delta == bevy::math::Vec2::ZERO && scroll_y == 0.0 {
        return;
    }

    for mut cam in query.iter_mut() {
        if drag_delta != bevy::math::Vec2::ZERO {
            cam.target_yaw += drag_delta.x * YAW_SENSITIVITY;
            cam.target_pitch =
                (cam.target_pitch - drag_delta.y * PITCH_SENSITIVITY).clamp(MIN_PITCH, MAX_PITCH);
            trace!(
                "camera_input: drag Δ({:.2},{:.2}) → yaw={:.2} pitch={:.2}",
                drag_delta.x,
                drag_delta.y,
                cam.target_yaw,
                cam.target_pitch,
            );
        }

        if scroll_y != 0.0 {
            cam.target_zoom =
                (cam.target_zoom - scroll_y * ZOOM_SENSITIVITY).clamp(MIN_ZOOM, MAX_ZOOM);
            trace!(
                "camera_input: scroll {:.2} → zoom={:.2}",
                scroll_y,
                cam.target_zoom,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Interpolation
// ---------------------------------------------------------------------------

/// Smoothly interpolates current yaw/pitch/zoom toward their targets
/// and updates the camera [`Transform`] accordingly.
pub fn camera_interpolation_system(
    time: Res<Time>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let t = (INTERPOLATION_SPEED * dt).min(1.0);

    for (mut cam, mut tf) in query.iter_mut() {
        cam.yaw_deg = lerp_angle(cam.yaw_deg, cam.target_yaw, t);
        cam.pitch_deg = lerp_angle(cam.pitch_deg, cam.target_pitch, t);
        cam.zoom += (cam.target_zoom - cam.zoom) * t;

        let pos = cam.compute_position(ORBIT_RADIUS);
        let look = cam.compute_look_at();
        *tf = Transform::from_translation(pos).looking_at(look, Vec3::Y);

        trace!(
            "camera_interp: yaw={:.2} pitch={:.2} zoom={:.3} pos={:?}",
            cam.yaw_deg,
            cam.pitch_deg,
            cam.zoom,
            pos,
        );
    }
}

// ---------------------------------------------------------------------------
// Shake
// ---------------------------------------------------------------------------

/// Decays shake amplitude and applies a deterministic pseudo-random offset
/// to the camera [`Transform`] while shake is active.
pub fn camera_shake_system(
    time: Res<Time>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (mut cam, mut tf) in query.iter_mut() {
        if cam.shake < 0.001 {
            cam.shake = 0.0;
            continue;
        }

        // Exponential decay.
        cam.shake *= (-SHAKE_DECAY * dt).exp();

        // Deterministic pseudo-random offset using sin at incommensurate
        // frequencies — reproducible for testing, no RNG required.
        let ox = cam.shake * (elapsed * 37.0).sin();
        let oy = cam.shake * (elapsed * 53.0).sin();
        let oz = cam.shake * (elapsed * 71.0).sin();

        tf.translation += Vec3::new(ox, oy, oz);

        debug!(
            "camera_shake: amp={:.4} offset=({:.4},{:.4},{:.4})",
            cam.shake, ox, oy, oz,
        );
    }
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

/// When a focus target is set, computes the required yaw/pitch to face it
/// and drives the camera toward that orientation.  Clears the target once
/// the camera is within 0.5 degrees.
pub fn camera_focus_system(mut query: Query<&mut CinematicCamera>) {
    for mut cam in query.iter_mut() {
        let target_pos = match cam.focus_target {
            Some(pos) => pos,
            None => continue,
        };

        // Vector from origin (orbit center) to focus target — we derive
        // the required yaw/pitch from this direction.
        let dir = target_pos.normalize_or_zero();
        if dir == Vec3::ZERO {
            cam.focus_target = None;
            debug!("camera_focus: target at origin, clearing");
            continue;
        }

        let desired_yaw = dir.x.atan2(dir.z).to_degrees();
        let desired_pitch = dir.y.asin().to_degrees();

        cam.target_yaw = desired_yaw;
        cam.target_pitch = desired_pitch.clamp(
            crate::constants::camera::MIN_PITCH,
            crate::constants::camera::MAX_PITCH,
        );

        let yaw_err = angle_diff(cam.yaw_deg, desired_yaw).abs();
        let pitch_err = (cam.pitch_deg - cam.target_pitch).abs();

        trace!(
            "camera_focus: desired yaw={:.2} pitch={:.2} err=({:.2},{:.2})",
            desired_yaw,
            desired_pitch,
            yaw_err,
            pitch_err,
        );

        if yaw_err < 0.5 && pitch_err < 0.5 {
            cam.focus_target = None;
            debug!("camera_focus: reached target, clearing");
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linearly interpolate between two angles (in degrees), correctly handling
/// the ±180° wrap-around.
pub fn lerp_angle(current: f32, target: f32, t: f32) -> f32 {
    let diff = angle_diff(current, target);
    current + diff * t
}

/// Shortest signed difference from `a` to `b` in degrees, in (-180, 180].
fn angle_diff(a: f32, b: f32) -> f32 {
    let mut d = (b - a) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_angle_no_wrap() {
        let result = lerp_angle(10.0, 20.0, 0.5);
        assert!((result - 15.0).abs() < 1e-4);
    }

    #[test]
    fn lerp_angle_wrap_positive() {
        // 350 → 10 should go forward (+20), not backward (-340).
        let result = lerp_angle(350.0, 10.0, 0.5);
        // Expected: 350 + 0.5 * 20 = 360 → 360.0 (or equivalent)
        assert!((result - 360.0).abs() < 1e-4);
    }

    #[test]
    fn lerp_angle_wrap_negative() {
        // 10 → 350 should go backward (-20).
        let result = lerp_angle(10.0, 350.0, 0.5);
        assert!((result - 0.0).abs() < 1e-4);
    }

    #[test]
    fn angle_diff_basic() {
        assert!((angle_diff(0.0, 90.0) - 90.0).abs() < 1e-4);
        assert!((angle_diff(90.0, 0.0) - (-90.0)).abs() < 1e-4);
    }

    #[test]
    fn angle_diff_wrap() {
        // 350 → 10 = +20 (shortest path)
        let d = angle_diff(350.0, 10.0);
        assert!((d - 20.0).abs() < 1e-4);
    }
}
