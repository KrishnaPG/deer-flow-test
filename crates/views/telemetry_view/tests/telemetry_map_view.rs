use deer_foundation_contracts::{IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{AnyRecord, TaskBody, TaskRecord};
use deer_runtime_world_projection::project_world_objects;
use deer_view_telemetry_view::render_telemetry_map_view;
use insta::assert_yaml_snapshot;

#[test]
fn telemetry_map_view_projects_world_objects_into_navigation_only_markers() {
    let projection = project_world_objects(&[AnyRecord::Task(TaskRecord::new(
        RecordId::from_static("task_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("task_1"), None, None, None),
        LineageMeta::root(),
        TaskBody {
            label: "Survey ridge".into(),
            status: "running".into(),
        },
    ))]);

    let rendered = render_telemetry_map_view(&projection, Some("task_1"));

    assert_yaml_snapshot!(rendered, @r#"
marker_count: 1
selected_marker_id: task_1
markers:
  - source_record_id: task_1
    kind: WorldTaskBeacon
    drill_down_target: task_detail
    selected: true
command_armed: false
"#);
}
