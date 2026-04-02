use deer_foundation_contracts::{
    AsIsHash, ChunkHash, EmbeddingBasisHash, IdentityMeta, LineageMeta, RecordFamily, RecordId,
    RecordRef,
};
use deer_foundation_domain::{
    AnyRecord, AsIsRepresentationBody, AsIsRepresentationRecord, ChunkBody, ChunkRecord,
    EmbeddingBody, EmbeddingRecord,
};
use serde::Deserialize;

use crate::error::NormalizationError;

#[derive(Debug, Deserialize)]
struct RepresentationFixture {
    as_is_record_id: String,
    as_is_hash: String,
    chunk_hash: String,
    embedding_basis_hash: String,
}

fn require_hash_anchor(value: &str, family: &'static str) -> Result<(), NormalizationError> {
    if value.is_empty() {
        Err(NormalizationError::MissingHashAnchor { family })
    } else {
        Ok(())
    }
}

pub fn normalize_representation_chain(
    fixture_json: &str,
) -> Result<Vec<AnyRecord>, NormalizationError> {
    let fixture: RepresentationFixture = serde_json::from_str(fixture_json)
        .map_err(|_| NormalizationError::UnsupportedEventKind("representation_fixture".into()))?;

    require_hash_anchor(&fixture.as_is_hash, "as_is")?;
    require_hash_anchor(&fixture.chunk_hash, "chunk")?;
    require_hash_anchor(&fixture.embedding_basis_hash, "embedding")?;

    let as_is_hash = AsIsHash::from(fixture.as_is_hash.as_str());
    let chunk_hash = ChunkHash::from(fixture.chunk_hash.as_str());
    let embedding_basis_hash = EmbeddingBasisHash::from(fixture.embedding_basis_hash.as_str());

    let as_is_record_id = RecordId::from(fixture.as_is_record_id.as_str());
    let chunk_record_id = RecordId::from("chunk_1");
    let embedding_record_id = RecordId::from("embedding_1");

    Ok(vec![
        AnyRecord::AsIsRepresentation(AsIsRepresentationRecord::new(
            as_is_record_id.clone(),
            IdentityMeta::hash_anchored(
                as_is_record_id.clone(),
                Some(as_is_hash.clone()),
                None,
                None,
            ),
            LineageMeta::root(),
            AsIsRepresentationBody {
                media_type: "application/octet-stream".into(),
            },
        )),
        AnyRecord::Chunk(ChunkRecord::new(
            chunk_record_id.clone(),
            IdentityMeta::hash_anchored(
                chunk_record_id.clone(),
                Some(as_is_hash.clone()),
                Some(chunk_hash.clone()),
                None,
            ),
            LineageMeta::derived_from(RecordRef::new(
                RecordFamily::AsIsRepresentation,
                as_is_record_id.clone(),
            )),
            ChunkBody {
                chunk_index: 0,
                text: String::new(),
            },
        )),
        AnyRecord::Embedding(EmbeddingRecord::new(
            embedding_record_id.clone(),
            IdentityMeta::hash_anchored(
                embedding_record_id,
                Some(as_is_hash),
                Some(chunk_hash),
                Some(embedding_basis_hash),
            ),
            LineageMeta::derived_from(RecordRef::new(RecordFamily::Chunk, chunk_record_id)),
            EmbeddingBody {
                model: "embedding-model".into(),
                dimensions: 0,
            },
        )),
    ])
}
