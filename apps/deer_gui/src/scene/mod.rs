//! Scene module — environment management, atmosphere, and particle systems.
//!
//! Re-exports the [`ScenePlugin`] for app integration, the
//! [`SceneConfig`] trait for scene-specific configuration, and
//! [`SceneManager`] / [`SceneRoot`] for runtime scene activation.

pub mod audio_bridge;
pub mod common;
pub mod manager;
mod plugin;
pub mod primitives;
pub mod tet;
mod traits;

pub use audio_bridge::{scene_audio_bridge_system, SceneAudioState};
pub use manager::{SceneManager, SceneRoot};
pub use plugin::ScenePlugin;
pub use primitives::{
    entity_scale, fibonacci_sphere_point, spawn_root, spawn_scene_ambient_light, spawn_starfield,
    Star,
};
pub use traits::SceneConfig;
