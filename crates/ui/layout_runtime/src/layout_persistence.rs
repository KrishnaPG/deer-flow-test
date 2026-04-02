use serde::Deserialize;
use thiserror::Error;

use crate::layout_model::{DockNode, LayoutSnapshot, CURRENT_LAYOUT_SNAPSHOT_VERSION};

#[derive(Debug, Error)]
pub enum LayoutPersistenceError {
    #[error("unsupported layout snapshot version {version}")]
    UnsupportedVersion { version: u32 },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl PartialEq for LayoutPersistenceError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::UnsupportedVersion { version: left },
                Self::UnsupportedVersion { version: right },
            ) => left == right,
            (Self::Json(left), Self::Json(right)) => left.to_string() == right.to_string(),
            _ => false,
        }
    }
}

impl Eq for LayoutPersistenceError {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LayoutSnapshotEnvelope {
    Current(LayoutSnapshot),
    Legacy(LegacyLayoutSnapshot),
}

#[derive(Debug, Deserialize)]
struct LegacyLayoutSnapshot {
    mode: String,
    panels: Vec<String>,
}

impl LegacyLayoutSnapshot {
    fn into_current(self) -> LayoutSnapshot {
        LayoutSnapshot::new(&self.mode, DockNode::tabs(self.panels), Vec::new())
    }
}

pub fn serialize_layout(snapshot: &LayoutSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string(snapshot)
}

pub fn deserialize_layout(encoded: &str) -> Result<LayoutSnapshot, LayoutPersistenceError> {
    let snapshot = match serde_json::from_str::<LayoutSnapshotEnvelope>(encoded)? {
        LayoutSnapshotEnvelope::Current(snapshot) => snapshot,
        LayoutSnapshotEnvelope::Legacy(snapshot) => snapshot.into_current(),
    };

    if snapshot.version != CURRENT_LAYOUT_SNAPSHOT_VERSION {
        return Err(LayoutPersistenceError::UnsupportedVersion {
            version: snapshot.version,
        });
    }

    Ok(snapshot)
}
