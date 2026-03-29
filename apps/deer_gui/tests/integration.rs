//! Single integration-test binary for deer_gui.
//!
//! All test modules live under `integration/` and are compiled into one
//! binary to avoid the massive link-time overhead of Bevy test binaries
//! (each ~hundreds of MB). This keeps total disk usage manageable on the
//! RAM disk.

mod integration {
    pub mod audio;
    pub mod bridge;
    pub mod bridge_adapter;
    pub mod camera;
    pub mod constants_diagnostics;
    pub mod hud;
    pub mod models_pipeline;
    pub mod picking;
    pub mod scene_audio;
    pub mod scene_descriptors;
    pub mod scene_manager;
    pub mod scene_primitives;
    pub mod scene_tet;
    pub mod scene_weather;
    pub mod theme;
    pub mod theme_world_colors;
    pub mod world;
    pub mod world_extended;
}
