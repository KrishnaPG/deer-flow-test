//! Cinematic orbital camera module.
//!
//! Provides a smooth, interpolated orbital camera with shake effects
//! and focus-target tracking. All constants are sourced from
//! [`crate::constants::camera`].

mod components;
pub mod navigation;
mod plugin;
mod systems;

pub use components::*;
pub use navigation::*;
pub use plugin::CameraPlugin;
pub use systems::*;
