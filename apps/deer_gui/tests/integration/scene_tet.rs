//! Integration tests for TET scene entity spawning.
//!
//! Verifies that the TET startup system spawns the expected number
//! of star, monolith, and data-trail entities using the constants
//! from `deer_gui::constants::visual`.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::constants::visual::{DATA_TRAIL_COUNT, STARFIELD_COUNT};
use deer_gui::scene::tet::setup::{DataTrail, Star, TetMonolith};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds an app that runs `tet_scene_setup_system` in Startup.
///
/// Uses `AssetPlugin` for the asset server and manually initialises
/// `Assets<Mesh>` and `Assets<StandardMaterial>` since we don't add
/// the full render pipeline.
fn build_tet_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.add_systems(Startup, deer_gui::scene::tet::setup::tet_scene_setup_system);
    app
}

// ---------------------------------------------------------------------------
// T-SCENE-02  Starfield spawns STARFIELD_COUNT entities
// ---------------------------------------------------------------------------

#[test]
fn t_scene_02_starfield_count() {
    let mut app = build_tet_app();
    app.update();

    let star_count = app
        .world_mut()
        .query_filtered::<Entity, With<Star>>()
        .iter(app.world())
        .count();

    assert_eq!(
        star_count, STARFIELD_COUNT,
        "should spawn exactly STARFIELD_COUNT={STARFIELD_COUNT} stars"
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-03  TetMonolith is spawned exactly once
// ---------------------------------------------------------------------------

#[test]
fn t_scene_03_tet_monolith_spawned() {
    let mut app = build_tet_app();
    app.update();

    let monolith_count = app
        .world_mut()
        .query_filtered::<Entity, With<TetMonolith>>()
        .iter(app.world())
        .count();

    assert_eq!(
        monolith_count, 1,
        "should spawn exactly one TetMonolith entity"
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-04  Data trails spawn DATA_TRAIL_COUNT entities
// ---------------------------------------------------------------------------

#[test]
fn t_scene_04_data_trail_count() {
    let mut app = build_tet_app();
    app.update();

    let trail_count = app
        .world_mut()
        .query_filtered::<Entity, With<DataTrail>>()
        .iter(app.world())
        .count();

    assert_eq!(
        trail_count, DATA_TRAIL_COUNT,
        "should spawn exactly DATA_TRAIL_COUNT={DATA_TRAIL_COUNT} data trails"
    );
}
