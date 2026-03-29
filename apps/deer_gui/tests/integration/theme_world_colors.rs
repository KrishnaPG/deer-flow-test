//! Integration tests for theme world-colour parameterisation.
//!
//! Verifies that the TET theme supplies correct world-material colours
//! and that the scene startup system respects them.

use bevy::asset::AssetPlugin;
use bevy::log::trace;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::tet::setup::{Star, TetMonolith};
use deer_gui::theme::{tet_theme, ThemeManager};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal app with AssetPlugin, ThemeManager, and the TET startup
/// system so that scene entities are spawned using theme colours.
fn build_themed_tet_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.insert_resource(ThemeManager::new(vec![tet_theme()]));
    app.add_systems(Startup, deer_gui::scene::tet::setup::tet_scene_setup_system);
    app
}

// ---------------------------------------------------------------------------
// T-TWC-01  TET theme has non-zero world colours
// ---------------------------------------------------------------------------

#[test]
fn t_twc_01_tet_theme_has_world_colors() {
    let t = tet_theme();

    // star_emissive must be non-zero (bright)
    assert!(
        t.star_emissive.red > 0.0 || t.star_emissive.green > 0.0 || t.star_emissive.blue > 0.0,
        "star_emissive should be non-zero"
    );
    // monolith_emissive must be non-zero
    assert!(
        t.monolith_emissive.red > 0.0
            || t.monolith_emissive.green > 0.0
            || t.monolith_emissive.blue > 0.0,
        "monolith_emissive should be non-zero"
    );
    // trail_emissive must be non-zero
    assert!(
        t.trail_emissive.red > 0.0 || t.trail_emissive.green > 0.0 || t.trail_emissive.blue > 0.0,
        "trail_emissive should be non-zero"
    );

    trace!("t_twc_01: passed — all world colours non-zero");
}

// ---------------------------------------------------------------------------
// T-TWC-02  TET theme world colours match previously hardcoded values
// ---------------------------------------------------------------------------

#[test]
fn t_twc_02_theme_world_colors_match_hardcoded() {
    let t = tet_theme();

    // Stars were hardcoded as (2.0, 2.0, 2.0, 1.0)
    let s = t.star_emissive;
    assert!(
        (s.red - 2.0).abs() < 1e-5 && (s.green - 2.0).abs() < 1e-5 && (s.blue - 2.0).abs() < 1e-5,
        "star_emissive should be (2,2,2) — got ({},{},{})",
        s.red,
        s.green,
        s.blue,
    );

    // Monolith was hardcoded as (0.3, 0.5, 1.0, 1.0)
    let m = t.monolith_emissive;
    assert!(
        (m.red - 0.3).abs() < 1e-5 && (m.green - 0.5).abs() < 1e-5 && (m.blue - 1.0).abs() < 1e-5,
        "monolith_emissive should be (0.3,0.5,1.0) — got ({},{},{})",
        m.red,
        m.green,
        m.blue,
    );

    // Trail was hardcoded as (0.0, 1.5, 0.8, 1.0)
    let tr = t.trail_emissive;
    assert!(
        (tr.red - 0.0).abs() < 1e-5
            && (tr.green - 1.5).abs() < 1e-5
            && (tr.blue - 0.8).abs() < 1e-5,
        "trail_emissive should be (0.0,1.5,0.8) — got ({},{},{})",
        tr.red,
        tr.green,
        tr.blue,
    );

    trace!("t_twc_02: passed — world colours match hardcoded values");
}

// ---------------------------------------------------------------------------
// T-TWC-03  Monolith glow channels are all positive
// ---------------------------------------------------------------------------

#[test]
fn t_twc_03_monolith_glow_channels_valid() {
    let t = tet_theme();

    for (i, &ch) in t.monolith_glow_channels.iter().enumerate() {
        assert!(
            ch > 0.0,
            "monolith_glow_channels[{i}] should be > 0, got {ch}"
        );
    }

    trace!(
        "t_twc_03: passed — glow channels = {:?}",
        t.monolith_glow_channels
    );
}

// ---------------------------------------------------------------------------
// T-TWC-04  Scene setup uses theme colours for monolith material
// ---------------------------------------------------------------------------

#[test]
fn t_twc_04_scene_setup_uses_theme_colors() {
    let mut app = build_themed_tet_app();
    app.update();

    let theme_mono = tet_theme().monolith_emissive;

    // Find the monolith entity and its material handle.
    let mut query = app
        .world_mut()
        .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<TetMonolith>>();
    let mat_handle = query
        .iter(app.world())
        .next()
        .expect("monolith entity should exist after update");

    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let mat = materials
        .get(&mat_handle.0)
        .expect("monolith material should be loaded");

    assert!(
        (mat.emissive.red - theme_mono.red).abs() < 1e-4
            && (mat.emissive.green - theme_mono.green).abs() < 1e-4
            && (mat.emissive.blue - theme_mono.blue).abs() < 1e-4,
        "monolith emissive ({},{},{}) should match theme ({},{},{})",
        mat.emissive.red,
        mat.emissive.green,
        mat.emissive.blue,
        theme_mono.red,
        theme_mono.green,
        theme_mono.blue,
    );

    trace!("t_twc_04: passed — monolith material matches theme");
}

// ---------------------------------------------------------------------------
// T-TWC-05  Glow system reads theme without panicking
// ---------------------------------------------------------------------------

#[test]
fn t_twc_05_glow_system_reads_theme() {
    let mut app = build_themed_tet_app();
    app.add_systems(Update, deer_gui::scene::tet::systems::tet_glow_system);

    // Run several frames — glow system should not panic and the monolith
    // entity + material should survive.
    for _ in 0..10 {
        app.update();
    }

    let star_count = app
        .world_mut()
        .query_filtered::<Entity, With<Star>>()
        .iter(app.world())
        .count();
    assert!(star_count > 0, "stars should still exist after glow frames");

    let mono_count = app
        .world_mut()
        .query_filtered::<Entity, With<TetMonolith>>()
        .iter(app.world())
        .count();
    assert_eq!(
        mono_count, 1,
        "monolith should still exist after glow frames"
    );

    trace!("t_twc_05: passed — glow system ran 10 frames without panic");
}
