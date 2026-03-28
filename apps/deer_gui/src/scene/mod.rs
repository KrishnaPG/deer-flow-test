//! TET Scene module — environment, atmosphere, and particle systems.
//!
//! Re-exports the [`ScenePlugin`] for app integration and the
//! [`SceneConfig`] trait for scene-specific configuration.

pub mod common;
mod plugin;
pub mod tet;
mod traits;

pub use plugin::ScenePlugin;
pub use traits::SceneConfig;
