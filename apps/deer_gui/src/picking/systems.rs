//! Picking systems — selection management and HUD sync.
//!
//! Handles entity selection via Bevy's built-in picking events,
//! manages the `Selected` marker component, rebuilds the spatial
//! index, and syncs selection data to the HUD inspector.

use bevy::log::{debug, info, trace};
use bevy::picking::prelude::*;
use bevy::prelude::*;

use crate::constants::visual::PICKING_COARSE_RADIUS_PX;
use crate::hud::HudState;
use crate::world::components::{Selectable, Selected, WorldEntity};
use crate::world::spatial::SpatialIndex;

// ---------------------------------------------------------------------------
// Messages (frame-buffered inter-system communication)
// ---------------------------------------------------------------------------

/// Message emitted when a selectable entity is clicked.
#[derive(Message, Debug)]
pub struct EntityClicked(pub Entity);

/// Message emitted when the selection changes.
#[derive(Message, Debug)]
pub struct SelectionChanged {
    pub old: Option<Entity>,
    pub new: Option<Entity>,
}

// ---------------------------------------------------------------------------
// PickingCandidates resource (inter-phase communication)
// ---------------------------------------------------------------------------

/// Inter-phase resource for two-phase picking (coarse → precise).
///
/// The coarse phase records the screen position and populates candidate entities.
/// The precise phase selects the closest candidate and emits an `EntityClicked` message.
#[derive(Resource, Debug, Default)]
pub struct PickingCandidates {
    /// Screen position of the click (in pixels).
    pub screen_pos: Option<Vec2>,
    /// World position of the click (in 3D world space).
    pub world_pos: Option<Vec3>,
    /// Candidate entities found in the coarse phase.
    pub candidates: Vec<Entity>,
}

// ---------------------------------------------------------------------------
// Spatial index rebuild system
// ---------------------------------------------------------------------------

/// Rebuilds the [`SpatialIndex`] each frame from all entities
/// with `Transform` + `Selectable` components.
pub fn spatial_index_rebuild_system(
    mut index: ResMut<SpatialIndex>,
    query: Query<(Entity, &Transform), With<Selectable>>,
) {
    index.clear();
    for (entity, transform) in &query {
        index.insert(entity, transform.translation);
    }
    trace!(
        "spatial_index_rebuild_system — {} entities in {} cells",
        index.entity_count(),
        index.cell_count()
    );
}

// ---------------------------------------------------------------------------
// Click observer handler (Phase 1: Record click position)
// ---------------------------------------------------------------------------

/// Observer callback for `Pointer<Click>` events.
///
/// Records the click position for the two-phase picking pipeline.
/// The actual selection is handled by `coarse_picking_system` and `precise_picking_system`.
pub fn on_entity_clicked(event: On<Pointer<Click>>, mut candidates: ResMut<PickingCandidates>) {
    // For now, record that a click occurred
    // In a full implementation, we would get the hit position from the picking backend
    debug!("on_entity_clicked — entity={:?}", event.event_target());

    // Note: In this simplified version, we track that a click occurred
    // The actual world position would need to be determined by raycasting
    candidates.screen_pos = Some(Vec2::ZERO); // Placeholder
    candidates.world_pos = None; // Will be populated by raycast if needed
    candidates.candidates.clear();
}

// ---------------------------------------------------------------------------
// Coarse picking phase (Phase 2: Find candidate entities)
// ---------------------------------------------------------------------------

/// Coarse picking phase: finds candidate entities near the click position.
///
/// Uses the spatial index to find all selectable entities within a radius
/// of the clicked position. Populates `PickingCandidates::candidates`.
pub fn coarse_picking_system(
    mut candidates: ResMut<PickingCandidates>,
    index: Res<SpatialIndex>,
    selectables: Query<(), With<Selectable>>,
) {
    let Some(world_pos) = candidates.world_pos else {
        // If no world position is set, we can't do coarse picking
        return;
    };

    // Query spatial index for candidates within coarse radius
    let radius = PICKING_COARSE_RADIUS_PX;
    let found = index.query_sphere(world_pos, radius);

    // Filter to only selectable entities
    candidates.candidates = found
        .into_iter()
        .filter(|&entity| selectables.get(entity).is_ok())
        .collect();

    trace!(
        "coarse_picking_system — world_pos={:?} radius={} candidates={}",
        world_pos,
        radius,
        candidates.candidates.len()
    );
}

// ---------------------------------------------------------------------------
// Precise picking phase (Phase 3: Select closest candidate)
// ---------------------------------------------------------------------------

