//! TET scene — the central tetrahedral structure environment.

pub mod config;
pub mod particles;
pub mod setup;
pub mod systems;

pub use config::TetSceneConfig;
pub use setup::tet_scene_setup_system;
pub use systems::{data_trail_system, tet_glow_system};
