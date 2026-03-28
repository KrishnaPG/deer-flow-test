//! Entity picking module — 3D selection via Bevy's built-in picking.
//!
//! Uses `bevy::picking` for mesh-based raycasting. When an entity
//! with a `Selectable` component is clicked, the picking system
//! updates the `Selected` marker component and syncs the inspector
//! data to `HudState`.

mod plugin;
mod systems;

pub use plugin::PickingPlugin;
