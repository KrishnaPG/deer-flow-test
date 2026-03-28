//! Integration tests for the audio subsystem.
//!
//! Uses `MinimalPlugins` + `AssetPlugin` to satisfy `DeerAudioPlugin`
//! system parameters without requiring a full renderer.

use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::log::trace;
use bevy::prelude::*;

use deer_gui::audio::{AudioManager, DeerAudioPlugin};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Build an app with the audio plugin ready to go.
fn build_audio_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), DeerAudioPlugin));
    app
}

// ---------------------------------------------------------------------------
// T-AUDIO-01: DeerAudioPlugin registers AudioManager resource
// ---------------------------------------------------------------------------

#[test]
fn t_audio_01_plugin_registers_audio_manager() {
    let mut app = build_audio_app();
    app.update();

    let manager = app.world_mut().get_resource::<AudioManager>();
    assert!(
        manager.is_some(),
        "AudioManager resource must be present after DeerAudioPlugin is added"
    );
    trace!("t_audio_01: passed — AudioManager registered");
}

// ---------------------------------------------------------------------------
// T-AUDIO-02: AudioManager initializes with volume 1.0 and not muted
// ---------------------------------------------------------------------------

#[test]
fn t_audio_02_default_values() {
    let mut app = build_audio_app();
    app.update();

    let manager = app.world_mut().resource::<AudioManager>();
    assert_eq!(
        manager.master_volume, 1.0,
        "default master_volume must be 1.0"
    );
    assert!(!manager.muted, "default muted must be false");
    assert!(manager.current_ambient.is_none(), "no ambient by default");
    assert!(
        manager.ambient_entity.is_none(),
        "no ambient entity by default"
    );
    trace!("t_audio_02: passed");
}

// ---------------------------------------------------------------------------
// T-AUDIO-03: Mute sets effective volume to zero
// ---------------------------------------------------------------------------

#[test]
fn t_audio_03_mute_sets_effective_volume_zero() {
    let mut app = build_audio_app();
    app.update();

    let mut manager = app.world_mut().resource_mut::<AudioManager>();
    manager.mute();
    assert!(manager.muted, "muted flag must be true after mute()");
    assert_eq!(
        manager.effective_volume(),
        0.0,
        "effective volume must be 0.0 when muted"
    );
    trace!("t_audio_03: passed");
}

// ---------------------------------------------------------------------------
// T-AUDIO-04: Unmute restores volume
// ---------------------------------------------------------------------------

#[test]
fn t_audio_04_unmute_restores_volume() {
    let mut app = build_audio_app();
    app.update();

    let mut manager = app.world_mut().resource_mut::<AudioManager>();
    manager.mute();
    manager.unmute();
    assert!(!manager.muted, "muted flag must be false after unmute()");
    assert_eq!(
        manager.effective_volume(),
        1.0,
        "effective volume must restore to 1.0"
    );
    trace!("t_audio_04: passed");
}

// ---------------------------------------------------------------------------
// T-AUDIO-05: SetMasterVolume updates and clamps
// ---------------------------------------------------------------------------

#[test]
fn t_audio_05_set_volume_clamps() {
    let mut app = build_audio_app();
    app.update();

    let mut manager = app.world_mut().resource_mut::<AudioManager>();

    manager.set_volume(0.5);
    assert_eq!(manager.master_volume, 0.5, "volume should be 0.5");

    manager.set_volume(1.5);
    assert_eq!(
        manager.master_volume, 1.0,
        "volume above 1.0 must clamp to 1.0"
    );

    manager.set_volume(-0.5);
    assert_eq!(
        manager.master_volume, 0.0,
        "volume below 0.0 must clamp to 0.0"
    );

    trace!("t_audio_05: passed");
}

// ---------------------------------------------------------------------------
// T-AUDIO-06: audio_command_system runs without panic with no messages
// ---------------------------------------------------------------------------

#[test]
fn t_audio_06_system_runs_without_panic() {
    let mut app = build_audio_app();
    // Three ticks with zero messages should not panic.
    app.update();
    app.update();
    app.update();
    trace!("t_audio_06: passed — 3 updates with no messages, no panic");
}
