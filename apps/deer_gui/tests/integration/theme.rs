//! Integration tests for the theme engine.
//!
//! Uses `MinimalPlugins` with a manually inserted [`ThemeManager`] to
//! avoid EGui / windowing dependencies required by [`ThemePlugin`].

use bevy::app::App;
use bevy::log::trace;
use bevy::prelude::*;

use deer_gui::theme::{tet_theme, Theme, ThemeManager};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a simple alt theme for multi-theme tests.
fn alt_theme() -> Theme {
    let mut t = tet_theme();
    t.name = "Alt Theme".to_string();
    t
}

// ---------------------------------------------------------------------------
// T-THEME-01: ThemeManager is queryable after app.update()
// ---------------------------------------------------------------------------

#[test]
fn t_theme_01_manager_queryable_after_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ThemeManager::new(vec![tet_theme()]));
    app.update();

    let tm = app.world().resource::<ThemeManager>();
    assert_eq!(tm.active_name(), "TET Orchestrator");
    trace!(
        "t_theme_01: passed — active theme is '{}'",
        tm.active_name()
    );
}

// ---------------------------------------------------------------------------
// T-THEME-02: current() returns active theme with correct name
// ---------------------------------------------------------------------------

#[test]
fn t_theme_02_current_returns_active_theme() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ThemeManager::new(vec![tet_theme(), alt_theme()]));
    app.update();

    let tm = app.world().resource::<ThemeManager>();
    let current = tm.current();
    assert_eq!(current.name, "TET Orchestrator");
    trace!("t_theme_02: passed — current().name = '{}'", current.name);
}

// ---------------------------------------------------------------------------
// T-THEME-03: Switch to non-existent theme returns false, no change
// ---------------------------------------------------------------------------

#[test]
fn t_theme_03_switch_nonexistent_returns_false() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ThemeManager::new(vec![tet_theme()]));
    app.update();

    let tm = app.world_mut().resource_mut::<ThemeManager>();
    let gen_before = tm.generation;
    let name_before = tm.active_name().to_string();

    let mut tm = app.world_mut().resource_mut::<ThemeManager>();
    let switched = tm.switch("Does Not Exist");

    assert!(!switched, "switch() should return false for unknown theme");
    assert_eq!(tm.active_name(), name_before);
    assert_eq!(tm.generation, gen_before);
    trace!("t_theme_03: passed — switch to unknown theme rejected");
}

// ---------------------------------------------------------------------------
// T-THEME-04: available_themes() lists all registered names
// ---------------------------------------------------------------------------

#[test]
fn t_theme_04_available_themes_lists_all() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ThemeManager::new(vec![tet_theme(), alt_theme()]));
    app.update();

    let tm = app.world().resource::<ThemeManager>();
    let names = tm.available_themes();
    assert_eq!(names, vec!["TET Orchestrator", "Alt Theme"]);
    trace!("t_theme_04: passed — available themes = {:?}", names);
}

// ---------------------------------------------------------------------------
// T-THEME-05: Generation increments on successful switch
// ---------------------------------------------------------------------------

#[test]
fn t_theme_05_generation_increments_on_switch() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ThemeManager::new(vec![tet_theme(), alt_theme()]));
    app.update();

    let gen_before = app.world().resource::<ThemeManager>().generation;

    let switched = app
        .world_mut()
        .resource_mut::<ThemeManager>()
        .switch("Alt Theme");

    assert!(switched, "switch() should return true for known theme");

    let tm = app.world().resource::<ThemeManager>();
    assert_eq!(tm.active_name(), "Alt Theme");
    assert!(
        tm.generation > gen_before,
        "generation ({}) should be greater than before ({})",
        tm.generation,
        gen_before,
    );
    trace!(
        "t_theme_05: passed — gen {} → {}",
        gen_before,
        tm.generation,
    );
}
