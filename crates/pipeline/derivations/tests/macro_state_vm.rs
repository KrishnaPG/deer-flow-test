use deer_foundation_contracts::{AsIsHash, IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{AnyRecord, ArtifactBody, ArtifactRecord, TaskBody, TaskRecord};
use deer_pipeline_derivations::derive_macro_state_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_macro_state_rows_with_backlinks_and_support_metadata() {
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

    let macro_state = derive_macro_state_vm(&records);

    assert_yaml_snapshot!(macro_state, @r#"
rows:
  - kind: task
    source_record_id: task_1
    panel_target: task_detail
  - kind: artifact
    source_record_id: artifact_1
    panel_target: artifact_detail
backlinks:
  - source_record_id: task_1
    level: L2
    plane: AsIs
    panel_target: task_detail
  - source_record_id: artifact_1
    level: L2
    plane: AsIs
    panel_target: artifact_detail
"#);
}
