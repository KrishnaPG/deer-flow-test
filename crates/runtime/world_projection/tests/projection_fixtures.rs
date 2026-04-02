use deer_foundation_contracts::{AsIsHash, IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{AnyRecord, ArtifactBody, ArtifactRecord, TaskBody, TaskRecord};
use deer_runtime_world_projection::project_world_objects;
use insta::assert_yaml_snapshot;

#[test]
fn projects_task_and_artifact_records_into_world_objects_with_backlinks() {
    let records = vec![
        AnyRecord::Task(TaskRecord::new(
            RecordId::from_static("task_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("task_1"), None, None, None),
            LineageMeta::root(),
            TaskBody {
                label: "Gather terrain notes".into(),
                status: "running".into(),
            },
        )),
        AnyRecord::Artifact(ArtifactRecord::new(
            RecordId::from_static("artifact_1"),
            IdentityMeta::hash_anchored(
                RecordId::from_static("artifact_1"),
                Some(AsIsHash::from_static("sha256:terrain-md")),
                None,
                None,
            ),
            LineageMeta::root(),
            ArtifactBody {
                label: "terrain.md".into(),
                media_type: "text/markdown".into(),
            },
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
