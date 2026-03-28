//! Single integration-test binary for deer_gui.
//!
//! All test modules live under `integration/` and are compiled into one
//! binary to avoid the massive link-time overhead of Bevy test binaries
//! (each ~hundreds of MB). This keeps total disk usage manageable on the
//! RAM disk.

mod integration {
    pub mod audio;
    pub mod camera;
    pub mod constants_diagnostics;
    pub mod scene_weather;
    pub mod theme;
    pub mod world;
}
