use deer_foundation_contracts::{
    AsIsHash, IdentityMeta, LineageMeta, RecordFamily, RecordId, RecordRef,
};
use deer_foundation_domain::{AnyRecord, ArtifactBody, ArtifactRecord};
use deer_runtime_world_projection::{project_world_objects, tombstone_visible};

#[test]
fn exclusions_control_tombstone_visibility() {
    assert!(tombstone_visible(true));
    assert!(!tombstone_visible(false));
}

#[test]
fn projected_objects_preserve_canonical_supersession_backlinks() {
    let lineage = LineageMeta {
        supersedes: Some(RecordRef::new(
            RecordFamily::Artifact,
            RecordId::from_static("artifact_0"),
        )),
        ..LineageMeta::root()
    };
    let records = vec![AnyRecord::Artifact(ArtifactRecord::new(
        RecordId::from_static("artifact_1"),
        IdentityMeta::hash_anchored(
            RecordId::from_static("artifact_1"),
            Some(AsIsHash::from_static("sha256:artifact-1")),
            None,
            None,
        ),
        lineage,
        ArtifactBody {
            label: "terrain.md".into(),
            media_type: "text/markdown".into(),
        },
    ))];

    let projected = project_world_objects(&records);

    assert_eq!(projected.objects.len(), 1);
    assert_eq!(
        projected.objects[0].supersedes_record_id.as_deref(),
        Some("artifact_0")
    );
}
