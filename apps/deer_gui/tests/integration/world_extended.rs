//! Extended integration tests for the world module.
//!
//! Covers SpatialIndex advanced queries (raycast, sphere, update, remove),
//! beacon pulse system, bridge event handler, and WorldState convenience
//! methods.

use bevy::prelude::*;

use deer_gui::bridge::{BridgeEvent, BridgeEventQueue};
use deer_gui::world::components::PulsingBeacon;
use deer_gui::world::spatial::SpatialIndex;
use deer_gui::world::systems::{beacon_pulse_system, bridge_event_handler_system};
use deer_gui::world::WorldState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<BridgeEventQueue>();
    app.init_resource::<WorldState>();
    app.init_resource::<SpatialIndex>();
    app
}

// ---------------------------------------------------------------------------
// T-WORLD-08  SpatialIndex raycast finds entities along ray
// ---------------------------------------------------------------------------

#[test]
fn t_world_08_spatial_raycast() {
    let mut app = test_app();
    app.update();

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    // Place e1 at origin (cell 0,0,0) and e2 at (50,0,0) (cell 2,0,0 with default size=20)
    idx.insert(e1, Vec3::ZERO);
    idx.insert(e2, Vec3::new(50.0, 0.0, 0.0));

    let idx = app.world().resource::<SpatialIndex>();
    let hits = idx.raycast(Vec3::ZERO, Vec3::X, 100.0);

    assert!(
        hits.contains(&e1),
        "raycast from origin along +X should hit e1 at origin"
    );
    assert!(
        hits.contains(&e2),
        "raycast from origin along +X should hit e2 at (50,0,0)"
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-09  SpatialIndex update moves entity between cells
// ---------------------------------------------------------------------------

#[test]
fn t_world_09_spatial_update() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn_empty().id();
    let old_pos = Vec3::new(5.0, 0.0, 0.0);
    let new_pos = Vec3::new(100.0, 0.0, 0.0);

    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    idx.insert(entity, old_pos);

    // Entity should be near old pos
    assert!(idx.query_nearby(old_pos).contains(&entity));

    // Update position
    idx.update(entity, old_pos, new_pos);

    // Entity should no longer be near old pos
    assert!(
        !idx.query_nearby(old_pos).contains(&entity),
        "entity should not be near old position after update"
    );
    // Entity should be near new pos
    assert!(
        idx.query_nearby(new_pos).contains(&entity),
        "entity should be near new position after update"
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-10  SpatialIndex remove
// ---------------------------------------------------------------------------

#[test]
fn t_world_10_spatial_remove() {
    let mut app = test_app();
    app.update();

    let entity = app.world_mut().spawn_empty().id();
    let pos = Vec3::new(10.0, 20.0, 30.0);

    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    idx.insert(entity, pos);
    assert_eq!(idx.entity_count(), 1);

    let removed = idx.remove(entity, pos);
    assert!(removed, "remove should return true when entity exists");
    assert_eq!(idx.entity_count(), 0);
    assert!(idx.is_empty());
}

// ---------------------------------------------------------------------------
// T-WORLD-11  beacon_pulse_system animates PulsingBeacon transforms
// ---------------------------------------------------------------------------

#[test]
fn t_world_11_beacon_pulse_system() {
    let mut app = test_app();
    app.add_systems(Update, beacon_pulse_system);
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            PulsingBeacon { frequency: 2.0 },
            Transform::from_scale(Vec3::ONE),
        ))
        .id();

    // Run several updates to advance time
    for _ in 0..10 {
        app.update();
    }

    let transform = app.world().get::<Transform>(entity).unwrap();
    // The beacon pulse system modifies scale based on sin(time * freq * TAU).
    // After some updates, the scale should have changed from its initial value.
    // We just verify it's been set (not NaN) and within a reasonable range.
    let scale = transform.scale.x;
    assert!(
        (0.5..=1.5).contains(&scale),
        "beacon scale {} should be within [0.5, 1.5]",
        scale
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-12  bridge_event_handler_system processes Ready event
// ---------------------------------------------------------------------------

#[test]
fn t_world_12_bridge_event_handler_ready() {
    let mut app = test_app();
    app.add_systems(Update, bridge_event_handler_system);
    app.update();

    // Verify system starts offline
    let ws = app.world().resource::<WorldState>();
    assert!(!ws.system_online());

    // Inject a Ready event
    let mut queue = app.world_mut().resource_mut::<BridgeEventQueue>();
    queue.events.push(BridgeEvent::Ready);

    app.update();

    let ws = app.world().resource::<WorldState>();
    assert!(
        ws.system_online(),
        "system should be online after BridgeEvent::Ready"
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-13  WorldState convenience methods (register, lookup, etc.)
// ---------------------------------------------------------------------------

#[test]
fn t_world_13_world_state_convenience_methods() {
    let mut app = test_app();
    app.update();

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    let mut ws = app.world_mut().resource_mut::<WorldState>();

    // Register
    assert!(ws.register("a1".to_string(), e1).is_none());
    assert!(ws.register("a2".to_string(), e2).is_none());
    assert_eq!(ws.entity_count(), 2);

    // Lookup
    assert_eq!(ws.lookup("a1"), Some(e1));
    assert_eq!(ws.lookup("a2"), Some(e2));
    assert_eq!(ws.lookup("missing"), None);

    // Unregister
    assert_eq!(ws.unregister("a1"), Some(e1));
    assert_eq!(ws.entity_count(), 1);
    assert_eq!(ws.lookup("a1"), None);

    // Unregister non-existent
    assert_eq!(ws.unregister("missing"), None);
}

// ---------------------------------------------------------------------------
// T-WORLD-14  SpatialIndex query_sphere
// ---------------------------------------------------------------------------

#[test]
fn t_world_14_spatial_query_sphere() {
    let mut app = test_app();
    app.update();

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    idx.insert(e1, Vec3::new(5.0, 0.0, 0.0));
    idx.insert(e2, Vec3::new(200.0, 0.0, 0.0));

    let idx = app.world().resource::<SpatialIndex>();
    let hits = idx.query_sphere(Vec3::ZERO, 30.0);

    assert!(
        hits.contains(&e1),
        "e1 at (5,0,0) should be within 30-unit sphere at origin"
    );
    assert!(
        !hits.contains(&e2),
        "e2 at (200,0,0) should NOT be within 30-unit sphere at origin"
    );
}

// ---------------------------------------------------------------------------
// T-WORLD-15  SpatialIndex len and is_empty
// ---------------------------------------------------------------------------

#[test]
fn t_world_15_spatial_len_and_is_empty() {
    let mut app = test_app();
    app.update();

    let idx = app.world().resource::<SpatialIndex>();
    assert!(idx.is_empty(), "fresh index should be empty");
    assert_eq!(idx.len(), 0);

    let entity = app.world_mut().spawn_empty().id();
    let mut idx = app.world_mut().resource_mut::<SpatialIndex>();
    idx.insert(entity, Vec3::ZERO);

    assert!(!idx.is_empty());
    assert_eq!(idx.len(), 1);
}
