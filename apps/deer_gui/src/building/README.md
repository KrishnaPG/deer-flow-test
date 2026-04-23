# Building Plugin

A modular building placement plugin for medieval structures in Bevy projects.

## Features

- **Modular Building Definitions**: Configurable via Bevy resources
- **Faction Coloring**: Dynamic color application with smooth transitions
- **Weathering Levels**: From pristine to destroyed states
- **Collision Shapes**: AABB collision detection for buildings
- **Layout Presets**: Predefined village, castle, and farm layouts
- **Grid Snapping**: Position buildings on a grid

## Quick Start

```rust
use bevy::prelude::*;
use deer_gui::building::*;

fn main() {
    App::new()
        .add_plugins(BuildingPlugin::default())
        .run();
}
```

## Core Types

### BuildingDef

Defines a building type with its properties:

```rust
use deer_gui::building::{BuildingDef, BuildingCategory, WeatheringLevel};
use bevy::math::Vec3;

let house = BuildingDef {
    id: "house".to_string(),
    name: "House".to_string(),
    category: BuildingCategory::Residential,
    model_path: "models/buildings/house.glb".to_string(),
    default_weathering: WeatheringLevel::Pristine,
    faction_colorable: true,
    collision_half_extents: Vec3::new(2.0, 2.0, 2.0),
    footprint: (2, 2),
    cost: [50, 100, 20, 10],
};
```

### BuildingCollider

Axis-aligned bounding box collision component:

```rust
use deer_gui::building::BuildingCollider;
use bevy::math::Vec3;

// Create a simple AABB collider
let collider = BuildingCollider::aabb(Vec3::new(2.0, 3.0, 2.5));

// With offset from entity position
let collider = BuildingCollider::aabb_with_offset(
    Vec3::new(2.0, 3.0, 2.5),
    Vec3::new(0.0, 1.0, 0.0),
);

// Query for point containment
let is_inside = collider.contains_point(&transform, point);

// AABB intersection test
let intersects = collider.intersects_aabb(&transform, other_min, other_max);
```

### BuildingSpawnConfig

Helper for creating building entities:

```rust
use deer_gui::building::BuildingSpawnConfig;
use bevy::math::Vec3;

let config = BuildingSpawnConfig::new(&house_def, (5, 10))
    .at_position(Vec3::new(100.0, 0.0, 200.0))
    .with_rotation(45.0)
    .with_faction("english".to_string(), Vec3::new(1.0, 0.0, 0.0));

// Spawn the building
commands.spawn((
    config.create_placed_building(),
    config.create_collider(),
    config.create_transform(),
    config.create_health(),
    // Add faction colored if applicable
));
```

## Components

### PlacedBuilding

Marker component for placed buildings:

```rust
#[derive(Component)]
pub struct PlacedBuilding {
    pub building_id: String,
    pub weathering: WeatheringLevel,
    pub grid_pos: (i32, i32),
    pub faction_id: Option<String>,
}
```

### FactionColored

Component for faction color transitions:

```rust
#[derive(Component)]
pub struct FactionColored {
    pub color: Vec3,
    pub target_color: Option<Vec3>,
    pub transition_progress: f32,
}

impl FactionColored {
    pub fn new(color: Vec3) -> Self;
    pub fn transition_to(&mut self, new_color: Vec3, duration: f32);
    pub fn current_color(&self) -> Vec3;
}
```

### BuildingHealth

Component for building durability:

```rust
#[derive(Component)]
pub struct BuildingHealth {
    pub current: f32,
    pub max: f32,
}

impl BuildingHealth {
    pub fn new(max_health: f32) -> Self;
    pub fn percentage(&self) -> f32;
    pub fn apply_damage(&mut self, damage: f32) -> WeatheringLevel;
}
```

## Resources

### BuildingRegistry

Registry of all building definitions and layouts:

```rust
use deer_gui::building::BuildingRegistry;

// Create with default medieval buildings
let registry = BuildingRegistry::medieval_defaults();

// Or create custom registry
let mut registry = BuildingRegistry::new();
registry.register_building(my_building_def);
registry.register_layout(my_layout_preset);
```

## Layout Presets

Predefined building layouts:

```rust
use deer_gui::building::presets;

let village_layout = presets::village();
let castle_layout = presets::castle();
let farm_layout = presets::farm();
```

## Weathering Levels

Building condition states:

```rust
use deer_gui::building::WeatheringLevel;

let pristine = WeatheringLevel::Pristine;  // 0.0 factor
let worn = WeatheringLevel::Worn;          // 0.25 factor
let weathered = WeatheringLevel::Weathered; // 0.5 factor
let ruined = WeatheringLevel::Ruined;      // 0.75 factor
let destroyed = WeatheringLevel::Destroyed; // 1.0 factor
```

## Building Categories

```rust
use deer_gui::building::BuildingCategory;

let residential = BuildingCategory::Residential;
let military = BuildingCategory::Military;
let religious = BuildingCategory::Religious;
let economic = BuildingCategory::Economic;
let agricultural = BuildingCategory::Agricultural;
```

## Reusability

This plugin can be extracted to a standalone crate for use in any Bevy project needing building placement. Configuration is done via Bevy resources (data-driven, no hardcoded paths).

### Dependencies

- `bevy` - Core ECS and rendering
- `serde` - Serialization for building definitions

### Integration

To integrate with other projects:

1. Add the `BuildingPlugin` to your app
2. Insert a `BuildingRegistry` with your building definitions
3. Use `BuildingSpawnConfig` to spawn buildings
4. Query for `BuildingCollider` for collision detection
