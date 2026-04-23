//! World module — ECS components, state, and systems for the world simulation.
//!
//! Submodules:
//! - [`state`] — `WorldState` / `SystemHealth` resources
//! - [`components`] — ECS components and marker types
//! - [`spatial`] — Grid-based spatial index
//! - [`systems`] — frame-update systems
//! - [`plugin`] — Bevy plugin registration
//! - [`foliage`] — GPU-instanced vegetation via bevy_feronia
//! - [`npc`] — NPC population with behavior trees via bevior_tree
//! - [`water`] — Water planes via bevy_water

pub mod components;
pub mod foliage;
pub mod npc;
mod plugin;
pub mod spatial;
pub mod state;
pub mod systems;
pub mod viewport;
pub mod water;

pub use foliage::{BiomeType, FoliagePlugin, FoliageType, WindConfig};
pub use npc::{AnimationState, NpcPlugin, NpcType};
pub use plugin::WorldPlugin;
pub use state::WorldState;
pub use water::{WaterBody, WaterGlobalConfig, WaterPlugin, WaterType};
