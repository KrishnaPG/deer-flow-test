//! Integration tests for the weather state machine.
//!
//! Covers WeatherMachine resource defaults, metric-to-weather evaluation,
//! priority ordering, and transition completion via real Bevy systems.

use bevy::prelude::*;

use deer_gui::constants::weather::{FOGGY_LATENCY_MS, RAINY_LOAD_PCT, STORMY_ERROR_RATE};
use deer_gui::scene::common::atmosphere::AtmosphereConfig;
use deer_gui::scene::common::weather::{
    weather_transition_system, weather_update_system, WeatherMachine, WeatherState,
};
use deer_gui::world::state::WorldState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with weather resources and systems only.
///
/// Deliberately avoids [`ScenePlugin`] which requires mesh/material assets.
fn weather_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<AtmosphereConfig>();
    app.init_resource::<WeatherMachine>();
    app.init_resource::<WorldState>();
    app.add_systems(Update, (weather_update_system, weather_transition_system));
    app
}

// ---------------------------------------------------------------------------
// T-SCENE-01  WeatherMachine resource registered after startup
// ---------------------------------------------------------------------------

#[test]
fn t_scene_01_weather_machine_registered() {
    let mut app = weather_app();
    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(weather.current, WeatherState::Clear);
    assert_eq!(weather.target, WeatherState::Clear);
}

// ---------------------------------------------------------------------------
// T-SCENE-02  WeatherMachine defaults to Clear
// ---------------------------------------------------------------------------

#[test]
fn t_scene_02_weather_defaults_clear() {
    let weather = WeatherMachine::default();
    assert_eq!(weather.current, WeatherState::Clear);
    assert_eq!(weather.target, WeatherState::Clear);
    assert_eq!(weather.transition_progress, 1.0);
}

// ---------------------------------------------------------------------------
// T-SCENE-03  Stormy when error_rate >= STORMY_ERROR_RATE
// ---------------------------------------------------------------------------

#[test]
fn t_scene_03_stormy_on_high_error_rate() {
    let mut app = weather_app();

    // Set error_rate above threshold.
    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.system_health.error_rate = STORMY_ERROR_RATE + 0.05;

    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(
        weather.target,
        WeatherState::Stormy,
        "error_rate={:.2} should trigger Stormy (threshold={STORMY_ERROR_RATE})",
        STORMY_ERROR_RATE + 0.05,
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-04  Clear when all metrics are healthy
// ---------------------------------------------------------------------------

#[test]
fn t_scene_04_clear_when_healthy() {
    let mut app = weather_app();

    // WorldState defaults are all-healthy.
    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(weather.target, WeatherState::Clear);
}

// ---------------------------------------------------------------------------
// T-SCENE-05  Foggy when avg_latency_ms >= FOGGY_LATENCY_MS
// ---------------------------------------------------------------------------

#[test]
fn t_scene_05_foggy_on_high_latency() {
    let mut app = weather_app();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.system_health.avg_latency_ms = FOGGY_LATENCY_MS + 100.0;

    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(
        weather.target,
        WeatherState::Foggy,
        "avg_latency_ms={:.0} should trigger Foggy (threshold={FOGGY_LATENCY_MS})",
        FOGGY_LATENCY_MS + 100.0,
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-06  Rainy when cpu_load >= RAINY_LOAD_PCT
// ---------------------------------------------------------------------------

#[test]
fn t_scene_06_rainy_on_high_cpu() {
    let mut app = weather_app();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.system_health.cpu_load = RAINY_LOAD_PCT + 0.1;

    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(
        weather.target,
        WeatherState::Rainy,
        "cpu_load={:.2} should trigger Rainy (threshold={RAINY_LOAD_PCT})",
        RAINY_LOAD_PCT + 0.1,
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-07  Stormy takes priority over Rainy
// ---------------------------------------------------------------------------

#[test]
fn t_scene_07_stormy_priority_over_rainy() {
    let mut app = weather_app();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.system_health.error_rate = STORMY_ERROR_RATE + 0.05;
    ws.system_health.cpu_load = RAINY_LOAD_PCT + 0.2;

    app.update();

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(
        weather.target,
        WeatherState::Stormy,
        "Stormy (error_rate) should take priority over Rainy (cpu_load)"
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-08  Transition completes after enough update cycles
// ---------------------------------------------------------------------------

#[test]
fn t_scene_08_transition_completes_over_time() {
    let mut app = weather_app();

    // First update to initialise time.
    app.update();

    // Trigger a weather change.
    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.system_health.error_rate = STORMY_ERROR_RATE + 0.05;

    // Run enough update cycles for the transition to complete.
    // Bevy's Time advances by a small real delta each update().
    // ~200 cycles should comfortably exceed WEATHER_TRANSITION_SECS (3.0s)
    // even with very small per-frame deltas (~16ms simulated).
    for _ in 0..200 {
        app.update();
    }

    let weather = app.world().resource::<WeatherMachine>();
    assert_eq!(
        weather.current,
        WeatherState::Stormy,
        "current weather should be Stormy after transition completes"
    );
    assert!(
        (weather.transition_progress - 1.0).abs() < f32::EPSILON,
        "transition_progress should be 1.0, got {}",
        weather.transition_progress
    );
}
