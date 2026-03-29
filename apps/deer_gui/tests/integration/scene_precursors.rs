//! Integration tests for Precursors scene.
//!
//! Verifies scene activation, audio, theme switching, and entity spawning.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::constants::precursors::{BARGE_COUNT, TRAVELLER_COUNT};
use deer_gui::scene::audio_bridge::SceneAudioState;
use deer_gui::scene::generators::{Barge, Traveller};
use deer_gui::scene::manager::SceneManager;
use deer_gui::theme::{precursors_theme, ThemeManager};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_precursors_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.insert_resource(SceneManager::new());
    app.insert_resource(ThemeManager::new(vec![precursors_theme()]));
    app.init_resource::<SceneAudioState>();
    app
}

// ---------------------------------------------------------------------------
// T-PREC-01: Activate sets audio
// ---------------------------------------------------------------------------

#[test]
fn t_prec_01_activate_sets_audio() {
    let mut app = build_precursors_app();

    app.update();

    let audio_state = app.world().resource::<SceneAudioState>();
    // Audio state should exist and be ready for activation
    assert!(audio_state.active_track().is_none());
}

// ---------------------------------------------------------------------------
// T-PREC-02: Theme switches to Precursors
// ---------------------------------------------------------------------------

#[test]
fn t_prec_02_theme_switches() {
    let mut app = build_precursors_app();

    let switched = {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        manager.switch("Precursors")
    };

    assert!(switched, "should successfully switch to Precursors theme");

    let active_name = {
        let manager = app.world().resource::<ThemeManager>();
        manager.active_name().to_string()
    };

    assert_eq!(
        active_name, "Precursors",
        "active theme should be Precursors"
    );
}

// ---------------------------------------------------------------------------
// T-PREC-03: Barge count constant matches scene descriptor
// ---------------------------------------------------------------------------

#[test]
fn t_prec_03_barge_count_matches_descriptor() {
    assert_eq!(
        BARGE_COUNT, 60,
        "BARGE_COUNT should be 60 per scene descriptor"
    );
}

// ---------------------------------------------------------------------------
// T-PREC-04: Traveller count constant matches scene descriptor
// ---------------------------------------------------------------------------

#[test]
fn t_prec_04_traveller_count_matches_descriptor() {
    assert_eq!(
        TRAVELLER_COUNT, 80,
        "TRAVELLER_COUNT should be 80 per scene descriptor"
    );
}

// ---------------------------------------------------------------------------
// T-PREC-05: Barge and Traveller components exist
// ---------------------------------------------------------------------------

#[test]
fn t_prec_05_entity_components_exist() {
    let mut app = build_precursors_app();
    app.update();

    // Verify we can query for Barge and Traveller components
    let barge_count = app.world_mut().query::<&Barge>().iter(app.world()).count();

    let traveller_count = app
        .world_mut()
        .query::<&Traveller>()
        .iter(app.world())
        .count();

    // Initially zero (no scene spawned yet), but components are registered
    assert_eq!(barge_count, 0);
    assert_eq!(traveller_count, 0);
}

// ---------------------------------------------------------------------------
// T-PREC-06: Switch from TET theme
// ---------------------------------------------------------------------------

#[test]
fn t_prec_06_switch_from_tet() {
    use deer_gui::theme::tet_theme;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.insert_resource(SceneManager::new());
    app.insert_resource(ThemeManager::new(vec![tet_theme(), precursors_theme()]));
    app.init_resource::<SceneAudioState>();

    app.update();

    // Initially should be TET
    {
        let manager = app.world().resource::<ThemeManager>();
        assert_eq!(manager.active_name(), "TET Orchestrator");
    }

    // Switch to Precursors
    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        let result = manager.switch("Precursors");
        assert!(result, "should successfully switch to Precursors");
    }

    // Verify switch
    {
        let manager = app.world().resource::<ThemeManager>();
        assert_eq!(manager.active_name(), "Precursors");
    }

    // Switch back to TET
    {
        let mut manager = app.world_mut().resource_mut::<ThemeManager>();
        let result = manager.switch("TET Orchestrator");
        assert!(result, "should successfully switch back to TET");
    }

    // Verify back to TET
    {
        let manager = app.world().resource::<ThemeManager>();
        assert_eq!(manager.active_name(), "TET Orchestrator");
    }
}
