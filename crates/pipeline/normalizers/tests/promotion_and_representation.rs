use deer_foundation_contracts::CanonicalLevel;
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{
    emit_intent_records, normalize_representation_chain, validate_promotion,
};

#[test]
fn representation_chain_requires_hash_anchors_and_preserves_backlinks() {
    let fixture = include_str!("fixtures/representation_chain.json");
    let chain = normalize_representation_chain(fixture).unwrap();

    assert!(matches!(chain[0], AnyRecord::AsIsRepresentation(_)));
    assert!(matches!(chain[1], AnyRecord::Chunk(_)));
    assert!(matches!(chain[2], AnyRecord::Embedding(_)));

    let AnyRecord::Chunk(chunk) = &chain[1] else {
        panic!("expected chunk record")
    };
    let AnyRecord::Embedding(embedding) = &chain[2] else {
        panic!("expected embedding record")
    };

    assert_eq!(chunk.header.lineage.derived_from.len(), 1);
    assert_eq!(
        chunk.header.lineage.derived_from[0].record_id.as_str(),
        "artifact_repr_1"
    );
    assert_eq!(embedding.header.lineage.derived_from.len(), 1);
    assert_eq!(
        embedding.header.lineage.derived_from[0].record_id.as_str(),
        "chunk_1"
    );
}

#[test]
fn rejects_invalid_direct_level_jump() {
    let error = validate_promotion(CanonicalLevel::L0, CanonicalLevel::L4).unwrap_err();
    assert!(error.to_string().contains("invalid level promotion"));
}

#[test]
fn submitted_is_the_only_stage_that_emits_intent_records() {
    let prefill_seed = emit_intent_records("prefill_seed").unwrap();
    let submitted = emit_intent_records("submitted").unwrap();

    assert!(prefill_seed.is_empty());
    assert_eq!(submitted.len(), 2);
    assert!(submitted
        .iter()
        .any(|record| matches!(record, AnyRecord::Intent(_))));
    assert!(submitted
        .iter()
        .any(|record| matches!(record, AnyRecord::WriteOperation(_))));
}
