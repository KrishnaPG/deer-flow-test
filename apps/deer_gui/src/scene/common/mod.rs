//! Shared scene systems — parallax scrolling, atmosphere transitions, and weather.

use bevy::prelude::Resource;

pub mod atmosphere;
pub mod parallax;
pub mod weather;

pub use atmosphere::{atmosphere_transition_system, AtmosphereConfig, AtmosphereState};
pub use parallax::{parallax_scroll_system, ParallaxLayer};
pub use weather::{weather_transition_system, weather_update_system, WeatherMachine, WeatherState};

// ---------------------------------------------------------------------------
// Active Scene Tracking
// ---------------------------------------------------------------------------

/// Resource tracking the currently active scene name.
///
/// Used by per-frame systems to conditionally run only in specific scenes.
/// Updated by [`SceneManager`] on activate/deactivate.
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveSceneTag(pub Option<String>);
