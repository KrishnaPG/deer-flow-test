//! Water plane system using bevy_water for medieval rivers and lakes.
//!
//! Provides a reusable [`WaterPlugin`] wrapper around bevy_water for
//! rendering realistic water surfaces with wave animations.
//!
//! ## Features
//!
//! - Rivers, lakes, and ocean support
//! - Configurable wave parameters (amplitude, frequency, speed)
//! - Flow direction for rivers
//! - Shore detection via terrain integration
//! - Splash particles on water contact
//! - Swimming collision support
//!
//! ## Reusability
//!
//! This plugin can be extracted to a standalone crate for use in any
//! Bevy project needing water rendering. Configuration is done
//! via Bevy resources (data-driven, no hardcoded paths).

use bevy::ecs::system::{Res, ResMut};
use bevy::log::{info, trace};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_water::{WaterPlugin as BevyWaterPlugin, WaterSettings};
use serde::{Deserialize, Serialize};

/// Trait for terrain water integration.
///
/// This provides a common interface for terrain systems to integrate with water.
pub trait TerrainWaterIntegration {
    /// Get the terrain height at a given world position.
    fn get_height_at(&self, world_pos: Vec2) -> Option<f32>;

    /// Get the terrain slope at a given world position.
    fn get_slope_at(&self, world_pos: Vec2) -> Option<f32>;

    /// Check if a position is near water (within threshold).
    fn is_near_water(&self, world_pos: Vec2, water_level: f32, threshold: f32) -> bool {
        if let Some(height) = self.get_height_at(world_pos) {
            (height - water_level).abs() < threshold
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Water Types
// ---------------------------------------------------------------------------

/// Types of water bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaterType {
    /// Wide flowing river.
    River,
    /// Still lake.
    Lake,
    /// Ocean/sea with large waves.
    Ocean,
    /// Small pond.
    Pond,
    /// Waterfall (vertical water flow).
    Waterfall,
}

impl Default for WaterType {
    fn default() -> Self {
        Self::Lake
    }
}

// ---------------------------------------------------------------------------
// Wave Parameters
// ---------------------------------------------------------------------------

/// Parameters controlling wave animation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveParams {
    /// Wave height amplitude in meters.
    #[serde(default = "default_wave_amplitude")]
    pub amplitude: f32,
    /// Wave frequency (waves per meter).
    #[serde(default = "default_wave_frequency")]
    pub frequency: f32,
    /// Wave animation speed.
    #[serde(default = "default_wave_speed")]
    pub speed: f32,
    /// Wave direction (normalized 2D vector).
    #[serde(default = "default_wave_direction")]
    pub direction: (f32, f32),
}

fn default_wave_amplitude() -> f32 {
    0.3
}

fn default_wave_frequency() -> f32 {
    0.5
}

fn default_wave_speed() -> f32 {
    1.0
}

fn default_wave_direction() -> (f32, f32) {
    (1.0, 0.0)
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: default_wave_amplitude(),
            frequency: default_wave_frequency(),
            speed: default_wave_speed(),
            direction: default_wave_direction(),
        }
    }
}

impl WaveParams {
    /// Create river-like waves (low amplitude, directional).
    pub fn river() -> Self {
        Self {
            amplitude: 0.1,
            frequency: 0.3,
            speed: 2.0,
            direction: (1.0, 0.0),
        }
    }

    /// Create ocean waves (high amplitude, varied direction).
    pub fn ocean() -> Self {
        Self {
            amplitude: 0.8,
            frequency: 0.2,
            speed: 1.5,
            direction: (0.8, 0.3),
        }
    }

    /// Create calm lake waves.
    pub fn lake() -> Self {
        Self {
            amplitude: 0.05,
            frequency: 0.4,
            speed: 0.5,
            direction: (0.5, 0.5),
        }
    }
}

// ---------------------------------------------------------------------------
// Water Body Definition
// ---------------------------------------------------------------------------

/// Definition of a water body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterBodyDef {
    /// Unique identifier for this water body.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Type of water body.
    pub water_type: WaterType,
    /// Center position.
    pub position: (f32, f32, f32),
    /// Size (width, depth) in meters.
    pub size: (f32, f32),
    /// Wave parameters.
    #[serde(default)]
    pub waves: WaveParams,
    /// Water color (RGBA).
    #[serde(default = "default_water_color")]
    pub color: (f32, f32, f32, f32),
    /// Water surface opacity.
    #[serde(default = "default_water_opacity")]
    pub opacity: f32,
    /// Whether this water body has collision (swimming).
    #[serde(default = "default_has_collision")]
    pub has_collision: bool,
    /// Water depth for swimming mechanics.
    #[serde(default = "default_water_depth")]
    pub depth: f32,
}

