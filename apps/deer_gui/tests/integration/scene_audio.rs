//! Integration tests for the scene-audio bridge.
//!
//! Verifies [`SceneAudioState`] resource behavior and that the
//! [`scene_audio_bridge_system`] correctly transitions state when the
//! desired ambient track changes.
//!
//! The bridge system is tested in isolation (without `DeerAudioPlugin`)
//! so that `AudioCommand` messages are written but not consumed — this
//! avoids the need for a rendering backend or registered `AudioSource`.

use bevy::app::App;
use bevy::log::trace;
use bevy::prelude::*;

use deer_gui::audio::AudioCommand;
use deer_gui::scene::{scene_audio_bridge_system, SceneAudioState};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Build a minimal app with the bridge system and `AudioCommand` messages.
///
/// Registers `AudioCommand` as a message type directly (without
/// `DeerAudioPlugin`) so the bridge can write messages without a
/// consumer that requires asset loading.
fn build_audio_bridge_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<AudioCommand>();
    app.init_resource::<SceneAudioState>();
    app.add_systems(Update, scene_audio_bridge_system);
    app
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-01: Default state has no track
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_01_state_default_no_track() {
    let state = SceneAudioState::default();

    assert!(
        state.desired_track().is_none(),
        "default desired_track must be None"
    );
    assert!(
        state.active_track().is_none(),
        "default active_track must be None"
    );
    assert!(
        !state.has_pending_change(),
        "no pending change when both tracks are None"
    );

    trace!("t_scene_audio_01: passed — default state verified");
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-02: request_ambient sets desired track
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_02_request_ambient_sets_desired() {
    let mut state = SceneAudioState::default();
    state.request_ambient("sounds/ambient_tet.ogg");

    assert_eq!(
        state.desired_track(),
        Some("sounds/ambient_tet.ogg"),
        "desired_track must match requested track"
    );
    assert!(
        state.has_pending_change(),
        "must have pending change after request_ambient"
    );

    trace!("t_scene_audio_02: passed — request_ambient sets desired");
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-03: Bridge sends PlayAmbient on initial track set
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_03_bridge_sends_play_on_change() {
    let mut app = build_audio_bridge_app();
    app.update(); // initial tick

    // Set desired track.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_ambient("sounds/ambient_tet.ogg");

    // Run the bridge system.
    app.update();

    // Verify state was applied.
    let state = app.world_mut().resource::<SceneAudioState>();
    assert!(
        !state.has_pending_change(),
        "pending change must be cleared after bridge runs"
    );
    assert_eq!(
        state.active_track(),
        Some("sounds/ambient_tet.ogg"),
        "active_track must match desired after bridge runs"
    );

    trace!("t_scene_audio_03: passed — bridge processes play on change");
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-04: Switching tracks stops then plays
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_04_bridge_sends_stop_then_play_on_switch() {
    let mut app = build_audio_bridge_app();
    app.update();

    // Set initial track and apply.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_ambient("sounds/ambient_tet.ogg");
    app.update();

    // Switch to a different track.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_ambient("sounds/ambient_precursors.ogg");
    app.update();

    let state = app.world_mut().resource::<SceneAudioState>();
    assert_eq!(
        state.active_track(),
        Some("sounds/ambient_precursors.ogg"),
        "active_track must be the new track"
    );
    assert!(
        !state.has_pending_change(),
        "no pending change after switch"
    );

    trace!("t_scene_audio_04: passed — stop + play on track switch");
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-05: No-op when no change
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_05_bridge_noop_when_no_change() {
    let mut app = build_audio_bridge_app();
    app.update();

    // Set and apply a track.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_ambient("sounds/ambient_tet.ogg");
    app.update();

    // Run again — no change should happen.
    app.update();
    app.update();

    let state = app.world_mut().resource::<SceneAudioState>();
    assert_eq!(
        state.active_track(),
        Some("sounds/ambient_tet.ogg"),
        "active_track must remain unchanged"
    );
    assert!(
        !state.has_pending_change(),
        "still no pending change after redundant updates"
    );

    trace!("t_scene_audio_05: passed — no-op on repeated updates");
}

// ---------------------------------------------------------------------------
// T-SCENE-AUDIO-06: request_stop clears active track
// ---------------------------------------------------------------------------

#[test]
fn t_scene_audio_06_request_stop_sends_stop() {
    let mut app = build_audio_bridge_app();
    app.update();

    // Set and apply a track.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_ambient("sounds/ambient_tet.ogg");
    app.update();

    // Request stop.
    app.world_mut()
        .resource_mut::<SceneAudioState>()
        .request_stop();
    app.update();

    let state = app.world_mut().resource::<SceneAudioState>();
    assert!(
        state.active_track().is_none(),
        "active_track must be None after stop"
    );
    assert!(
        state.desired_track().is_none(),
        "desired_track must be None after stop"
    );
    assert!(
        !state.has_pending_change(),
        "no pending change after stop applied"
    );

    trace!("t_scene_audio_06: passed — stop clears ambient");
}
