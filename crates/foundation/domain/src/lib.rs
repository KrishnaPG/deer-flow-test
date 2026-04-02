pub mod carrier;
pub mod common;
pub mod governance;
pub mod representation;
pub mod semantic;
pub mod structural;

pub use carrier::*;
pub use governance::*;
pub use representation::*;
pub use semantic::*;
pub use structural::*;

use deer_foundation_contracts::{CanonicalRecord, RecordHeader};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnyRecord {
    Run(RunRecord),
    Session(SessionRecord),
    Task(TaskRecord),
    Message(MessageRecord),
    ToolCall(ToolCallRecord),
    Artifact(ArtifactRecord),
    Clarification(ClarificationRecord),
    RuntimeStatus(RuntimeStatusRecord),
    Delivery(DeliveryRecord),
    L0Source(L0SourceRecord),
    L1Sanitized(L1SanitizedRecord),
    L2View(L2ViewRecord),
    L3Insight(L3InsightRecord),
    L4Prediction(L4PredictionRecord),
    L5Prescription(L5PrescriptionRecord),
    L6Outcome(L6OutcomeRecord),
    AsIsRepresentation(AsIsRepresentationRecord),
    Chunk(ChunkRecord),
    Embedding(EmbeddingRecord),
    Intent(IntentRecord),
    Transform(TransformRecord),
    Exclusion(ExclusionRecord),
    Conflict(ConflictRecord),
    Resolution(ResolutionRecord),
    ReplayCheckpoint(ReplayCheckpointRecord),
    Dedup(DedupRecord),
    Batch(BatchRecord),
    Branch(BranchRecord),
    Version(VersionRecord),
    WriteOperation(WriteOperationRecord),
    GraphNode(GraphNodeRecord),
    GraphEdge(GraphEdgeRecord),
    KnowledgeEntity(KnowledgeEntityRecord),
    KnowledgeRelation(KnowledgeRelationRecord),
}

impl CanonicalRecord for AnyRecord {
    fn header(&self) -> &RecordHeader {
        match self {
            AnyRecord::Run(record) => &record.header,
            AnyRecord::Session(record) => &record.header,
            AnyRecord::Task(record) => &record.header,
            AnyRecord::Message(record) => &record.header,
            AnyRecord::ToolCall(record) => &record.header,
            AnyRecord::Artifact(record) => &record.header,
            AnyRecord::Clarification(record) => &record.header,
            AnyRecord::RuntimeStatus(record) => &record.header,
            AnyRecord::Delivery(record) => &record.header,
            AnyRecord::L0Source(record) => &record.header,
            AnyRecord::L1Sanitized(record) => &record.header,
            AnyRecord::L2View(record) => &record.header,
            AnyRecord::L3Insight(record) => &record.header,
            AnyRecord::L4Prediction(record) => &record.header,
            AnyRecord::L5Prescription(record) => &record.header,
            AnyRecord::L6Outcome(record) => &record.header,
            AnyRecord::AsIsRepresentation(record) => &record.header,
            AnyRecord::Chunk(record) => &record.header,
            AnyRecord::Embedding(record) => &record.header,
            AnyRecord::Intent(record) => &record.header,
            AnyRecord::Transform(record) => &record.header,
            AnyRecord::Exclusion(record) => &record.header,
            AnyRecord::Conflict(record) => &record.header,
            AnyRecord::Resolution(record) => &record.header,
            AnyRecord::ReplayCheckpoint(record) => &record.header,
            AnyRecord::Dedup(record) => &record.header,
            AnyRecord::Batch(record) => &record.header,
            AnyRecord::Branch(record) => &record.header,
            AnyRecord::Version(record) => &record.header,
            AnyRecord::WriteOperation(record) => &record.header,
            AnyRecord::GraphNode(record) => &record.header,
            AnyRecord::GraphEdge(record) => &record.header,
            AnyRecord::KnowledgeEntity(record) => &record.header,
            AnyRecord::KnowledgeRelation(record) => &record.header,
        }
    }
}
