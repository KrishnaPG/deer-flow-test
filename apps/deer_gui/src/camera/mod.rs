//! Cinematic orbital camera module.
//!
//! Provides a smooth, interpolated orbital camera with shake effects
//! and focus-target tracking. All constants are sourced from
//! [`crate::constants::camera`].

mod components;
mod plugin;
mod systems;

pub use components::*;
pub use plugin::CameraPlugin;
pub use systems::*;
