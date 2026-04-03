//! Integration tests for the world module.
//!
//! Covers WorldState resource registration, entity registry CRUD,
//! and SpatialIndex insert / query / clear behaviour.

use bevy::prelude::*;

use deer_gui::bridge::BridgeEventQueue;
use deer_gui::world::spatial::SpatialIndex;
use deer_gui::world::WorldState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with WorldState and BridgeEventQueue
/// initialised directly (avoids spawning the real Python bridge).
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<BridgeEventQueue>();
    app.init_resource::<WorldState>();
    app
}

// ---------------------------------------------------------------------------
// T-WORLD-01  WorldState resource exists after startup
// ---------------------------------------------------------------------------

#[test]
fn t_world_01_world_state_registered() {
    let mut app = test_app();
    app.update();

    let world_state = app.world().resource::<WorldState>();
    assert!(
        !world_state.system_online(),
        "system should default to offline"
    );
    assert!(world_state.entity_registry.is_empty());
}

// ---------------------------------------------------------------------------
// T-WORLD-02  Entity registration round-trip
// ---------------------------------------------------------------------------

#[test]
fn t_world_02_entity_registration_round_trip() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn_empty().id();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.entity_registry.insert("agent-42".to_string(), entity);

    let ws = app.world().resource::<WorldState>();
    assert_eq!(ws.entity_registry.get("agent-42"), Some(&entity));
    assert_eq!(ws.entity_registry.len(), 1);
}

// ---------------------------------------------------------------------------
// T-WORLD-03  Entity unregistration
// ---------------------------------------------------------------------------

#[test]
fn t_world_03_entity_unregistration() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn_empty().id();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    ws.entity_registry.insert("agent-99".to_string(), entity);
    assert_eq!(ws.entity_registry.len(), 1);

    let removed = ws.entity_registry.remove("agent-99");
    assert_eq!(removed, Some(entity));
    assert!(ws.entity_registry.is_empty());
}

// ---------------------------------------------------------------------------
// T-WORLD-04  Unregister non-existent ID returns None
// ---------------------------------------------------------------------------

#[test]
fn t_world_04_unregister_nonexistent() {
    let mut app = test_app();
    app.update();

    let mut ws = app.world_mut().resource_mut::<WorldState>();
    let result = ws.entity_registry.remove("does-not-exist");
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// T-WORLD-05  SpatialIndex insert + query
// ---------------------------------------------------------------------------

#[test]
fn t_world_05_spatial_insert_and_query() {
    let mut app = test_app();
    app.insert_resource(SpatialIndex::default());
    app.update();

    let entity = app.world_mut().spawn_empty().id();

    let pos = Vec3::new(5.0, 0.0, 0.0);
    app.world_mut()
        .resource_mut::<SpatialIndex>()
        .insert(entity, pos);

    let idx = app.world().resource::<SpatialIndex>();
    let nearby = idx.query_nearby(pos);
    assert!(
        nearby.contains(&entity),
        "entity should be found near its position"
    );
    assert_eq!(idx.entity_count(), 1);
}

// ---------------------------------------------------------------------------
// T-WORLD-06  SpatialIndex out-of-range query
// ---------------------------------------------------------------------------

#[test]
fn t_world_06_spatial_out_of_range() {
    let mut app = test_app();
    // Use default cell_size = 20.0. An entity at (100, 0, 0) is in cell (5,0,0).
    // query_nearby at origin checks cells (-1..=1, -1..=1, -1..=1) — no overlap.
    app.insert_resource(SpatialIndex::default());
    app.update();

    let entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<SpatialIndex>()
        .insert(entity, Vec3::new(100.0, 0.0, 0.0));

    let idx = app.world().resource::<SpatialIndex>();
    let nearby = idx.query_nearby(Vec3::ZERO);
    assert!(
        !nearby.contains(&entity),
        "entity at (100,0,0) should not appear in query near origin"
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-07  SpatialIndex clear
// ---------------------------------------------------------------------------

#[test]
fn t_world_07_spatial_clear() {
    let mut app = test_app();
    app.insert_resource(SpatialIndex::default());
    app.update();

    let world = app.world_mut();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();

    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    idx.insert(e1, Vec3::new(1.0, 2.0, 3.0));
    idx.insert(e2, Vec3::new(50.0, 60.0, 70.0));
    assert_eq!(idx.entity_count(), 2);

    idx.clear();
    assert_eq!(idx.entity_count(), 0);
    assert_eq!(idx.cell_count(), 0);
}

// ---------------------------------------------------------------------------
// T-WORLD-08  WorldEntity maps to canonical ref
// ---------------------------------------------------------------------------

#[test]
fn t_world_08_world_entity_maps_to_canonical_ref() {
    use deer_gui::shell::{CanonicalEntityRef, CanonicalRecordFamily};
    use deer_gui::world::components::{ArtifactKind, WorldEntity, WorldEntityType};

    let world_entity = WorldEntity {
        entity_id: "mission-77".to_string(),
        entity_type: WorldEntityType::Mission { progress: 0.8 },
    };

    let canonical: CanonicalEntityRef = world_entity.to_canonical_ref();
    assert_eq!(canonical.family, CanonicalRecordFamily::Mission);
    assert_eq!(canonical.canonical_id, "mission-77");
}
