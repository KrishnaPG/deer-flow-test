use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{EventId, RecordId, RecordRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteOperationKind {
    AppendRecord,
    AppendGovernanceRecord,
    EmitCheckpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOperation {
    pub kind: WriteOperationKind,
    pub record_id: RecordId,
}

impl WriteOperation {
    pub fn new(kind: WriteOperationKind, record_id: RecordId) -> Self {
        Self { kind, record_id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCursor {
    pub sequence: u64,
    pub direction: ReplayDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEnvelope {
    pub sequence: u64,
    pub event_id: EventId,
    pub record_ref: RecordRef,
    pub write: WriteOperation,
    pub parent_event_id: Option<EventId>,
    pub occurred_at: DateTime<Utc>,
}

impl ReplayEnvelope {
    pub fn append(
        sequence: u64,
        event_id: EventId,
        record_ref: RecordRef,
        write: WriteOperation,
        parent_event_id: Option<EventId>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            sequence,
            event_id,
            record_ref,
            write,
            parent_event_id,
            occurred_at,
        }
    }

    pub fn cursor(&self) -> ReplayCursor {
        ReplayCursor {
            sequence: self.sequence,
            direction: ReplayDirection::Forward,
        }
    }
}
