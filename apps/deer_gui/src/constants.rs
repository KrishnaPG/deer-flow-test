//! Centralized constants for the Deer GUI Control Center.
//!
//! All magic numbers, colors, z-layers, timings, and thresholds live here.
//! No other module should define literal constants — import from here instead.

// ---------------------------------------------------------------------------
// Z-depth layers
// ---------------------------------------------------------------------------

/// Depth layering for world-space and HUD elements.
pub mod z_layer {
    /// Far background (skybox, starfield).
    pub const Z_WORLD_FAR: f32 = -1000.0;
    /// Mid-ground (terrain, structures).
    pub const Z_WORLD_MID: f32 = -500.0;
    /// Near-ground (agents, particles).
    pub const Z_WORLD_NEAR: f32 = -100.0;
    /// Overlay / HUD layer.
    pub const Z_HUD: f32 = 100.0;
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// Orbital-camera defaults, limits, and sensitivity multipliers.
pub mod camera {
    /// Starting horizontal rotation (degrees).
    pub const DEFAULT_YAW: f32 = 0.0;
    /// Starting vertical tilt (degrees, negative = looking down).
    pub const DEFAULT_PITCH: f32 = -15.0;
    /// Starting zoom factor (1.0 = normal).
    pub const DEFAULT_ZOOM: f32 = 1.0;

    /// Closest the camera may zoom.
    pub const MIN_ZOOM: f32 = 0.5;
    /// Farthest the camera may zoom.
    pub const MAX_ZOOM: f32 = 5.0;

    /// Lowest allowed pitch (degrees).
    pub const MIN_PITCH: f32 = -60.0;
    /// Highest allowed pitch (degrees).
    pub const MAX_PITCH: f32 = 10.0;

    /// Smooth-follow interpolation speed (units / sec).
    pub const INTERPOLATION_SPEED: f32 = 4.0;
    /// Shake-effect exponential decay rate.
    pub const SHAKE_DECAY: f32 = 5.0;

    /// Mouse-drag → yaw conversion factor.
    pub const YAW_SENSITIVITY: f32 = 0.3;
    /// Mouse-drag → pitch conversion factor.
    pub const PITCH_SENSITIVITY: f32 = 0.2;
    /// Scroll-wheel → zoom conversion factor.
    pub const ZOOM_SENSITIVITY: f32 = 0.1;

    /// Distance from the orbit target to the camera.
    pub const ORBIT_RADIUS: f32 = 200.0;
}

// ---------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------

/// Frame budgets, fade durations, and periodic rates.
pub mod timing {
    /// Max bridge messages drained per frame.
    pub const BRIDGE_POLL_MAX_PER_FRAME: usize = 64;
    /// Seconds before an event-ticker entry fades out.
    pub const EVENT_TICKER_FADE_SECS: f32 = 8.0;
    /// Maximum visible entries in the event ticker.
    pub const EVENT_TICKER_MAX_ENTRIES: usize = 50;
    /// Seconds for a weather-state crossfade.
    pub const WEATHER_TRANSITION_SECS: f32 = 3.0;
    /// Beacon pulse frequency (Hz).
    pub const BEACON_PULSE_HZ: f32 = 2.0;
}

// ---------------------------------------------------------------------------
// Visual
// ---------------------------------------------------------------------------

/// Rendering sizes, opacities, counts, and glow ranges.
pub mod visual {
    /// Radius of individual agent particle quads.
    pub const AGENT_PARTICLE_SIZE: f32 = 0.3;
    /// Background alpha for HUD panels (0.0–1.0).
    pub const HUD_PANEL_ALPHA: f32 = 0.75;
    /// Corner rounding for HUD panels (pixels).
    pub const HUD_PANEL_ROUNDING: f32 = 8.0;
    /// Hard cap on concurrently rendered agents.
    pub const MAX_VISIBLE_AGENTS: usize = 500;

    /// Radius of the central tetrahedral structure.
    pub const TET_STRUCTURE_RADIUS: f32 = 50.0;
    /// Minimum glow intensity of the tet structure.
    pub const TET_GLOW_MIN: f32 = 0.3;
    /// Maximum glow intensity of the tet structure.
    pub const TET_GLOW_MAX: f32 = 1.0;

    /// Number of stars in the background starfield.
    pub const STARFIELD_COUNT: usize = 2000;
    /// Radius of the starfield sphere.
    pub const STARFIELD_RADIUS: f32 = 800.0;

    /// Movement speed of data-trail particles (units / sec).
    pub const DATA_TRAIL_SPEED: f32 = 5.0;
    /// Number of active data-trail particles.
    pub const DATA_TRAIL_COUNT: usize = 100;
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

/// OS-window title and size constraints.
pub mod window {
    /// Window title shown in the OS title bar.
    pub const TITLE: &str = "Deer GUI \u{2014} Control Center";
    /// Default window width (pixels).
    pub const DEFAULT_WIDTH: u32 = 1480;
    /// Default window height (pixels).
    pub const DEFAULT_HEIGHT: u32 = 920;
    /// Minimum window width (pixels).
    pub const MIN_WIDTH: f32 = 1080.0;
    /// Minimum window height (pixels).
    pub const MIN_HEIGHT: f32 = 760.0;
}

// ---------------------------------------------------------------------------
// Weather thresholds
// ---------------------------------------------------------------------------

/// Metric thresholds that drive the ambient-weather system.
pub mod weather {
    /// Latency (ms) above which the scene turns foggy.
    pub const FOGGY_LATENCY_MS: f32 = 500.0;
    /// CPU/load fraction above which rain begins.
    pub const RAINY_LOAD_PCT: f32 = 0.7;
    /// Error-rate fraction above which a storm triggers.
    pub const STORMY_ERROR_RATE: f32 = 0.1;
}

// ---------------------------------------------------------------------------
// Aggregation
// ---------------------------------------------------------------------------

/// Swarm clustering and visual-density controls.
pub mod aggregation {
    /// Minimum agents in a cell before they are clustered.
    pub const SWARM_AGGREGATION_THRESHOLD: usize = 10;
    /// Visual-to-real ratio when a swarm cluster is shown.
    pub const SWARM_VISUAL_RATIO: f32 = 0.05;
}
