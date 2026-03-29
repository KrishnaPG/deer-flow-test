//! Deer GUI library root — re-exports all modules for integration tests.

pub mod audio;
pub mod bridge;
pub mod camera;
pub mod constants;
pub mod diagnostics;
pub mod hud;
pub mod models;
pub mod picking;
pub mod scene;
pub mod theme;
// pub mod ui; // FIXME: fonts.rs has compilation errors (pre-existing)
pub mod world;
