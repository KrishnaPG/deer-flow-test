use deer_runtime_world_projection::{WorldObject, WorldProjection};
use deer_view_scene3d::{build_world_scene_vm, render_actor_cloud, SceneAnchor};
use insta::assert_yaml_snapshot;

fn scene_anchor_id(anchor: &SceneAnchor) -> &str {
    &anchor.source_record_id
}

#[test]
fn world_scene_vm_exposes_generic_scene_anchors() {
    let projection = WorldProjection {
        objects: vec![WorldObject::task_beacon("task_1", "task_detail")],
    };

    let scene = build_world_scene_vm(&projection);
    let anchor = &scene.anchors[0];

    assert_eq!(scene.anchors.len(), 1);
    assert_eq!(scene_anchor_id(anchor), "task_1");
    assert_eq!(anchor.kind, "WorldTaskBeacon");
    assert_eq!(anchor.drill_down_target, "task_detail");
    assert!(anchor.pickable);
    assert_eq!(anchor.position.x, 0.0);
}

#[test]
fn actor_cloud_renders_projected_world_objects_as_pickable_scene_anchors() {
    let projection = WorldProjection {
        objects: vec![
            WorldObject::task_beacon("task_1", "task_detail"),
            WorldObject::artifact_unlock("artifact_1", "artifact_detail"),
        ],
    };

    let rendered = render_actor_cloud(&projection);

    assert_yaml_snapshot!(rendered, @r#"
object_count: 2
anchors:
  - source_record_id: task_1
    kind: WorldTaskBeacon
    drill_down_target: task_detail
    pickable: true
    position:
      x: 0
      y: 0
      z: 0
  - source_record_id: artifact_1
    kind: WorldArtifactUnlock
    drill_down_target: artifact_detail
    pickable: true
    position:
      x: 2.5
      y: 0
      z: 0
"#);
}
