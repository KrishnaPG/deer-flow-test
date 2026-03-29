//! [`PickingPlugin`] — registers picking messages, observers, and systems.
//!
//! Integrates Bevy's built-in `bevy_picking` with the world's `Selectable`
//! component and the HUD inspector. Entities must have both `Selectable`
//! and `Pickable` components to be clickable.
//!
//! # Two-Phase Picking Pipeline
//!
//! The picking system uses a two-phase approach for deterministic selection:
//! 1. **Coarse phase** (`coarse_picking_system`): Uses spatial index to find candidates near click
//! 2. **Precise phase** (`precise_picking_system`): Selects the closest candidate by distance
//!
//! System ordering: spatial_rebuild → coarse → precise → selection → deselection → sync

use bevy::log::info;
use bevy::prelude::*;

use crate::world::spatial::SpatialIndex;

use super::systems::{
    coarse_picking_system, deselection_system, on_entity_clicked, precise_picking_system,
    selection_change_logger_system, selection_sync_system, selection_update_system,
    spatial_index_rebuild_system, EntityClicked, PickingCandidates, SelectionChanged,
};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the entity picking and selection subsystem.
///
/// * Inserts [`SpatialIndex`] and [`PickingCandidates`] resources.
/// * Registers custom messages for click/selection change.
/// * Adds an observer for `Pointer<Click>` events.
/// * Adds systems in chain: spatial rebuild → coarse → precise → selection → deselection → sync.
pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        info!("PickingPlugin::build — registering picking resources and systems");

        app.init_resource::<SpatialIndex>()
            .init_resource::<PickingCandidates>()
            .add_message::<EntityClicked>()
            .add_message::<SelectionChanged>()
            .add_observer(on_entity_clicked)
            .add_systems(
                Update,
                (
                    spatial_index_rebuild_system,
                    coarse_picking_system,
                    precise_picking_system,
                    selection_update_system,
                    deselection_system,
                    selection_change_logger_system,
                    selection_sync_system,
                )
                    .chain(),
            );
    }
}
