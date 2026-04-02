use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBody {
    pub title: String,
    pub status: String,
}
define_record!(
    RunRecord,
    RunBody,
    RecordFamily::Run,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBody {
    pub name: String,
}
define_record!(
    SessionRecord,
    SessionBody,
    RecordFamily::Session,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBody {
    pub label: String,
    pub status: String,
}
define_record!(
    TaskRecord,
    TaskBody,
    RecordFamily::Task,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageBody {
    pub role: String,
    pub text: String,
}
define_record!(
    MessageRecord,
    MessageBody,
    RecordFamily::Message,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallBody {
    pub tool_name: String,
    pub status: String,
}
define_record!(
    ToolCallRecord,
    ToolCallBody,
    RecordFamily::ToolCall,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactBody {
    pub label: String,
    pub media_type: String,
}
define_record!(
    ArtifactRecord,
    ArtifactBody,
    RecordFamily::Artifact,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarificationBody {
    pub prompt: String,
    pub resolved: bool,
}
define_record!(
    ClarificationRecord,
    ClarificationBody,
    RecordFamily::Clarification,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStatusBody {
    pub status: String,
    pub detail: String,
}
define_record!(
    RuntimeStatusRecord,
    RuntimeStatusBody,
    RecordFamily::RuntimeStatus,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryBody {
    pub channel: String,
    pub delivered: bool,
}
define_record!(
    DeliveryRecord,
    DeliveryBody,
    RecordFamily::Delivery,
    CanonicalLevel::L2,
    CanonicalPlane::AsIs,
    StorageDisposition::StorageNative
);
