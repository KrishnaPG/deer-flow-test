//! Picking systems — selection management and HUD sync.
//!
//! Handles entity selection via Bevy's built-in picking events,
//! manages the `Selected` marker component, rebuilds the spatial
//! index, and syncs selection data to the HUD inspector.

use bevy::log::{debug, info, trace};
use bevy::picking::prelude::*;
use bevy::prelude::*;

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
// Click observer handler
// ---------------------------------------------------------------------------

/// Observer callback for `Pointer<Click>` events on selectable entities.
///
/// This function is registered as an observer and fires when any entity
/// with `Selectable` + `Pickable` receives a click.
pub fn on_entity_clicked(
    event: On<Pointer<Click>>,
    selectables: Query<(), With<Selectable>>,
    mut click_messages: MessageWriter<EntityClicked>,
) {
    let entity = event.event_target();
    if selectables.get(entity).is_ok() {
        debug!("on_entity_clicked — entity={:?}", entity);
        click_messages.write(EntityClicked(entity));
    }
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
