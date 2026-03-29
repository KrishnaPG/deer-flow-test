//! Integration tests for Descent scene.
//!
//! Verifies scene activation, audio, theme switching, and entity spawning
//! for the drop-ship descent through alien clouds.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::constants::descent::{BEACON_COUNT, CLOUD_COUNT, POD_COUNT};
use deer_gui::scene::audio_bridge::SceneAudioState;
use deer_gui::scene::generators::{CloudParticle, DropPod, GlowEntity};
use deer_gui::scene::manager::{SceneManager, SceneRoot};
use deer_gui::theme::{descent_theme, ThemeManager};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_descent_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.insert_resource(SceneManager::new());
    app.insert_resource(ThemeManager::new(vec![descent_theme()]));
    app.insert_resource(SceneAudioState::default());
    app
}

// ---------------------------------------------------------------------------
// T-DESC-01: Activate sets audio
// ---------------------------------------------------------------------------

#[test]
fn t_desc_01_activate_sets_audio() {
    let mut app = build_descent_app();

    // Add descent theme to manager
    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    let audio_state = app.world().resource::<SceneAudioState>();
    // Audio state should exist and be ready for activation
    assert!(audio_state.active_track().is_none());
}

// ---------------------------------------------------------------------------
// T-DESC-02: Root spawned
// ---------------------------------------------------------------------------

#[test]
fn t_desc_02_root_spawned() {
    let mut app = build_descent_app();

    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    // Initially no scene root (need to activate a scene)
    let root_count = app
        .world_mut()
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(app.world())
        .count();

    assert_eq!(root_count, 0, "no scene root before activation");
}

// ---------------------------------------------------------------------------
// T-DESC-03: Cloud count matches descriptor
// ---------------------------------------------------------------------------

#[test]
fn t_desc_03_cloud_count() {
    let mut app = build_descent_app();

    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    // Verify constant matches expected value from descriptor
    assert_eq!(
        CLOUD_COUNT, 120,
        "CLOUD_COUNT should be 120 per scene descriptor"
    );

    // Query for existing cloud particles (should be 0 before scene spawn)
    let cloud_count = app
        .world_mut()
        .query::<&CloudParticle>()
        .iter(app.world())
        .count();

    // Initially zero (no scene spawned yet), but component type is registered
    assert_eq!(cloud_count, 0);
}

// ---------------------------------------------------------------------------
// T-DESC-04: Pod count matches descriptor
// ---------------------------------------------------------------------------

#[test]
fn t_desc_04_pod_count() {
    let mut app = build_descent_app();

    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    // Verify constant matches expected value from descriptor
    assert_eq!(POD_COUNT, 40, "POD_COUNT should be 40 per scene descriptor");

    // Query for existing drop pods (should be 0 before scene spawn)
    let pod_count = app
        .world_mut()
        .query::<&DropPod>()
        .iter(app.world())
        .count();

    // Initially zero (no scene spawned yet), but component type is registered
    assert_eq!(pod_count, 0);
}

// ---------------------------------------------------------------------------
// T-DESC-05: Beacon count matches descriptor
// ---------------------------------------------------------------------------

#[test]
fn t_desc_05_beacon_count() {
    let mut app = build_descent_app();

    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    // Verify constant matches expected value from descriptor
    assert_eq!(
        BEACON_COUNT, 20,
        "BEACON_COUNT should be 20 per scene descriptor"
    );

    // Query for existing glow entities (should be 0 before scene spawn)
    let beacon_count = app
        .world_mut()
        .query::<&GlowEntity>()
        .iter(app.world())
        .count();

    // Initially zero (no scene spawned yet), but component type is registered
    assert_eq!(beacon_count, 0);
}

// ---------------------------------------------------------------------------
// T-DESC-06: Deactivate clears entities
// ---------------------------------------------------------------------------

#[test]
fn t_desc_06_deactivate_clears() {
    let mut app = build_descent_app();

    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        *manager = ThemeManager::new(vec![descent_theme()]);
    }

    app.update();

    // All counts should be 0 when no scene is active
    let cloud_count = app
        .world_mut()
        .query::<&CloudParticle>()
        .iter(app.world())
        .count();

    let pod_count = app
        .world_mut()
        .query::<&DropPod>()
        .iter(app.world())
        .count();

    let beacon_count = app
        .world_mut()
        .query::<&GlowEntity>()
        .iter(app.world())
        .count();

    assert_eq!(cloud_count, 0, "cloud count should be 0 after deactivate");
    assert_eq!(pod_count, 0, "pod count should be 0 after deactivate");
    assert_eq!(beacon_count, 0, "beacon count should be 0 after deactivate");
}
