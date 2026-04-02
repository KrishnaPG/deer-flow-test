use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentBody {
    pub action: String,
}
define_record!(
    IntentRecord,
    IntentBody,
    RecordFamily::Intent,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformBody {
    pub producer: String,
}
define_record!(
    TransformRecord,
    TransformBody,
    RecordFamily::Transform,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionBody {
    pub reason: String,
}
define_record!(
    ExclusionRecord,
    ExclusionBody,
    RecordFamily::Exclusion,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictBody {
    pub reason: String,
}
define_record!(
    ConflictRecord,
    ConflictBody,
    RecordFamily::Conflict,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionBody {
    pub strategy: String,
}
define_record!(
    ResolutionRecord,
    ResolutionBody,
    RecordFamily::Resolution,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCheckpointBody {
    pub label: String,
}
define_record!(
    ReplayCheckpointRecord,
    ReplayCheckpointBody,
    RecordFamily::ReplayCheckpoint,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DedupBody {
    pub dedup_key: String,
}
define_record!(
    DedupRecord,
    DedupBody,
    RecordFamily::Dedup,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchBody {
    pub batch_key: String,
}
define_record!(
    BatchRecord,
    BatchBody,
    RecordFamily::Batch,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchBody {
    pub branch_name: String,
}
define_record!(
    BranchRecord,
    BranchBody,
    RecordFamily::Branch,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionBody {
    pub version: String,
}
define_record!(
    VersionRecord,
    VersionBody,
    RecordFamily::Version,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOperationBody {
    pub op: String,
}
define_record!(
    WriteOperationRecord,
    WriteOperationBody,
    RecordFamily::WriteOperation,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);
