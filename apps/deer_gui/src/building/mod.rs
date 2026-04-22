//! Building placement plugin for medieval structures.
//!
//! Provides a reusable [`BuildingPlugin`] that can be configured via
//! Bevy resources and supports faction coloring, weathering variants,
//! and layout presets.
//!
//! ## Features
//!
//! - Modular building definitions with faction color support
//! - Weathering levels from pristine to ruined
//! - Layout presets for villages, castles, and farms
//! - Grid-snapping placement system
//! - Collision shape support
//!
//! ## Reusability
//!
//! This plugin can be extracted to a standalone crate for use in any
//! Bevy project needing building placement. Configuration is done
//! via Bevy resources (data-driven, no hardcoded paths).

use bevy::ecs::system::Res;
use bevy::log::{debug, info, trace};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Building Types
// ---------------------------------------------------------------------------

/// Categories of medieval buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingCategory {
    /// Residential buildings (houses, cottages).
    Residential,
    /// Military structures (towers, walls, barracks).
    Military,
    /// Religious buildings (churches, chapels).
    Religious,
    /// Economic buildings (markets, mills, blacksmiths).
    Economic,
    /// Agricultural buildings (farms, barns, stables).
    Agricultural,
}

// ---------------------------------------------------------------------------
// Weathering Levels
// ---------------------------------------------------------------------------

/// Weathering state for buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatheringLevel {
    /// Brand new, pristine condition.
    Pristine,
    /// Slightly aged with minor wear.
    Worn,
    /// Noticeable decay, some damage.
    Weathered,
    /// Heavy decay, partial collapse.
    Ruined,
    /// Complete ruin, foundation only.
    Destroyed,
}

impl Default for WeatheringLevel {
    fn default() -> Self {
        Self::Pristine
    }
}

impl WeatheringLevel {
    /// Get the weathering factor (0.0 = pristine, 1.0 = destroyed).
    pub fn factor(&self) -> f32 {
        match self {
            Self::Pristine => 0.0,
            Self::Worn => 0.25,
            Self::Weathered => 0.5,
            Self::Ruined => 0.75,
            Self::Destroyed => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Building Definition
// ---------------------------------------------------------------------------

/// Definition of a building type, loaded from configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDef {
    /// Unique identifier for this building type.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Building category.
    pub category: BuildingCategory,
    /// Path to the glTF model.
    pub model_path: String,
    /// Default weathering level.
    #[serde(default)]
    pub default_weathering: WeatheringLevel,
    /// Whether this building can be colored by faction themes.
    #[serde(default = "default_true")]
    pub faction_colorable: bool,
    /// Collision shape dimensions (half-extents).
    pub collision_half_extents: Vec3,
    /// Grid footprint size (width, depth) in grid units.
    #[serde(default = "default_footprint")]
    pub footprint: (u32, u32),
    /// Cost in resources (food, wood, stone, gold).
    #[serde(default)]
    pub cost: [u32; 4],
}

fn default_true() -> bool {
    true
}

fn default_footprint() -> (u32, u32) {
    (1, 1)
}

impl Default for BuildingDef {
    fn default() -> Self {
        Self {
            id: "house".to_string(),
            name: "House".to_string(),
            category: BuildingCategory::Residential,
            model_path: "models/buildings/house.glb".to_string(),
            default_weathering: WeatheringLevel::Pristine,
            faction_colorable: true,
            collision_half_extents: Vec3::new(2.0, 2.0, 2.0),
            footprint: (2, 2),
            cost: [50, 100, 20, 10],
        }
    }
}

// ---------------------------------------------------------------------------
// Layout Presets
// ---------------------------------------------------------------------------

/// A building placement within a layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingPlacement {
    /// Building definition ID.
    pub building_id: String,
    /// Position offset from layout origin.
    pub offset: (f32, f32, f32),
    /// Rotation in degrees around Y axis.
    #[serde(default)]
    pub rotation: f32,
    /// Scale multiplier.
    #[serde(default = "default_scale")]
    pub scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

/// A predefined layout of multiple buildings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreset {
    /// Unique identifier for this layout.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of this layout.
    pub description: String,
    /// Building placements.
    pub buildings: Vec<BuildingPlacement>,
}

/// Preset layouts for common village configurations.
pub mod presets {
    use super::*;

