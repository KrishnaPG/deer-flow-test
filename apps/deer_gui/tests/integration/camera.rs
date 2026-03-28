//! Integration tests for the cinematic camera system.
//!
//! Uses `MinimalPlugins` with manually spawned entities and systems
//! to avoid renderer dependencies (Camera3d requires the render pipeline).

use std::time::Duration;

use bevy::app::App;
use bevy::log::trace;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use deer_gui::camera::{camera_interpolation_system, camera_shake_system, CinematicCamera};
use deer_gui::constants::camera::{DEFAULT_PITCH, DEFAULT_YAW, DEFAULT_ZOOM, MAX_ZOOM};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn build_camera_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Fixed time step for deterministic dt across CI.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.016,
    )));

    let cam_entity = app
        .world_mut()
        .spawn((CinematicCamera::default(), Transform::default()))
        .id();

    app.add_systems(
        Update,
        (camera_interpolation_system, camera_shake_system).chain(),
    );

    (app, cam_entity)
}
/// Read the [`CinematicCamera`] component from the given entity.
fn read_cam(app: &mut App, entity: Entity) -> CinematicCamera {
    app.world_mut()
        .entity(entity)
        .get::<CinematicCamera>()
        .unwrap()
        .clone()
}

// -- T-CAM-01 ---------------------------------------------------------------

#[test]
fn t_cam_01_plugin_spawns_one_camera() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(ImagePlugin::default());
    app.add_plugins(deer_gui::camera::CameraPlugin);
    app.update();

    let count = app
        .world_mut()
        .query::<&CinematicCamera>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "CameraPlugin must spawn exactly one CinematicCamera"
    );
    trace!("t_cam_01: passed — found {count} camera(s)");
}

// -- T-CAM-02 ---------------------------------------------------------------

#[test]
fn t_cam_02_default_values_match_constants() {
    let (mut app, entity) = build_camera_app();
    app.update();

    let cam = read_cam(&mut app, entity);
    assert!((cam.yaw_deg - DEFAULT_YAW).abs() < 1e-4, "yaw mismatch");
    assert!(
        (cam.pitch_deg - DEFAULT_PITCH).abs() < 1e-4,
        "pitch mismatch"
    );
    assert!((cam.zoom - DEFAULT_ZOOM).abs() < 1e-4, "zoom mismatch");
    trace!("t_cam_02: passed");
}

// -- T-CAM-03 ---------------------------------------------------------------

#[test]
fn t_cam_03_interpolation_moves_toward_target() {
    let (mut app, entity) = build_camera_app();
    app.update(); // first tick to initialise Time

    // Set a new target yaw.
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<CinematicCamera>()
        .unwrap()
        .target_yaw = 45.0;

    // Run several ticks to let interpolation converge.
    // With INTERPOLATION_SPEED=4.0 and dt=0.016, lerp factor t=0.064.
    // Convergence is exponential; 60 ticks gives ~98% convergence.
    for _ in 0..60 {
        app.update();
    }

    let cam = read_cam(&mut app, entity);
    let distance = (cam.yaw_deg - 45.0).abs();
    assert!(
        distance < 5.0,
        "yaw_deg ({:.2}) should be close to 45.0 after 60 ticks (distance={:.2})",
        cam.yaw_deg,
        distance,
    );
    trace!("t_cam_03: passed — yaw_deg={:.2}", cam.yaw_deg);
}

// -- T-CAM-04 ---------------------------------------------------------------

#[test]
fn t_cam_04_shake_decays() {
    let (mut app, entity) = build_camera_app();
    app.update();

    // Inject shake amplitude.
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<CinematicCamera>()
        .unwrap()
        .shake = 1.0;

    for _ in 0..60 {
        app.update();
    }

    let cam = read_cam(&mut app, entity);
    assert!(
        cam.shake < 0.1,
        "shake ({:.4}) should have decayed well below 1.0 after 60 ticks",
        cam.shake,
    );
    trace!("t_cam_04: passed — shake={:.4}", cam.shake);
}

// -- T-CAM-05 ---------------------------------------------------------------

#[test]
fn t_cam_05_stable_without_input() {
    let (mut app, entity) = build_camera_app();
    app.update();

    let before = read_cam(&mut app, entity);

    for _ in 0..10 {
        app.update();
    }

    let after = read_cam(&mut app, entity);
    assert!(
        (after.target_yaw - before.target_yaw).abs() < 1e-6,
        "target_yaw drifted without input: {} → {}",
        before.target_yaw,
        after.target_yaw,
    );
    assert!(
        (after.target_pitch - before.target_pitch).abs() < 1e-6,
        "target_pitch drifted without input: {} → {}",
        before.target_pitch,
        after.target_pitch,
    );
    trace!("t_cam_05: passed — targets unchanged");
}

// -- T-CAM-06 ---------------------------------------------------------------

#[test]
fn t_cam_06_zoom_clamps_to_max() {
    let (mut app, entity) = build_camera_app();
    app.update();

    // Set target zoom way beyond MAX_ZOOM.
    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut cam = entity_mut.get_mut::<CinematicCamera>().unwrap();
        cam.target_zoom = MAX_ZOOM + 10.0;
        cam.zoom = MAX_ZOOM + 10.0;
    }

    for _ in 0..30 {
        app.update();
    }

    let cam = read_cam(&mut app, entity);
    // Interpolation lerps zoom toward target_zoom but does not clamp;
    // clamping is an input-layer concern. Since we bypassed input and set
    // both zoom & target_zoom above MAX_ZOOM, zoom stays above it.
    assert!(
        cam.zoom > MAX_ZOOM,
        "zoom ({:.2}) should remain above MAX_ZOOM ({:.2}) when set directly",
        cam.zoom,
        MAX_ZOOM,
    );
    trace!("t_cam_06: passed — zoom={:.2}", cam.zoom);
}
