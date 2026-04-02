use serde::{Deserialize, Serialize};

use crate::{
    ArtifactId, AsIsHash, ChunkHash, EmbeddingBasisHash, EventId, MissionId, RecordFamily,
    RecordId, RunId, TaskId, ThreadId, TraceId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRef {
    pub family: RecordFamily,
    pub record_id: RecordId,
}

impl RecordRef {
    pub fn new(family: RecordFamily, record_id: RecordId) -> Self {
        Self { family, record_id }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMeta {
    pub primary_id: RecordId,
    pub as_is_hash: Option<AsIsHash>,
    pub chunk_hash: Option<ChunkHash>,
    pub embedding_basis_hash: Option<EmbeddingBasisHash>,
}

impl IdentityMeta {
    pub fn hash_anchored(
        primary_id: RecordId,
        as_is_hash: Option<AsIsHash>,
        chunk_hash: Option<ChunkHash>,
        embedding_basis_hash: Option<EmbeddingBasisHash>,
    ) -> Self {
        Self {
            primary_id,
            as_is_hash,
            chunk_hash,
            embedding_basis_hash,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMeta {
    pub mission_id: Option<MissionId>,
    pub run_id: Option<RunId>,
    pub artifact_id: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub thread_id: Option<ThreadId>,
    pub task_id: Option<TaskId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageMeta {
    pub derived_from: Vec<RecordRef>,
    pub source_events: Vec<EventId>,
    pub supersedes: Option<RecordRef>,
}

impl LineageMeta {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn derived_from(parent: RecordRef) -> Self {
        Self {
            derived_from: vec![parent],
            source_events: Vec::new(),
            supersedes: None,
        }
    }
}
