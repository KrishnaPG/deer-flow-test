use deer_foundation_contracts::{
    AsIsHash, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordFamily, RecordHeader, RecordId, StorageDisposition,
};
use serde::{Deserialize, Serialize};

fn header(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    as_is_hash: Option<AsIsHash>,
) -> RecordHeader {
    let identity = IdentityMeta::hash_anchored(record_id.clone(), as_is_hash, None, None);

    RecordHeader::new(
        record_id,
        family,
        level,
        plane,
        StorageDisposition::ClientTransient,
        identity,
        CorrelationMeta::default(),
        LineageMeta::root(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    header: RecordHeader,
    title: String,
}

impl SessionRecord {
    pub fn new(record_id: RecordId, title: String) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Session,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            title,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRecord {
    header: RecordHeader,
    status: String,
}

impl RunRecord {
    pub fn new(record_id: RecordId, status: String) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Run,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            status,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRecord {
    header: RecordHeader,
    role: String,
    text: String,
}

impl MessageRecord {
    pub fn new(
        record_id: RecordId,
        role: String,
        text: String,
        level: CanonicalLevel,
        plane: CanonicalPlane,
    ) -> Self {
        Self {
            header: header(record_id, RecordFamily::Message, level, plane, None),
            role,
            text,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRecord {
    header: RecordHeader,
    title: String,
    state: String,
}

impl TaskRecord {
    pub fn new(record_id: RecordId, title: String, state: String) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Task,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            title,
            state,
        }
    }

    pub fn record_id(&self) -> &RecordId {
        &self.header.record_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    header: RecordHeader,
    name: String,
    status: String,
}

impl ArtifactRecord {
    pub fn new(
        record_id: RecordId,
        name: String,
        status: String,
        as_is_hash: Option<AsIsHash>,
    ) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Artifact,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                as_is_hash,
            ),
            name,
            status,
        }
    }

    pub fn record_id(&self) -> &RecordId {
        &self.header.record_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarificationRecord {
    header: RecordHeader,
    prompt: String,
}

impl ClarificationRecord {
    pub fn new(record_id: RecordId, prompt: String) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Clarification,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            prompt,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStatusRecord {
    header: RecordHeader,
    state: String,
}

impl RuntimeStatusRecord {
    pub fn new(state: String) -> Self {
        let record_id = RecordId::from(format!("runtime_status:{state}"));

        Self {
            header: header(
                record_id,
                RecordFamily::RuntimeStatus,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            state,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentRecord {
    header: RecordHeader,
    stage: String,
}

impl IntentRecord {
    pub fn new(record_id: RecordId, stage: String) -> Self {
        Self {
            header: header(
                record_id,
                RecordFamily::Intent,
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
                None,
            ),
            stage,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnyRecord {
    Session(SessionRecord),
    Run(RunRecord),
    Message(MessageRecord),
    Task(TaskRecord),
    Artifact(ArtifactRecord),
    Clarification(ClarificationRecord),
    RuntimeStatus(RuntimeStatusRecord),
    Intent(IntentRecord),
}
