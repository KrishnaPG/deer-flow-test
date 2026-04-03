//! Entity picking module — 3D selection via Bevy's built-in picking.
//!
//! Uses `bevy::picking` for mesh-based raycasting. When an entity
//! with a `Selectable` component is clicked, the picking system
//! updates the `Selected` marker component and syncs the inspector
//! data to `HudState`.
//!
//! The picking pipeline uses a two-phase approach:
//! 1. **Coarse phase**: Uses spatial index to find candidates near click
//! 2. **Precise phase**: Selects the closest candidate from the list

mod plugin;
pub mod systems;

pub use plugin::PickingPlugin;
pub use systems::{selection_change_logger_system, PickingCandidates, SelectionChanged};
