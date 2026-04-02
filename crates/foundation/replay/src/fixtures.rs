use std::path::Path;

use deer_foundation_contracts::ReplayEnvelope;
use deer_foundation_domain::AnyRecord;
use serde::{Deserialize, Serialize};

use crate::{ReplayError, ReplayLog};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFixtureEntry {
    pub envelope: ReplayEnvelope,
    pub record: AnyRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFixture {
    pub entries: Vec<ReplayFixtureEntry>,
}

impl ReplayFixture {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ReplayError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path).map_err(|source| ReplayError::FixtureRead {
            path: path.display().to_string(),
            source,
        })?;

        serde_json::from_str(&raw).map_err(|source| ReplayError::FixtureParse {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn into_log(self) -> Result<ReplayLog, ReplayError> {
        let mut log = ReplayLog::default();

        for entry in self.entries {
            log.append(entry.envelope, entry.record)?;
        }

        Ok(log)
    }
}