/// Precise picking phase: selects the closest candidate entity.
///
/// From the candidates populated by `coarse_picking_system`, finds the
/// closest entity to the click position and emits an `EntityClicked` message.
pub fn precise_picking_system(
    mut candidates: ResMut<PickingCandidates>,
    transforms: Query<&Transform, With<Selectable>>,
    mut click_messages: MessageWriter<EntityClicked>,
) {
    let Some(world_pos) = candidates.world_pos else {
        return;
    };

    if candidates.candidates.is_empty() {
        trace!("precise_picking_system — no candidates to evaluate");
        return;
    }

    // Find the closest candidate by squared distance
    let closest = candidates
        .candidates
        .iter()
        .filter_map(|&entity| {
            transforms.get(entity).ok().map(|transform| {
                let dist_sq = transform.translation.distance_squared(world_pos);
                (entity, dist_sq)
            })
        })
        .min_by(|(_, dist_a), (_, dist_b)| {
            dist_a
                .partial_cmp(dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _)| entity);

    if let Some(entity) = closest {
        debug!(
            "precise_picking_system — selected closest entity={:?}",
            entity
        );
        click_messages.write(EntityClicked(entity));
    }

    // Clear candidates for next frame
    candidates.candidates.clear();
    candidates.screen_pos = None;
    candidates.world_pos = None;
}

// ---------------------------------------------------------------------------
// Selection update system
// ---------------------------------------------------------------------------

/// Processes [`EntityClicked`] messages to update the `Selected` component.
///
/// Removes `Selected` from the previously selected entity (if any),
/// adds `Selected` to the newly clicked entity, and emits a
/// [`SelectionChanged`] message.
pub fn selection_update_system(
    mut commands: Commands,
    mut click_messages: MessageReader<EntityClicked>,
    mut change_messages: MessageWriter<SelectionChanged>,
    currently_selected: Query<Entity, With<Selected>>,
) {
    for EntityClicked(clicked) in click_messages.read() {
        let old = currently_selected.iter().next();

        // Deselect old
        if let Some(old_entity) = old {
            if old_entity == *clicked {
                trace!("selection_update_system — same entity clicked, no change");
                continue;
            }
            debug!("selection_update_system — deselecting {:?}", old_entity);
            commands.entity(old_entity).remove::<Selected>();
        }

        // Select new
        debug!("selection_update_system — selecting {:?}", clicked);
        commands.entity(*clicked).insert(Selected);

        change_messages.write(SelectionChanged {
            old,
            new: Some(*clicked),
        });
    }
}

// ---------------------------------------------------------------------------
// Deselection system (click on empty space)
// ---------------------------------------------------------------------------

/// Clears the selection when the user clicks on empty space.
///
/// Listens for mouse button presses and, if no `EntityClicked`
/// message was emitted this frame, deselects the current entity.
pub fn deselection_system(
    mut commands: Commands,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    click_messages: MessageReader<EntityClicked>,
    currently_selected: Query<Entity, With<Selected>>,
    mut change_messages: MessageWriter<SelectionChanged>,
) {
    // Gracefully no-op when InputPlugin is absent (e.g. headless / test).
    let Some(mouse) = mouse else { return };

    // Only act on fresh left-click
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // If an entity was clicked this frame, don't deselect
    if !click_messages.is_empty() {
        return;
    }

    // Deselect all
    for entity in &currently_selected {
        debug!("deselection_system — deselecting {:?}", entity);
        commands.entity(entity).remove::<Selected>();
        change_messages.write(SelectionChanged {
            old: Some(entity),
            new: None,
        });
    }
}

// ---------------------------------------------------------------------------
// HUD sync system
// ---------------------------------------------------------------------------

/// Syncs the currently selected entity's data to [`HudState::selected_entity`].
///
/// Reads the `Selected` entity's `WorldEntity` component and maps
/// its data into the HUD inspector format via [`crate::hud::adapters`].
pub fn selection_sync_system(
    mut hud: ResMut<HudState>,
    selected_query: Query<(Entity, &WorldEntity), With<Selected>>,
) {
    let selected = selected_query.iter().next();

    match selected {
        Some((_entity, world_entity)) => {
            let inspector_data = crate::hud::adapters::world_entity_to_inspector(world_entity);

            if hud
                .selected_entity
                .as_ref()
                .is_none_or(|e| e.entity_id != inspector_data.entity_id)
            {
                info!(
                    "selection_sync_system — updated inspector: {}",
                    inspector_data.entity_id
                );
            }

            hud.selected_entity = Some(inspector_data);
        }
        None => {
            if hud.selected_entity.is_some() {
                info!("selection_sync_system — cleared inspector");
                hud.selected_entity = None;
            }
        }
    }
}
