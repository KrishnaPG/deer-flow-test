use deer_runtime_world_projection::{WorldObject, WorldProjection};
use deer_view_scene3d::{emit_world_pick, SceneAnchor, SceneHost, SpatialIndex, SpatialRay, Vec3};

#[test]
fn world_pick_emits_selection_and_focus_without_arming_command() {
    let projection = WorldProjection {
        objects: vec![WorldObject::task_beacon("task_1", "task_detail")],
    };
    let host = SceneHost::from_projection("world", &projection);
    let ray = SpatialRay::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    let event = emit_world_pick(host.spatial_index(), ray).expect("expected ray hit");

    assert_eq!(event.selection_id, "task_1");
    assert_eq!(event.focus_target, "task_detail");
    assert!(!event.command_armed);
}

#[test]
fn world_pick_returns_none_when_ray_misses_all_anchors() {
    let projection = WorldProjection {
        objects: vec![WorldObject::task_beacon("task_1", "task_detail")],
    };
    let host = SceneHost::from_projection("world", &projection);
    let ray = SpatialRay::new(Vec3::new(10.0, 10.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    let event = emit_world_pick(host.spatial_index(), ray);

    assert!(event.is_none());
}

#[test]
fn world_pick_prefers_the_nearest_intersection() {
    let mut index = SpatialIndex::new();
    index.insert_anchor(SceneAnchor::pickable(
        "task_1",
        "WorldTaskBeacon",
        "task_detail",
        0.0,
        0.0,
        0.0,
    ));
    index.insert_anchor(SceneAnchor::pickable(
        "artifact_1",
        "WorldArtifactUnlock",
        "artifact_detail",
        0.0,
        0.0,
        3.0,
    ));
    let ray = SpatialRay::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    let event = emit_world_pick(&index, ray).expect("expected nearest hit");

    assert_eq!(event.selection_id, "task_1");
    assert_eq!(event.focus_target, "task_detail");
    assert!(!event.command_armed);
}
