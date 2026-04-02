use deer_foundation_contracts::{RecordId, ReplayCursor, ReplayEnvelope};
use deer_foundation_domain::AnyRecord;

use crate::ReplayError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayEntry {
    pub envelope: ReplayEnvelope,
    pub record: AnyRecord,
}

#[derive(Clone, Debug, Default)]
pub struct ReplayLog {
    entries: Vec<ReplayEntry>,
}

impl ReplayLog {
    pub fn append(
        &mut self,
        envelope: ReplayEnvelope,
        record: AnyRecord,
    ) -> Result<(), ReplayError> {
        if let Some(last) = self.entries.last().map(|entry| entry.envelope.sequence) {
            if envelope.sequence <= last {
                return Err(ReplayError::NonMonotonicSequence {
                    last,
                    next: envelope.sequence,
                });
            }
        }

        self.entries.push(ReplayEntry { envelope, record });
        Ok(())
    }

    pub fn entries(&self) -> &[ReplayEntry] {
        &self.entries
    }

    pub fn after(&self, cursor: Option<&ReplayCursor>) -> Vec<&ReplayEntry> {
        match cursor {
            Some(cursor) => self
                .entries
                .iter()
                .filter(|entry| entry.envelope.sequence > cursor.sequence)
                .collect(),
            None => self.entries.iter().collect(),
        }
    }

    pub fn latest_for(&self, record_id: &RecordId) -> Option<&ReplayEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| &entry.envelope.record_ref.record_id == record_id)
    }
}
