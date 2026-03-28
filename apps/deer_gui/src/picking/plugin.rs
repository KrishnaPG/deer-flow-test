//! [`PickingPlugin`] — registers picking messages, observers, and systems.
//!
//! Integrates Bevy's built-in `bevy_picking` with the world's `Selectable`
//! component and the HUD inspector. Entities must have both `Selectable`
//! and `Pickable` components to be clickable.

use bevy::log::info;
use bevy::prelude::*;

use crate::world::spatial::SpatialIndex;

use super::systems::{
    deselection_system, on_entity_clicked, selection_sync_system, selection_update_system,
    spatial_index_rebuild_system, EntityClicked, SelectionChanged,
};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the entity picking and selection subsystem.
///
/// * Inserts a [`SpatialIndex`] resource.
/// * Registers custom messages for click/selection change.
/// * Adds an observer for `Pointer<Click>` events.
/// * Adds systems: spatial rebuild → selection update → deselection → HUD sync.
pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        info!("PickingPlugin::build — registering picking resources and systems");

        app.init_resource::<SpatialIndex>()
            .add_message::<EntityClicked>()
            .add_message::<SelectionChanged>()
            .add_observer(on_entity_clicked)
            .add_systems(
                Update,
                (
                    spatial_index_rebuild_system,
                    selection_update_system,
                    deselection_system,
                    selection_sync_system,
                )
                    .chain(),
            );
    }
}
