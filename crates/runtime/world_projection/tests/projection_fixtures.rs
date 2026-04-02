use deer_foundation_domain::{AnyRecord, ArtifactRecord, TaskRecord};
use deer_runtime_world_projection::project_world_objects;
use insta::assert_yaml_snapshot;

#[test]
fn projects_task_and_artifact_records_into_world_objects_with_backlinks() {
    let records = vec![
        AnyRecord::Task(TaskRecord::new(
            "task_1".into(),
            "Gather terrain notes".into(),
            "running".into(),
        )),
        AnyRecord::Artifact(ArtifactRecord::new(
            "artifact_1".into(),
            "terrain.md".into(),
            "presented".into(),
            Some("sha256:terrain-md".into()),
        )),
    ];

    let projected = project_world_objects(&records);

    assert_yaml_snapshot!(projected, @r#"
objects:
  - kind: WorldTaskBeacon
    source_record_id: task_1
    drill_down_target: task_detail
  - kind: WorldArtifactUnlock
    source_record_id: artifact_1
    drill_down_target: artifact_detail
"#);
}
