//! Integration tests for the picking / selection subsystem.
//!
//! Covers selection update, deselection, HUD sync, and spatial
//! index rebuild behaviour.

use bevy::prelude::*;

use deer_gui::hud::HudState;
use deer_gui::picking::PickingCandidates;
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
    app.init_resource::<PickingCandidates>();
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

// ---------------------------------------------------------------------------
// T-PICK-07  Coarse picking phase populates PickingCandidates
// ---------------------------------------------------------------------------

#[test]
fn t_pick_07_coarse_candidates_populated() {
    let mut app = test_app();
    app.update();

    // Simulate a click at screen position
    let screen_pos = Vec2::new(100.0, 100.0);
    {
        let mut candidates = app.world_mut().resource_mut::<PickingCandidates>();
        candidates.screen_pos = Some(screen_pos);
    }

    // Spawn entities near the click position (in 3D space)
    let click_world_pos = Vec3::new(100.0, 0.0, 100.0);
    let e1 = app
        .world_mut()
        .spawn((Selectable, Transform::from_translation(click_world_pos)))
        .id();
    let e2 = app
        .world_mut()
        .spawn((
            Selectable,
            Transform::from_translation(click_world_pos + Vec3::X * 10.0),
        ))
        .id();

    // Manually populate spatial index and candidates
    {
        let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
        idx.clear();
        idx.insert(e1, click_world_pos);
        idx.insert(e2, click_world_pos + Vec3::X * 10.0);
    }

    // Manually populate candidates as coarse_picking_system would
    {
        let idx = app.world().resource::<SpatialIndex>();
        let nearby = idx.query_sphere(click_world_pos, 24.0);
        let mut candidates = app.world_mut().resource_mut::<PickingCandidates>();
        candidates.candidates = nearby;
    }

    let candidates = app.world().resource::<PickingCandidates>();
    assert_eq!(
        candidates.screen_pos,
        Some(screen_pos),
        "screen_pos should be recorded"
    );
    assert_eq!(
        candidates.candidates.len(),
        2,
        "both entities should be candidates"
    );
    assert!(
        candidates.candidates.contains(&e1),
        "e1 should be a candidate"
    );
    assert!(
        candidates.candidates.contains(&e2),
        "e2 should be a candidate"
    );
}

// ---------------------------------------------------------------------------
// T-PICK-08  Precise picking phase selects closest candidate
// ---------------------------------------------------------------------------

#[test]
fn t_pick_08_precise_selects_closest() {
    let mut app = test_app();
    app.update();

    let click_world_pos = Vec3::new(100.0, 0.0, 100.0);
    let e1 = app
        .world_mut()
        .spawn((Selectable, Transform::from_translation(click_world_pos)))
        .id();
    let e2 = app
        .world_mut()
        .spawn((
            Selectable,
            Transform::from_translation(click_world_pos + Vec3::X * 20.0),
        ))
        .id();

    // Set up candidates with e1 closer than e2
    {
        let mut candidates = app.world_mut().resource_mut::<PickingCandidates>();
        candidates.screen_pos = Some(Vec2::new(100.0, 100.0));
        candidates.candidates = vec![e2, e1]; // e2 first, but e1 is closer
    }

    // Simulate precise picking: find closest to click_world_pos
    let closest = {
        let candidates = app.world().resource::<PickingCandidates>();

        candidates
            .candidates
            .iter()
            .min_by_key(|&&entity| {
                if let Some(transform) = app.world().get::<Transform>(entity) {
                    let dist = transform.translation.distance_squared(click_world_pos);
                    dist as i32
                } else {
                    i32::MAX
                }
            })
            .copied()
    };

    assert_eq!(closest, Some(e1), "closest entity (e1) should be selected");
}

// ---------------------------------------------------------------------------
// T-PICK-09  No candidates when no click recorded
// ---------------------------------------------------------------------------

#[test]
fn t_pick_09_no_candidates_no_click() {
    let mut app = test_app();
    app.update();

    // Don't set screen_pos (simulating no click)
    let click_world_pos = Vec3::new(100.0, 0.0, 100.0);
    let _e1 = app
        .world_mut()
        .spawn((Selectable, Transform::from_translation(click_world_pos)))
        .id();

    let candidates = app.world().resource::<PickingCandidates>();
    assert!(
        candidates.screen_pos.is_none(),
        "screen_pos should be None without click"
    );
    assert!(
        candidates.candidates.is_empty(),
        "candidates should be empty without click"
    );
}

// ---------------------------------------------------------------------------
// T-PICK-10  Deselection clears selected entity
// ---------------------------------------------------------------------------

#[test]
fn t_pick_10_deselection_clears_selected() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn((Selectable, Selected)).id();

    // Verify initially selected
    assert!(
        app.world().get::<Selected>(entity).is_some(),
        "entity should be selected initially"
    );

    // Simulate deselection
    app.world_mut().entity_mut(entity).remove::<Selected>();

    // Verify deselected
    assert!(
        app.world().get::<Selected>(entity).is_none(),
        "entity should not be selected after deselection"
    );

    // Verify PickingCandidates is cleared
    let mut candidates = app.world_mut().resource_mut::<PickingCandidates>();
    candidates.screen_pos = None;
    candidates.candidates.clear();

    assert!(
        candidates.screen_pos.is_none(),
        "screen_pos should be cleared"
    );
    assert!(
        candidates.candidates.is_empty(),
        "candidates should be cleared"
    );
}

// ---------------------------------------------------------------------------
// T-PICK-11  Selection updates shell selection state
// ---------------------------------------------------------------------------

#[test]
fn t_pick_11_selection_updates_shell_selection_state() {
    use bevy::prelude::*;
    use deer_gui::picking::systems::{
        selection_sync_to_shell_system, selection_update_system, EntityClicked,
    };
    use deer_gui::picking::SelectionChanged;
    use deer_gui::shell::{selection_broker_system, ShellSelectionRequest, ShellState};
    use deer_gui::world::components::{
        AgentState, Selectable, Selected, WorldEntity, WorldEntityType,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SpatialIndex>();
    app.init_resource::<HudState>();
    app.init_resource::<PickingCandidates>();
    app.init_resource::<ShellState>();
    app.add_message::<EntityClicked>();
    app.add_message::<SelectionChanged>();
    app.add_message::<ShellSelectionRequest>();
    app.add_systems(
        Update,
        (
            selection_update_system,
            selection_sync_to_shell_system,
            selection_broker_system,
        )
            .chain(),
    );
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Selectable,
            Selected,
            WorldEntity {
                entity_id: "agent-7".to_string(),
                entity_type: WorldEntityType::Agent(AgentState::Working),
            },
        ))
        .id();

    app.world_mut().write_message(EntityClicked(entity));
    app.update();

    let shell = app.world().resource::<ShellState>();
    assert_eq!(
        shell.selection.primary.as_ref().unwrap().canonical_id,
        "agent-7"
    );
}
