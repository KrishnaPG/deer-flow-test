use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsIsRepresentationBody {
    pub media_type: String,
}
define_record!(
    AsIsRepresentationRecord,
    AsIsRepresentationBody,
    RecordFamily::AsIsRepresentation,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkBody {
    pub chunk_index: u32,
    pub text: String,
}
define_record!(
    ChunkRecord,
    ChunkBody,
    RecordFamily::Chunk,
    CanonicalLevel::L2,
    CanonicalPlane::Chunks,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingBody {
    pub model: String,
    pub dimensions: usize,
}
define_record!(
    EmbeddingRecord,
    EmbeddingBody,
    RecordFamily::Embedding,
    CanonicalLevel::L2,
    CanonicalPlane::Embeddings,
    StorageDisposition::IndexProjection
);