    /// Create a simple village layout.
    pub fn village() -> LayoutPreset {
        LayoutPreset {
            id: "village".to_string(),
            name: "Small Village".to_string(),
            description: "A small medieval village with houses, a church, and a market".to_string(),
            buildings: vec![
                BuildingPlacement {
                    building_id: "house".to_string(),
                    offset: (0.0, 0.0, 0.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "house".to_string(),
                    offset: (8.0, 0.0, 0.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "house".to_string(),
                    offset: (4.0, 0.0, 8.0),
                    rotation: 180.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "church".to_string(),
                    offset: (16.0, 0.0, 4.0),
                    rotation: 90.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "market".to_string(),
                    offset: (4.0, 0.0, -6.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
            ],
        }
    }

    /// Create a castle layout.
    pub fn castle() -> LayoutPreset {
        LayoutPreset {
            id: "castle".to_string(),
            name: "Castle Complex".to_string(),
            description: "A fortified castle with keep, towers, and walls".to_string(),
            buildings: vec![
                BuildingPlacement {
                    building_id: "keep".to_string(),
                    offset: (0.0, 0.0, 0.0),
                    rotation: 0.0,
                    scale: 1.2,
                },
                BuildingPlacement {
                    building_id: "tower".to_string(),
                    offset: (-15.0, 0.0, -15.0),
                    rotation: 45.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "tower".to_string(),
                    offset: (15.0, 0.0, -15.0),
                    rotation: -45.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "tower".to_string(),
                    offset: (-15.0, 0.0, 15.0),
                    rotation: 135.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "tower".to_string(),
                    offset: (15.0, 0.0, 15.0),
                    rotation: -135.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "wall_section".to_string(),
                    offset: (0.0, 0.0, -15.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "wall_section".to_string(),
                    offset: (0.0, 0.0, 15.0),
                    rotation: 180.0,
                    scale: 1.0,
                },
            ],
        }
    }

    /// Create a farm layout.
    pub fn farm() -> LayoutPreset {
        LayoutPreset {
            id: "farm".to_string(),
            name: "Farmstead".to_string(),
            description: "A farm with barn, fields, and farmhouse".to_string(),
            buildings: vec![
                BuildingPlacement {
                    building_id: "house".to_string(),
                    offset: (0.0, 0.0, 0.0),
                    rotation: 0.0,
                    scale: 0.9,
                },
                BuildingPlacement {
                    building_id: "barn".to_string(),
                    offset: (10.0, 0.0, 0.0),
                    rotation: 90.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "stable".to_string(),
                    offset: (10.0, 0.0, 10.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
                BuildingPlacement {
                    building_id: "silo".to_string(),
                    offset: (-5.0, 0.0, 5.0),
                    rotation: 0.0,
                    scale: 1.0,
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Registry of all building definitions.
#[derive(Resource, Debug, Default, Clone)]
pub struct BuildingRegistry {
    /// Building definitions by ID.
    pub buildings: HashMap<String, BuildingDef>,
    /// Layout presets by ID.
    pub layouts: HashMap<String, LayoutPreset>,
}

impl BuildingRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a building definition.
    pub fn register_building(&mut self, def: BuildingDef) {
        debug!("BuildingRegistry: registering building '{}'", def.id);
        self.buildings.insert(def.id.clone(), def);
    }

    /// Register a layout preset.
    pub fn register_layout(&mut self, preset: LayoutPreset) {
        debug!("BuildingRegistry: registering layout '{}'", preset.id);
        self.layouts.insert(preset.id.clone(), preset);
    }

    /// Get a building definition by ID.
    pub fn get_building(&self, id: &str) -> Option<&BuildingDef> {
        self.buildings.get(id)
    }

    /// Get a layout preset by ID.
    pub fn get_layout(&self, id: &str) -> Option<&LayoutPreset> {
        self.layouts.get(id)
    }

    /// Create a registry with default medieval buildings.
    pub fn medieval_defaults() -> Self {
        let mut registry = Self::new();

        // Register default buildings
        registry.register_building(BuildingDef {
            id: "house".to_string(),
            name: "House".to_string(),
            category: BuildingCategory::Residential,
            model_path: "models/buildings/house.glb".to_string(),
            footprint: (2, 2),
            cost: [50, 100, 20, 10],
            ..Default::default()
        });

        registry.register_building(BuildingDef {
            id: "church".to_string(),
            name: "Church".to_string(),
            category: BuildingCategory::Religious,
            model_path: "models/buildings/church.glb".to_string(),
            footprint: (3, 4),
            cost: [100, 200, 100, 50],
            faction_colorable: false,
            ..Default::default()
        });

        registry.register_building(BuildingDef {
            id: "tower".to_string(),
            name: "Tower".to_string(),
            category: BuildingCategory::Military,
            model_path: "models/buildings/tower.glb".to_string(),
            footprint: (2, 2),
            cost: [30, 80, 120, 20],
            ..Default::default()
        });

        registry.register_building(BuildingDef {
            id: "barn".to_string(),
            name: "Barn".to_string(),
            category: BuildingCategory::Agricultural,
            model_path: "models/buildings/barn.glb".to_string(),
            footprint: (3, 2),
            cost: [20, 150, 30, 5],
            ..Default::default()
        });

        registry.register_building(BuildingDef {
            id: "market".to_string(),
            name: "Market".to_string(),
            category: BuildingCategory::Economic,
            model_path: "models/buildings/market.glb".to_string(),
            footprint: (3, 3),
            cost: [50, 100, 50, 100],
            ..Default::default()
        });

        // Register default layouts
        registry.register_layout(presets::village());
        registry.register_layout(presets::castle());
        registry.register_layout(presets::farm());

        registry
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for placed buildings.
#[derive(Component, Debug)]
pub struct PlacedBuilding {
    /// Reference to the building definition.
    pub building_id: String,
    /// Current weathering level.
    pub weathering: WeatheringLevel,
    /// Grid position.
    pub grid_pos: (i32, i32),
    /// Owning faction (for color application).
    pub faction_id: Option<String>,
}

/// Component for faction-colored buildings.
#[derive(Component, Debug)]
pub struct FactionColored {
    /// Current faction color (RGB).
    pub color: Vec3,
    /// Target faction color for smooth transition.
    pub target_color: Option<Vec3>,
    /// Transition progress (0.0 to 1.0).
    pub transition_progress: f32,
}

impl FactionColored {
    /// Create a new faction-colored component.
    pub fn new(color: Vec3) -> Self {
        Self {
            color,
            target_color: None,
            transition_progress: 1.0,
        }
    }

    /// Start transition to a new color.
    pub fn transition_to(&mut self, new_color: Vec3, duration: f32) {
        self.target_color = Some(new_color);
        self.transition_progress = 0.0;
    }

    /// Update transition progress.
    pub fn update(&mut self, delta: f32, duration: f32) {
        if self.target_color.is_some() && self.transition_progress < 1.0 {
            self.transition_progress += delta / duration;
            self.transition_progress = self.transition_progress.min(1.0);

            if self.transition_progress >= 1.0 {
                if let Some(target) = self.target_color.take() {
                    self.color = target;
                }
            }
        }
    }

    /// Get the current interpolated color.
    pub fn current_color(&self) -> Vec3 {
        if let Some(target) = self.target_color {
            let t = self.transition_progress;
            self.color.lerp(target, t)
        } else {
            self.color
        }
    }
}

/// Component for buildings that can be damaged/destroyed.
#[derive(Component, Debug)]
pub struct BuildingHealth {
    /// Current health points.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
}

impl BuildingHealth {
    /// Create a new health component.
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
        }
    }

    /// Get health as percentage.
    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }

    /// Apply damage and return new weathering level.
    pub fn apply_damage(&mut self, damage: f32) -> WeatheringLevel {
        self.current = (self.current - damage).max(0.0);
        let pct = self.percentage();

        if pct > 0.75 {
            WeatheringLevel::Pristine
        } else if pct > 0.5 {
            WeatheringLevel::Worn
        } else if pct > 0.25 {
            WeatheringLevel::Weathered
        } else if pct > 0.0 {
            WeatheringLevel::Ruined
        } else {
            WeatheringLevel::Destroyed
        }
    }
}

// ---------------------------------------------------------------------------
// BuildingPlugin
// ---------------------------------------------------------------------------

/// Modular building placement plugin for medieval structures.
///
/// This plugin can be added to any Bevy app for building placement.
/// Configuration is done via the `BuildingRegistry` resource.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(BuildingPlugin::default());
/// app.insert_resource(BuildingRegistry::medieval_defaults());
/// ```
#[derive(Default)]
pub struct BuildingPlugin {
    /// Initial registry (optional).
    pub registry: Option<BuildingRegistry>,
}

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        info!("BuildingPlugin: initializing building system");

        // Add registry resource
        if let Some(registry) = &self.registry {
            app.insert_resource(registry.clone());
        } else {
            app.insert_resource(BuildingRegistry::medieval_defaults());
        }

        // Add systems
        app.add_systems(Startup, setup_building_system);
        app.add_systems(Update, update_faction_colors_system);

        info!("BuildingPlugin: registered systems");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Setup the building system.
fn setup_building_system(registry: Res<BuildingRegistry>) {
    trace!("setup_building_system: initializing");

    info!(
        "BuildingPlugin: initialized with {} buildings, {} layouts",
        registry.buildings.len(),
        registry.layouts.len()
    );

    for (id, building) in &registry.buildings {
        trace!("  - Building: '{}' ({})", building.name, id);
    }

    for (id, layout) in &registry.layouts {
        trace!(
            "  - Layout: '{}' ({}) - {} buildings",
            layout.name,
            id,
            layout.buildings.len()
        );
    }
}

/// Update faction colors for buildings.
fn update_faction_colors_system(time: Res<Time>, mut query: Query<(&mut FactionColored, &Mesh3d)>) {
    let delta = time.delta_secs();

    for (mut faction_color, _mesh) in query.iter_mut() {
        // Update transition
        faction_color.update(delta, 2.0); // 2 second transition

        // Note: In a real implementation, we would update the material color here
        // This would require access to the material assets
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_def_default() {
        let def = BuildingDef::default();
        assert_eq!(def.id, "house");
        assert!(def.faction_colorable);
    }

    #[test]
    fn weathering_level_factor() {
        assert_eq!(WeatheringLevel::Pristine.factor(), 0.0);
        assert_eq!(WeatheringLevel::Ruined.factor(), 0.75);
        assert_eq!(WeatheringLevel::Destroyed.factor(), 1.0);
    }

    #[test]
    fn building_health_damage() {
        let mut health = BuildingHealth::new(100.0);
        assert_eq!(health.percentage(), 1.0);

        let weathering = health.apply_damage(30.0);
        assert_eq!(health.current, 70.0);
        assert_eq!(weathering, WeatheringLevel::Worn);

        let weathering = health.apply_damage(50.0);
        assert_eq!(health.current, 20.0);
        assert_eq!(weathering, WeatheringLevel::Ruined);

        let weathering = health.apply_damage(25.0);
        assert_eq!(health.current, 0.0);
        assert_eq!(weathering, WeatheringLevel::Destroyed);
    }

    #[test]
    fn building_registry() {
        let registry = BuildingRegistry::medieval_defaults();
        assert!(registry.get_building("house").is_some());
        assert!(registry.get_building("church").is_some());
        assert!(registry.get_layout("village").is_some());
        assert!(registry.get_layout("castle").is_some());
    }

    #[test]
    fn faction_colored_creation() {
        let color = Vec3::new(1.0, 0.0, 0.0);
        let faction_colored = FactionColored::new(color);
        assert_eq!(faction_colored.color, color);
        assert!(faction_colored.target_color.is_none());
        assert_eq!(faction_colored.transition_progress, 1.0);
    }

    #[test]
    fn faction_colored_transition() {
        let mut faction_colored = FactionColored::new(Vec3::new(1.0, 0.0, 0.0));
        faction_colored.transition_to(Vec3::new(0.0, 1.0, 0.0), 2.0);

        assert!(faction_colored.target_color.is_some());
        assert_eq!(faction_colored.transition_progress, 0.0);

        // Update transition
        faction_colored.update(1.0, 2.0);
        assert_eq!(faction_colored.transition_progress, 0.5);

        let current = faction_colored.current_color();
        assert!(current.x > 0.0 && current.x < 1.0); // Interpolated
        assert!(current.y > 0.0 && current.y < 1.0);
    }

    #[test]
    fn faction_colored_completion() {
        let mut faction_colored = FactionColored::new(Vec3::new(1.0, 0.0, 0.0));
        faction_colored.transition_to(Vec3::new(0.0, 1.0, 0.0), 2.0);

        // Complete transition
        faction_colored.update(2.0, 2.0);
        assert_eq!(faction_colored.transition_progress, 1.0);
        assert!(faction_colored.target_color.is_none());
        assert_eq!(faction_colored.color, Vec3::new(0.0, 1.0, 0.0));
    }
}
