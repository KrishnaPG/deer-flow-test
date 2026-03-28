//! Shared scene systems — parallax scrolling, atmosphere transitions, and weather.

pub mod atmosphere;
pub mod parallax;
pub mod weather;

pub use atmosphere::{atmosphere_transition_system, AtmosphereConfig, AtmosphereState};
pub use parallax::{parallax_scroll_system, ParallaxLayer};
pub use weather::{weather_transition_system, weather_update_system, WeatherMachine, WeatherState};
