//! Camera systems for input, interpolation, shake, and focus tracking.
//!
//! Each system is a standalone Bevy system function designed for low coupling.
//! Chain them in Update order via [`super::plugin::CameraPlugin`].

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::input::ButtonInput;
use bevy::log::{debug, trace};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{MessageReader, MouseButton, Query, Res, Time, Transform};

use super::components::{CameraMode, CinematicCamera};
use super::navigation::ViewportNavigationRequest;
use crate::constants::camera::{
    INTERPOLATION_SPEED, MAX_PITCH, MAX_ZOOM, MIN_PITCH, MIN_ZOOM, ORBIT_RADIUS, PITCH_SENSITIVITY,
    SHAKE_DECAY, YAW_SENSITIVITY, ZOOM_SENSITIVITY,
};

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Reads accumulated mouse motion (right-button held) to update `target_yaw` / `target_pitch`,
/// and accumulated mouse scroll to update `target_zoom`.
///
/// Input resources are optional so the system gracefully no-ops in
/// headless / test environments where `InputPlugin` is absent.
pub fn camera_input_system(
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    accumulated_motion: Option<Res<AccumulatedMouseMotion>>,
    accumulated_scroll: Option<Res<AccumulatedMouseScroll>>,
    mut query: Query<&mut CinematicCamera>,
) {
    let right_held = mouse_buttons
        .as_ref()
        .is_some_and(|b| b.pressed(MouseButton::Right));

    let drag_delta = if right_held {
        accumulated_motion
            .as_ref()
            .map(|m| m.delta)
            .unwrap_or(bevy::math::Vec2::ZERO)
    } else {
        bevy::math::Vec2::ZERO
    };

    let scroll_y = accumulated_scroll
        .as_ref()
        .map(|s| s.delta.y)
        .unwrap_or(0.0);

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
///
/// Behavior varies based on camera mode:
/// - FirstPerson: Yaw/pitch control view direction, position set by movement system
/// - ThirdPerson: Camera orbits around focus_target
/// - Orbital: Classic RTS orbital camera (existing behavior)
/// - Cinematic: Camera follows waypoints
pub fn camera_interpolation_system(
    time: Res<Time>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let t = (INTERPOLATION_SPEED * dt).min(1.0);
    let transition_duration = 0.4; // seconds for mode transition

    for (mut cam, mut tf) in query.iter_mut() {
        // Update mode transition
        cam.update_mode_transition(dt, transition_duration);

        // Interpolate yaw/pitch/zoom
        cam.yaw_deg = lerp_angle(cam.yaw_deg, cam.target_yaw, t);
        cam.pitch_deg = lerp_angle(cam.pitch_deg, cam.target_pitch, t);
        cam.zoom += (cam.target_zoom - cam.zoom) * t;

        // Update transform based on mode
        match cam.mode {
            CameraMode::FirstPerson => {
                // FirstPerson: position is set by movement system, just update rotation
                let yaw_rad = cam.yaw_deg.to_radians();
                let pitch_rad = cam.pitch_deg.to_radians();

                // Create rotation from yaw and pitch
                let yaw_rotation = Quat::from_axis_angle(Vec3::Y, -yaw_rad);
                let pitch_rotation = Quat::from_axis_angle(Vec3::X, -pitch_rad);
                let rotation = yaw_rotation * pitch_rotation;

                tf.rotation = rotation;

                trace!(
                    "camera_interp: FirstPerson yaw={:.2} pitch={:.2} pos={:?}",
                    cam.yaw_deg,
                    cam.pitch_deg,
                    tf.translation,
                );
            }
            CameraMode::ThirdPerson => {
                // ThirdPerson: orbit around focus_target
                let target = cam.focus_target.unwrap_or(Vec3::ZERO);
                let distance = cam.third_person.distance * cam.zoom;
                let height = cam.third_person.height_offset;

                let yaw_rad = cam.yaw_deg.to_radians();
                let pitch_rad = cam.pitch_deg.to_radians();

                // Calculate camera position relative to target
                let offset_x = distance * pitch_rad.cos() * yaw_rad.sin();
                let offset_y = height + distance * pitch_rad.sin();
                let offset_z = distance * pitch_rad.cos() * yaw_rad.cos();

                let pos = target + Vec3::new(offset_x, offset_y, offset_z);
                *tf = Transform::from_translation(pos).looking_at(target, Vec3::Y);

                trace!(
                    "camera_interp: ThirdPerson yaw={:.2} pitch={:.2} dist={:.2} pos={:?}",
                    cam.yaw_deg,
                    cam.pitch_deg,
                    distance,
                    pos,
                );
            }
            CameraMode::Orbital | CameraMode::Cinematic => {
                // Orbital/Cinematic: existing behavior
                let pos = cam.compute_position(ORBIT_RADIUS);
                let look = cam.compute_look_at();
                *tf = Transform::from_translation(pos).looking_at(look, Vec3::Y);

                trace!(
                    "camera_interp: {:?} yaw={:.2} pitch={:.2} zoom={:.3} pos={:?}",
                    cam.mode,
                    cam.yaw_deg,
                    cam.pitch_deg,
                    cam.zoom,
                    pos,
                );
            }
        }
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
// Viewport Navigation
// ---------------------------------------------------------------------------

/// Reads viewport navigation requests and sets the camera focus target.
///
/// Converts normalized viewport coordinates (0..1) to world-space positions
/// using the camera's current translation and zoom level.
///
/// Runs before the focus system, which consumes the focus target.
pub fn viewport_navigation_system(
    requests: Option<MessageReader<ViewportNavigationRequest>>,
    mut query: Query<(&mut CinematicCamera, &Transform)>,
) {
    let Some(mut requests) = requests else { return };
    for request in requests.read() {
        for (mut cam, transform) in &mut query {
            let world_pos =
                unproject_viewport_to_world(request.target_center, transform.translation, cam.zoom);
            cam.focus_target = Some(world_pos);
            trace!(
                "viewport_navigation: target_center={:?} → focus_target={:?}",
                request.target_center,
                cam.focus_target,
            );
        }
    }
}

/// Converts normalized viewport coordinates (0..1) to world-space position.
///
/// The viewport center (0.5, 0.5) maps to the camera's current translation.
/// The scale is determined by the zoom level and orbit radius.
fn unproject_viewport_to_world(normalized: Vec2, camera_translation: Vec3, zoom: f32) -> Vec3 {
    let scale = zoom * 100.0;
    let x = (normalized.x - 0.5) * scale + camera_translation.x;
    let z = (normalized.y - 0.5) * scale + camera_translation.z;
    Vec3::new(x, 0.0, z)
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
// FirstPerson Movement
// ---------------------------------------------------------------------------

/// Handles WASD movement for FirstPerson camera mode.
///
/// Movement direction is calculated based on current camera yaw.
/// Forward (W) moves in the direction the camera is facing (on XZ plane).
pub fn first_person_movement_system(
    time: Res<Time>,
    keyboard_input: Option<Res<ButtonInput<bevy::input::keyboard::KeyCode>>>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
) {
    let Some(keys) = keyboard_input else {
        return;
    };

    let dt = time.delta_secs();

    for (cam, mut transform) in query.iter_mut() {
        if cam.mode != CameraMode::FirstPerson {
            continue;
        }

        let speed = cam.first_person.move_speed * dt;
        let yaw_rad = cam.yaw_deg.to_radians();

        // Forward vector (on XZ plane)
        let forward = Vec3::new(yaw_rad.sin(), 0.0, yaw_rad.cos());
        // Right vector (perpendicular to forward, on XZ plane)
        let right = Vec3::new(yaw_rad.cos(), 0.0, -yaw_rad.sin());

        let mut movement = Vec3::ZERO;

        if keys.pressed(bevy::input::keyboard::KeyCode::KeyW) {
            movement += forward;
        }
        if keys.pressed(bevy::input::keyboard::KeyCode::KeyS) {
            movement -= forward;
        }
        if keys.pressed(bevy::input::keyboard::KeyCode::KeyD) {
            movement += right;
        }
        if keys.pressed(bevy::input::keyboard::KeyCode::KeyA) {
            movement -= right;
        }

        // Vertical movement for flying mode (Space = up, Shift = down)
        if keys.pressed(bevy::input::keyboard::KeyCode::Space) {
            movement.y += 1.0;
        }
        if keys.pressed(bevy::input::keyboard::KeyCode::ShiftLeft)
            || keys.pressed(bevy::input::keyboard::KeyCode::ShiftRight)
        {
            movement.y -= 1.0;
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize() * speed;
            transform.translation += movement;

            trace!(
                "first_person_move: mode={:?} movement={:?} pos={:?}",
                cam.mode,
                movement,
                transform.translation
            );
        }
    }
}

/// Handles mouse look for FirstPerson and ThirdPerson camera modes.
///
/// Always active when in FirstPerson mode, or when left mouse button is held
/// in ThirdPerson mode.
pub fn mouse_look_system(
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    accumulated_motion: Option<Res<AccumulatedMouseMotion>>,
    mut query: Query<&mut CinematicCamera>,
) {
    let left_held = mouse_buttons
        .as_ref()
        .is_some_and(|b| b.pressed(MouseButton::Left) || b.pressed(MouseButton::Right));

    let drag_delta = if left_held {
        accumulated_motion
            .as_ref()
            .map(|m| m.delta)
            .unwrap_or(Vec2::ZERO)
    } else {
        Vec2::ZERO
    };

    if drag_delta == Vec2::ZERO {
        return;
    }

    for mut cam in query.iter_mut() {
        let sensitivity = cam.first_person.mouse_sensitivity;

        match cam.mode {
            CameraMode::FirstPerson => {
                // Full mouse look - rotate camera
                cam.target_yaw -= drag_delta.x * sensitivity;
                cam.target_pitch =
                    (cam.target_pitch - drag_delta.y * sensitivity).clamp(-89.0, 89.0);
                trace!(
                    "mouse_look: FirstPerson drag Δ({:.2},{:.2}) → yaw={:.2} pitch={:.2}",
                    drag_delta.x,
                    drag_delta.y,
                    cam.target_yaw,
                    cam.target_pitch,
                );
            }
            CameraMode::ThirdPerson => {
                // Orbit around target
                cam.target_yaw -= drag_delta.x * YAW_SENSITIVITY;
                cam.target_pitch = (cam.target_pitch - drag_delta.y * PITCH_SENSITIVITY)
                    .clamp(cam.third_person.min_pitch, cam.third_person.max_pitch);
                trace!(
                    "mouse_look: ThirdPerson drag Δ({:.2},{:.2}) → yaw={:.2} pitch={:.2}",
                    drag_delta.x,
                    drag_delta.y,
                    cam.target_yaw,
                    cam.target_pitch,
                );
            }
            _ => {
                // Orbital and Cinematic modes use right-click drag only
                if mouse_buttons
                    .as_ref()
                    .is_some_and(|b| b.pressed(MouseButton::Right))
                {
                    cam.target_yaw += drag_delta.x * YAW_SENSITIVITY;
                    cam.target_pitch = (cam.target_pitch - drag_delta.y * PITCH_SENSITIVITY)
                        .clamp(MIN_PITCH, MAX_PITCH);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mode Toggle (Tab key)
// ---------------------------------------------------------------------------

/// Cycles through camera modes when Tab key is pressed.
///
/// Mode order: Orbital → FirstPerson → ThirdPerson → Orbital
pub fn camera_mode_toggle_system(
    keyboard_input: Option<Res<ButtonInput<bevy::input::keyboard::KeyCode>>>,
    mut query: Query<&mut CinematicCamera>,
) {
    let Some(keys) = keyboard_input else {
        return;
    };

    // Tab key pressed (not just held)
    if !keys.just_pressed(bevy::input::keyboard::KeyCode::Tab) {
        return;
    }

    for mut cam in query.iter_mut() {
        let new_mode = match cam.mode {
            CameraMode::Orbital => CameraMode::FirstPerson,
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
            CameraMode::ThirdPerson => CameraMode::Orbital,
            CameraMode::Cinematic => CameraMode::Orbital, // Exit cinematic to orbital
        };

        cam.transition_to_mode(new_mode);

        // When switching to FirstPerson, preserve current camera position
        // When switching from FirstPerson, set focus_target to current orbital center
        // Note: FirstPerson position is stored in Transform, not in CinematicCamera

        debug!(
            "camera_mode_toggle: {:?} → {:?} (transitioning)",
            cam.mode, new_mode
        );
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
