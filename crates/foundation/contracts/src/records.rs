use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CorrelationMeta, IdentityMeta, LineageMeta, RecordId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalPlane {
    AsIs,
    Chunks,
    Embeddings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageDisposition {
    StorageNative,
    IndexProjection,
    ClientTransient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordFamily {
    Run,
    Session,
    Task,
    Message,
    ToolCall,
    Artifact,
    Clarification,
    RuntimeStatus,
    Delivery,
    L0Source,
    L1Sanitized,
    L2View,
    L3Insight,
    L4Prediction,
    L5Prescription,
    L6Outcome,
    AsIsRepresentation,
    Chunk,
    Embedding,
    Intent,
    Transform,
    Exclusion,
    Conflict,
    Resolution,
    ReplayCheckpoint,
    Dedup,
    Batch,
    Branch,
    Version,
    WriteOperation,
    GraphNode,
    GraphEdge,
    KnowledgeEntity,
    KnowledgeRelation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub record_id: RecordId,
    pub family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub identity: IdentityMeta,
    pub correlations: CorrelationMeta,
    pub lineage: LineageMeta,
    pub observed_at: DateTime<Utc>,
}

impl RecordHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        record_id: RecordId,
        family: RecordFamily,
        level: CanonicalLevel,
        plane: CanonicalPlane,
        storage: StorageDisposition,
        identity: IdentityMeta,
        correlations: CorrelationMeta,
        lineage: LineageMeta,
    ) -> Self {
        Self {
            record_id,
            family,
            level,
            plane,
            storage,
            identity,
            correlations,
            lineage,
            observed_at: Utc::now(),
        }
    }
}

pub trait CanonicalRecord {
    fn header(&self) -> &RecordHeader;
}
