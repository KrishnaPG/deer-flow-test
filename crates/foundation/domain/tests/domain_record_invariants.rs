use deer_foundation_contracts::{
    AsIsHash, CanonicalLevel, CanonicalPlane, CanonicalRecord, ChunkHash, IdentityMeta,
    LineageMeta, RecordFamily, RecordId,
};
use deer_foundation_domain::{
    AnyRecord, EmbeddingBody, EmbeddingRecord, ExclusionBody, ExclusionRecord, RunBody, RunRecord,
};

#[test]
fn any_record_reports_expected_family_level_and_plane() {
    let run = RunRecord::new(
        RecordId::from_static("run_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("run_1"), None, None, None),
        LineageMeta::root(),
        RunBody {
            title: "mission alpha".into(),
            status: "running".into(),
        },
    );

    let record = AnyRecord::Run(run);
    assert_eq!(record.header().family, RecordFamily::Run);
    assert_eq!(record.header().level, CanonicalLevel::L2);
    assert_eq!(record.header().plane, CanonicalPlane::AsIs);
}

#[test]
fn embedding_records_require_hash_anchored_identity() {
    let record = EmbeddingRecord::new(
        RecordId::from_static("embed_1"),
        IdentityMeta::hash_anchored(
            RecordId::from_static("embed_1"),
            Some(AsIsHash::from_static("sha256:artifact")),
            Some(ChunkHash::from_static("sha256:chunk")),
            Some(deer_foundation_contracts::EmbeddingBasisHash::from_static(
                "sha256:basis",
            )),
        ),
        LineageMeta::root(),
        EmbeddingBody {
            model: "text-embedding-3-large".into(),
            dimensions: 3072,
        },
    );

    assert_eq!(
        record.header.identity.as_is_hash.unwrap().as_str(),
        "sha256:artifact"
    );
    assert_eq!(
        record.header.identity.chunk_hash.unwrap().as_str(),
        "sha256:chunk"
    );
}

#[test]
fn exclusion_records_preserve_append_only_lineage() {
    let exclusion = ExclusionRecord::new(
        RecordId::from_static("exclusion_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("exclusion_1"), None, None, None),
        LineageMeta::derived_from(deer_foundation_contracts::RecordRef::new(
            RecordFamily::Artifact,
            RecordId::from_static("artifact_9"),
        )),
        ExclusionBody {
            reason: "policy hidden".into(),
        },
    );

    assert_eq!(exclusion.header.lineage.derived_from.len(), 1);
    assert_eq!(
        exclusion.header.lineage.derived_from[0].record_id.as_str(),
        "artifact_9"
    );
}
