use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind,
};
use deer_storage_core::commit::{commit_external_ref, ExternalRefCommitRequest};

#[test]
fn external_ref_only_becomes_file_saved_after_canonical_promote() {
    let request = ExternalRefCommitRequest {
        source_uri: "s3://landing-zone/raw.wav".into(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L0,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("artifact"),
        format: StoragePayloadFormat::new("wav"),
        partitions: vec![("kind".into(), "Meeting Audio".into())],
        content_hash: "hash".into(),
    };

    let result = commit_external_ref(request).expect("external ref commit should succeed");
    assert!(result.promoted);
    assert_eq!(
        result.saved_relative_target,
        "b/L0/as-is/artifact/kind=meeting-audio/hash.wav"
    );
}

#[test]
fn external_ref_commit_rejects_missing_source_uri() {
    let request = ExternalRefCommitRequest {
        source_uri: String::new(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L0,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("artifact"),
        format: StoragePayloadFormat::new("wav"),
        partitions: vec![],
        content_hash: "hash".into(),
    };

    assert_eq!(commit_external_ref(request), Err("missing source uri"));
}
