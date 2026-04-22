//! Building placement plugin for medieval structures.
//!
//! Provides a reusable [`BuildingPlugin`] that can be configured via
//! Bevy resources and supports faction coloring, weathering variants,
//! and layout presets.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Plugin that registers building-related systems and resources.
pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        // TODO: register resources, systems, and asset loaders.
        info!("BuildingPlugin: initialized");
    }
}

/// Definition of a building type, loaded from configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDef {
    /// Unique identifier for this building type.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Path to the glTF model.
    pub model_path: String,
    /// Default weathering level (0.0 = pristine, 1.0 = ruined).
    pub weathering: f32,
    /// Whether this building can be colored by faction themes.
    pub faction_colorable: bool,
    /// Collision shape dimensions (half-extents).
    pub collision_half_extents: Vec3,
}

impl Default for BuildingDef {
    fn default() -> Self {
        Self {
            id: "house".to_string(),
            name: "House".to_string(),
            model_path: "models/house.glb".to_string(),
            weathering: 0.0,
            faction_colorable: true,
            collision_half_extents: Vec3::new(2.0, 2.0, 2.0),
        }
    }
}
