//! Integration tests for the picking / selection subsystem.
//!
//! Covers selection update, deselection, HUD sync, and spatial
//! index rebuild behaviour.

use bevy::prelude::*;

use deer_gui::hud::HudState;
use deer_gui::world::components::{AgentState, Selectable, Selected, WorldEntity, WorldEntityType};
use deer_gui::world::spatial::SpatialIndex;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal app with the resources that picking systems need,
/// without adding the full PickingPlugin (which requires bevy_picking).
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SpatialIndex>();
    app.init_resource::<HudState>();
    app
}

// ---------------------------------------------------------------------------
// T-PICK-01  Selectable component can be added and queried
// ---------------------------------------------------------------------------

#[test]
fn t_pick_01_selectable_component() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((Selectable, Transform::from_xyz(10.0, 0.0, 0.0)))
        .id();

    let has_selectable = app.world().get::<Selectable>(entity).is_some();
    assert!(has_selectable, "entity should have Selectable component");
}

// ---------------------------------------------------------------------------
// T-PICK-02  Selected marker can be toggled
// ---------------------------------------------------------------------------

#[test]
fn t_pick_02_selected_marker_toggle() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn((Selectable,)).id();

    // Initially not selected
    assert!(
        app.world().get::<Selected>(entity).is_none(),
        "should not be selected initially"
    );

    // Add Selected
    app.world_mut().entity_mut(entity).insert(Selected);
    assert!(
        app.world().get::<Selected>(entity).is_some(),
        "should be selected after insert"
    );

    // Remove Selected
    app.world_mut().entity_mut(entity).remove::<Selected>();
    assert!(
        app.world().get::<Selected>(entity).is_none(),
        "should not be selected after remove"
    );
}

// ---------------------------------------------------------------------------
// T-PICK-03  Only one entity should be selected at a time (manual)
// ---------------------------------------------------------------------------

#[test]
fn t_pick_03_single_selection() {
    let mut app = test_app();
    app.update();

    let e1 = app.world_mut().spawn((Selectable, Selected)).id();
    let e2 = app.world_mut().spawn((Selectable,)).id();

    // Manually simulate selection switch: deselect e1, select e2
    app.world_mut().entity_mut(e1).remove::<Selected>();
    app.world_mut().entity_mut(e2).insert(Selected);

    let selected_count = app
        .world_mut()
        .query_filtered::<Entity, With<Selected>>()
        .iter(app.world())
        .count();

    assert_eq!(selected_count, 1, "only one entity should be selected");
}

// ---------------------------------------------------------------------------
// T-PICK-04  HudState selected_entity defaults to None
// ---------------------------------------------------------------------------

#[test]
fn t_pick_04_hud_state_default_no_selection() {
    let mut app = test_app();
    app.update();

    let hud = app.world().resource::<HudState>();
    assert!(
        hud.selected_entity.is_none(),
        "HudState should have no selection by default"
    );
}

// ---------------------------------------------------------------------------
// T-PICK-05  HudState can be manually updated with selection data
// ---------------------------------------------------------------------------

#[test]
fn t_pick_05_hud_state_manual_sync() {
    let mut app = test_app();
    app.update();

    // Spawn a WorldEntity and simulate selection sync
    let _entity = app
        .world_mut()
        .spawn((
            WorldEntity {
                entity_id: "agent-7".to_string(),
                entity_type: WorldEntityType::Agent(AgentState::Working),
            },
            Selectable,
            Selected,
        ))
        .id();

    // Manually update HudState as the sync system would
    let mut hud = app.world_mut().resource_mut::<HudState>();
    hud.selected_entity = Some(deer_gui::hud::EntityInspectorData {
        entity_id: "agent-7".to_string(),
        display_name: "agent-7".to_string(),
        details: deer_gui::hud::InspectorDetails::Agent {
            state: "Working".to_string(),
            tokens_used: 0,
            context_size: 0,
            pending_actions: Vec::new(),
        },
    });

    let hud = app.world().resource::<HudState>();
    assert_eq!(hud.selected_entity.as_ref().unwrap().entity_id, "agent-7");
}

// ---------------------------------------------------------------------------
// T-PICK-06  SpatialIndex rebuild with Selectable entities
// ---------------------------------------------------------------------------

#[test]
fn t_pick_06_spatial_index_selectable_entities() {
    let mut app = test_app();
    app.update();

    let pos1 = Vec3::new(5.0, 0.0, 0.0);
    let pos2 = Vec3::new(50.0, 0.0, 0.0);

    let e1 = app
        .world_mut()
        .spawn((Selectable, Transform::from_translation(pos1)))
        .id();
    let e2 = app
        .world_mut()
        .spawn((Selectable, Transform::from_translation(pos2)))
        .id();

    // Simulate spatial_index_rebuild_system manually
    {
        let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
        idx.clear();
        idx.insert(e1, pos1);
        idx.insert(e2, pos2);
    }

    let idx = app.world().resource::<SpatialIndex>();
    assert_eq!(idx.entity_count(), 2, "both entities should be indexed");
    assert!(
        idx.query_nearby(pos1).contains(&e1),
        "e1 should be near pos1"
    );
    assert!(
        idx.query_nearby(pos2).contains(&e2),
        "e2 should be near pos2"
    );
}