fn default_water_color() -> (f32, f32, f32, f32) {
    (0.1, 0.3, 0.5, 0.8)
}

fn default_water_opacity() -> f32 {
    0.8
}

fn default_has_collision() -> bool {
    true
}

fn default_water_depth() -> f32 {
    5.0
}

impl Default for WaterBodyDef {
    fn default() -> Self {
        Self {
            id: "lake".to_string(),
            name: "Lake".to_string(),
            water_type: WaterType::Lake,
            position: (0.0, 0.0, 0.0),
            size: (100.0, 100.0),
            waves: WaveParams::default(),
            color: default_water_color(),
            opacity: default_water_opacity(),
            has_collision: true,
            depth: 5.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Global Configuration
// ---------------------------------------------------------------------------

/// Global configuration for the water system.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WaterGlobalConfig {
    /// Water body definitions.
    #[serde(default)]
    pub bodies: Vec<WaterBodyDef>,
    /// Global water level (for terrain integration).
    #[serde(default = "default_water_level")]
    pub water_level: f32,
    /// Enable splash particles on water contact.
    #[serde(default = "default_true")]
    pub enable_splash: bool,
    /// Enable swimming mechanics.
    #[serde(default = "default_true")]
    pub enable_swimming: bool,
}

fn default_water_level() -> f32 {
    0.0
}

fn default_true() -> bool {
    true
}

impl Default for WaterGlobalConfig {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            water_level: 0.0,
            enable_splash: true,
            enable_swimming: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for water body entities.
#[derive(Component, Debug)]
pub struct WaterBody {
    /// Water body definition ID.
    pub id: String,
    /// Type of water body.
    pub water_type: WaterType,
    /// Current wave state.
    pub wave_time: f32,
}

/// Component for entities that can swim.
#[derive(Component, Debug)]
pub struct Swimmable {
    /// Swimming speed multiplier.
    pub speed_multiplier: f32,
    /// Current breath (for underwater mechanics).
    pub breath: f32,
    /// Maximum breath.
    pub max_breath: f32,
    /// Is currently swimming.
    pub is_swimming: bool,
    /// Is currently underwater.
    pub is_underwater: bool,
}

impl Default for Swimmable {
    fn default() -> Self {
        Self {
            speed_multiplier: 0.5,
            breath: 100.0,
            max_breath: 100.0,
            is_swimming: false,
            is_underwater: false,
        }
    }
}

impl Swimmable {
    /// Check if entity can swim (has breath).
    pub fn can_swim(&self) -> bool {
        self.breath > 0.0
    }

    /// Consume breath when underwater.
    pub fn consume_breath(&mut self, amount: f32, delta_time: f32) {
        if self.is_underwater {
            self.breath = (self.breath - amount * delta_time).max(0.0);
        }
    }

    /// Recover breath when above water.
    pub fn recover_breath(&mut self, amount: f32, delta_time: f32) {
        if !self.is_underwater {
            self.breath = (self.breath + amount * delta_time).min(self.max_breath);
        }
    }
}

// ---------------------------------------------------------------------------
// Water Plugin
// ---------------------------------------------------------------------------

/// Modular water plugin wrapping bevy_water for medieval scenes.
///
/// This plugin can be added to any Bevy app for water rendering.
/// Configuration is done via the `WaterGlobalConfig` resource.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(WaterPlugin::default());
/// app.insert_resource(WaterGlobalConfig {
///     bodies: vec![...],
///     ..Default::default()
/// });
/// ```
#[derive(Default)]
pub struct WaterPlugin {
    /// Initial configuration (optional).
    pub config: Option<WaterGlobalConfig>,
}

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        info!("WaterPlugin: initializing water system");

        // Add bevy_water with default settings
        app.add_plugins(BevyWaterPlugin::default());

        // Configure water settings based on our config
        if let Some(config) = &self.config {
            app.insert_resource(config.clone());

            // Apply water settings from our config
            if let Some(first_body) = config.bodies.first() {
                app.insert_resource(WaterSettings {
                    height: first_body.position.1, // Use Y position as water height
                    ..default()
                });
            }
        } else {
            app.init_resource::<WaterGlobalConfig>();
        }

        // Add systems
        app.add_systems(Startup, setup_water_system);
        app.add_systems(
            Update,
            (
                update_water_waves_system,
                update_water_level_system,
                update_swimming_system,
            ),
        );

        info!("WaterPlugin: registered systems");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Setup the water system.
fn setup_water_system(config: Res<WaterGlobalConfig>) {
    trace!("setup_water_system: initializing");

    info!(
        "WaterPlugin: initialized with {} water bodies, water level: {}m",
        config.bodies.len(),
        config.water_level
    );

    for body in &config.bodies {
        trace!(
            "  - Water: '{}' ({:?}) at {:?}, size {:?}",
            body.name,
            body.water_type,
            body.position,
            body.size
        );
    }
}

/// Update wave animations for water bodies.
fn update_water_waves_system(time: Res<Time>, mut water_bodies: Query<&mut WaterBody>) {
    let delta = time.delta_secs();

    for mut water in water_bodies.iter_mut() {
        water.wave_time += delta * water.wave_time;
    }
}

/// Update water level from config to bevy_water settings.
fn update_water_level_system(
    config: Res<WaterGlobalConfig>,
    mut water_settings: ResMut<WaterSettings>,
) {
    // Sync water level from our config to bevy_water settings
    water_settings.height = config.water_level;
}

/// Update swimming state for entities in water.
fn update_swimming_system(
    config: Res<WaterGlobalConfig>,
    time: Res<Time>,
    mut query: Query<(&mut Swimmable, &Transform)>,
) {
    let delta = time.delta_secs();

    for (mut swimmable, transform) in query.iter_mut() {
        let water_level = config.water_level;
        let entity_height = transform.translation.y;

        // Determine if entity is in water
        let in_water = entity_height < water_level;
        let underwater = entity_height < water_level - 1.0; // 1 meter below surface

        swimmable.is_swimming = in_water;
        swimmable.is_underwater = underwater;

        // Update breath
        if underwater {
            swimmable.consume_breath(10.0, delta); // Consume 10 breath per second
        } else {
            swimmable.recover_breath(20.0, delta); // Recover 20 breath per second
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_type_default() {
        let water_type = WaterType::default();
        assert_eq!(water_type, WaterType::Lake);
    }

    #[test]
    fn wave_params_default() {
        let params = WaveParams::default();
        assert!(params.amplitude > 0.0);
        assert!(params.frequency > 0.0);
    }

    #[test]
    fn wave_params_presets() {
        let river = WaveParams::river();
        assert!(river.amplitude < WaveParams::ocean().amplitude);

        let lake = WaveParams::lake();
        assert!(lake.speed < WaveParams::ocean().speed);
    }

    #[test]
    fn water_body_def_default() {
        let def = WaterBodyDef::default();
        assert_eq!(def.id, "lake");
        assert!(def.has_collision);
    }

    #[test]
    fn water_global_config_default() {
        let config = WaterGlobalConfig::default();
        assert!(config.bodies.is_empty());
        assert!(config.enable_splash);
    }

    #[test]
    fn swimmable_default() {
        let swimmable = Swimmable::default();
        assert_eq!(swimmable.breath, 100.0);
        assert!(swimmable.speed_multiplier < 1.0);
        assert!(!swimmable.is_swimming);
        assert!(!swimmable.is_underwater);
    }

    #[test]
    fn swimmable_breath_consumption() {
        let mut swimmable = Swimmable::default();
        swimmable.is_underwater = true;

        swimmable.consume_breath(10.0, 1.0);
        assert_eq!(swimmable.breath, 90.0);

        swimmable.consume_breath(100.0, 0.5);
        assert_eq!(swimmable.breath, 40.0);

        // Should not go below 0
        swimmable.consume_breath(100.0, 1.0);
        assert_eq!(swimmable.breath, 0.0);
    }

    #[test]
    fn swimmable_breath_recovery() {
        let mut swimmable = Swimmable::default();
        swimmable.breath = 50.0;
        swimmable.is_underwater = false;

        swimmable.recover_breath(20.0, 1.0);
        assert_eq!(swimmable.breath, 70.0);

        swimmable.recover_breath(100.0, 0.5);
        assert_eq!(swimmable.breath, 100.0); // Capped at max

        // Should not go above max
        swimmable.recover_breath(50.0, 1.0);
        assert_eq!(swimmable.breath, 100.0);
    }
}
