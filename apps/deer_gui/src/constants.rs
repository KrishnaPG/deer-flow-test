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
    /// Default fade-in duration (seconds) when starting scene ambient audio.
    pub const SCENE_AUDIO_FADE_IN_SECS: f32 = 2.0;
    /// Default fade-out duration (seconds) when stopping scene ambient audio.
    pub const SCENE_AUDIO_FADE_OUT_SECS: f32 = 1.5;
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

    /// Coarse picking radius in pixels for the two-phase picking system.
    pub const PICKING_COARSE_RADIUS_PX: f32 = 24.0;
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
// Performance / pre-allocation
// ---------------------------------------------------------------------------

/// Buffer sizes and capacity hints for hot-path pre-allocation.
pub mod perf {
    /// Initial capacity for the bridge event queue (events per frame).
    ///
    /// Matches [`super::timing::BRIDGE_POLL_MAX_PER_FRAME`] so the queue
    /// never reallocates during normal operation.
    pub const EVENT_QUEUE_CAPACITY: usize = 64;

    /// Initial capacity (bytes) for the reader-thread line buffer.
    ///
    /// 4 KiB covers the vast majority of bridge JSON messages without
    /// reallocation; messages exceeding this grow the buffer once and it
    /// stays enlarged for the process lifetime.
    pub const LINE_BUFFER_CAPACITY: usize = 4096;
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

// ---------------------------------------------------------------------------
// Precursors scene
// ---------------------------------------------------------------------------

/// Precursors scene constants — barge and traveller counts, speeds, and radii.
pub mod precursors {
    /// Number of river barges in the Precursors scene.
    pub const BARGE_COUNT: usize = 60;
    /// Number of path travellers in the Precursors scene.
    pub const TRAVELLER_COUNT: usize = 80;
    /// Base speed of river barges (units per frame multiplier).
    pub const BARGE_SPEED: f32 = 3.0;
    /// Base speed of path travellers (units per frame multiplier).
    pub const TRAVELLER_SPEED: f32 = 2.0;
    /// Radius of the river path for barges.
    pub const RIVER_RADIUS: f32 = 300.0;
    /// Radius of the path for travellers.
    pub const PATH_RADIUS: f32 = 250.0;
}

// ---------------------------------------------------------------------------
// Descent scene
// ---------------------------------------------------------------------------

/// Descent scene visual parameters.
pub mod descent {
    /// Number of cloud particles in the cloud layer.
    pub const CLOUD_COUNT: usize = 120;
    /// Number of drop pods falling through the scene.
    pub const POD_COUNT: usize = 40;
    /// Number of colony beacons (static glow entities).
    pub const BEACON_COUNT: usize = 20;
    /// Speed of cloud particles moving upward.
    pub const CLOUD_SPEED: f32 = 6.0;
    /// Speed of drop pods falling downward.
    pub const POD_SPEED: f32 = 8.0;
    /// Radius of the cloud layer distribution.
    pub const CLOUD_RADIUS: f32 = 400.0;
}

// ---------------------------------------------------------------------------
// Font URLs for Google Fonts CSS
// ---------------------------------------------------------------------------

/// Google Fonts CSS URLs and user agent for font loading.
pub mod fonts {
    /// IBM Plex Sans CSS URL (body font).
    pub const DEFAULT_BODY_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&display=swap";

    /// JetBrains Mono CSS URL (monospace font).
    pub const DEFAULT_MONO_FONT_CSS: &str =
        "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap";

    /// User agent header for font requests (modern browser).
    pub const FONT_CSS_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
}
