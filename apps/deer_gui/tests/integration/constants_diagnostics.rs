//! Integration tests for `constants::camera` and `diagnostics::DiagnosticsPlugin`.
//!
//! Validates constant invariants, camera defaults, and diagnostic registration
//! using headless Bevy apps (MinimalPlugins — no window or renderer).

use bevy::app::App;
use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;

use deer_gui::camera::CameraPlugin;
use deer_gui::constants::camera::{
    DEFAULT_PITCH, DEFAULT_ZOOM, MAX_PITCH, MAX_ZOOM, MIN_PITCH, MIN_ZOOM,
};
use deer_gui::diagnostics::DiagnosticsPlugin;

// ---------------------------------------------------------------------------
// T-CONST-01: camera constant ordering invariants
// ---------------------------------------------------------------------------

/// **T-CONST-01** — Verify that camera constants satisfy:
///   MIN_ZOOM < DEFAULT_ZOOM < MAX_ZOOM
///   MIN_PITCH < DEFAULT_PITCH < MAX_PITCH
///
/// Also confirms that an App with MinimalPlugins + CameraPlugin starts
/// without panic.
#[test]
fn t_const_01_camera_constant_ordering() {
    // Static invariants — these hold regardless of any App.
    assert!(
        MIN_ZOOM < DEFAULT_ZOOM,
        "MIN_ZOOM ({MIN_ZOOM}) must be < DEFAULT_ZOOM ({DEFAULT_ZOOM})"
    );
    assert!(
        DEFAULT_ZOOM < MAX_ZOOM,
        "DEFAULT_ZOOM ({DEFAULT_ZOOM}) must be < MAX_ZOOM ({MAX_ZOOM})"
    );
    assert!(
        MIN_PITCH < DEFAULT_PITCH,
        "MIN_PITCH ({MIN_PITCH}) must be < DEFAULT_PITCH ({DEFAULT_PITCH})"
    );
    assert!(
        DEFAULT_PITCH < MAX_PITCH,
        "DEFAULT_PITCH ({DEFAULT_PITCH}) must be < MAX_PITCH ({MAX_PITCH})"
    );

    // Smoke-test: build the app and tick once to prove it doesn't panic.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CameraPlugin);
    app.update();
}

// ---------------------------------------------------------------------------
// T-CONST-02: spawned camera zoom within bounds
// ---------------------------------------------------------------------------

/// **T-CONST-02** — After CameraPlugin runs its Startup systems the
/// spawned camera's default zoom must lie within [MIN_ZOOM, MAX_ZOOM].
#[test]
fn t_const_02_camera_zoom_in_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CameraPlugin);
    app.update(); // runs Startup schedule

    let mut found = false;
    for cam in app
        .world_mut()
        .query::<&deer_gui::camera::CinematicCamera>()
        .iter(app.world())
    {
        assert!(
            cam.zoom >= MIN_ZOOM && cam.zoom <= MAX_ZOOM,
            "camera zoom {} outside [{}, {}]",
            cam.zoom,
            MIN_ZOOM,
            MAX_ZOOM,
        );
        found = true;
    }
    assert!(
        found,
        "CameraPlugin must spawn at least one CinematicCamera"
    );
}

// ---------------------------------------------------------------------------
// T-DIAG-01: DiagnosticsPlugin registers frame-time diagnostics
// ---------------------------------------------------------------------------

/// **T-DIAG-01** — DiagnosticsPlugin adds FrameTimeDiagnosticsPlugin and
/// EntityCountDiagnosticsPlugin without panic. Three update ticks confirm
/// the systems run without error.
#[test]
fn t_diag_01_frame_time_diagnostics_registered() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DiagnosticsPlugin);
    app.update();
    app.update();
    app.update();

    let store = app.world().resource::<DiagnosticsStore>();
    assert!(
        store.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME).is_some(),
        "FRAME_TIME diagnostic must be registered"
    );
    assert!(
        store.get(&FrameTimeDiagnosticsPlugin::FPS).is_some(),
        "FPS diagnostic must be registered"
    );
}

// ---------------------------------------------------------------------------
// T-DIAG-02: entity-count diagnostic is registered
// ---------------------------------------------------------------------------

/// **T-DIAG-02** — After DiagnosticsPlugin registration the entity-count
/// diagnostic path must be present in the DiagnosticsStore.
#[test]
fn t_diag_02_entity_count_diagnostic_registered() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DiagnosticsPlugin);
    app.update();

    let store = app.world().resource::<DiagnosticsStore>();
    assert!(
        store
            .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
            .is_some(),
        "ENTITY_COUNT diagnostic must be registered"
    );
}
