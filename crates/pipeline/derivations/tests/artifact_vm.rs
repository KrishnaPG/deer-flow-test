use deer_foundation_contracts::{AsIsHash, IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{AnyRecord, ArtifactBody, ArtifactRecord};
use deer_pipeline_derivations::derive_artifact_shelf_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_artifact_shelf_vm_with_preview_and_retrieval_metadata() {
    let records = vec![AnyRecord::Artifact(ArtifactRecord::new(
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
    ))];

    let shelf = derive_artifact_shelf_vm(&records);

    assert_yaml_snapshot!(shelf, @r#"
entries:
  - artifact_id: artifact_1
    title: terrain.md
    status: presented
    preview_supported: true
    retrieval_mode: mediated_pointer
"#);
}
