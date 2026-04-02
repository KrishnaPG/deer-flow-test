use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeBody {
    pub label: String,
}
define_record!(
    GraphNodeRecord,
    GraphNodeBody,
    RecordFamily::GraphNode,
    CanonicalLevel::L3,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeBody {
    pub label: String,
}
define_record!(
    GraphEdgeRecord,
    GraphEdgeBody,
    RecordFamily::GraphEdge,
    CanonicalLevel::L3,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntityBody {
    pub name: String,
}
define_record!(
    KnowledgeEntityRecord,
    KnowledgeEntityBody,
    RecordFamily::KnowledgeEntity,
    CanonicalLevel::L3,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRelationBody {
    pub relation: String,
}
define_record!(
    KnowledgeRelationRecord,
    KnowledgeRelationBody,
    RecordFamily::KnowledgeRelation,
    CanonicalLevel::L3,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);
