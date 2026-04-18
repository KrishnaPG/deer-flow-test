use async_trait::async_trait;
use bytes::Bytes;
use deer_foundation_paths::StagingDatabasePath;
use redb::{Database, TableDefinition};
use std::sync::Arc;

use crate::acp_client::ids::ChatSessionId;
use crate::acp_client::raw_publisher::{RawEventPublishError, RawEventPublisher, RawEventReader};

const EVENTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("hermes_l0_drop");

#[derive(Clone)]
pub struct RedbRawEventPublisher {
    db: Arc<Database>,
}

impl RedbRawEventPublisher {
    pub fn new(path: &StagingDatabasePath) -> Result<Self, String> {
        let db = Database::create(path.as_path())
            .map_err(|e| format!("Failed to open redb database: {}", e))?;
        
        // Ensure table exists
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        {
            let _ = write_txn.open_table(EVENTS_TABLE).map_err(|e| e.to_string())?;
        }
        write_txn.commit().map_err(|e| e.to_string())?;

        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl RawEventPublisher for RedbRawEventPublisher {
    async fn publish_raw_event(
        &self,
        session_id: &ChatSessionId,
        sequence: u64,
        raw_bytes: Bytes,
    ) -> Result<(), RawEventPublishError> {
        let session_bytes = session_id.as_str().as_bytes();
        let mut key = Vec::with_capacity(session_bytes.len() + 8);
        key.extend_from_slice(session_bytes);
        key.extend_from_slice(&sequence.to_be_bytes());

        let db = Arc::clone(&self.db);
        let raw_bytes_clone = raw_bytes.clone();

        // Perform the blocking DB write on a spawn_blocking thread
        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write().map_err(|e| {
                RawEventPublishError::Transport { message: format!("Failed to begin write txn: {}", e) }
            })?;
            {
                let mut table = write_txn.open_table(EVENTS_TABLE).map_err(|e| {
                    RawEventPublishError::Transport { message: format!("Failed to open table: {}", e) }
                })?;
                table.insert(key.as_slice(), raw_bytes_clone.as_ref()).map_err(|e| {
                    RawEventPublishError::Transport { message: format!("Failed to insert: {}", e) }
                })?;
            }
            write_txn.commit().map_err(|e| {
                RawEventPublishError::Transport { message: format!("Failed to commit: {}", e) }
            })?;
            Ok(())
        })
        .await
        .map_err(|e| RawEventPublishError::Transport { message: format!("Task panicked: {}", e) })??;

        Ok(())
    }
}

#[async_trait]
impl RawEventReader for RedbRawEventPublisher {
    async fn read_session_events(
        &self,
        session_id: &ChatSessionId,
    ) -> Result<Vec<Bytes>, RawEventPublishError> {
        let session_bytes = session_id.as_str().as_bytes();
        // The start key is session_id followed by sequence 0
        let mut start_key = Vec::with_capacity(session_bytes.len() + 8);
        start_key.extend_from_slice(session_bytes);
        start_key.extend_from_slice(&0u64.to_be_bytes());

        // The end key is session_id followed by u64::MAX
        let mut end_key = Vec::with_capacity(session_bytes.len() + 8);
        end_key.extend_from_slice(session_bytes);
        end_key.extend_from_slice(&u64::MAX.to_be_bytes());

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(|e| {
                RawEventPublishError::Transport { message: format!("Failed to begin read txn: {}", e) }
            })?;
            let table = read_txn.open_table(EVENTS_TABLE).map_err(|e| {
                RawEventPublishError::Transport { message: format!("Failed to open table: {}", e) }
            })?;

            let mut events = Vec::new();
            let range = start_key.as_slice()..=end_key.as_slice();
            for item in table.range(range).map_err(|e| {
                RawEventPublishError::Transport { message: format!("Failed to read range: {}", e) }
            })? {
                let (_, value) = item.map_err(|e| {
                    RawEventPublishError::Transport { message: format!("Failed to read item: {}", e) }
                })?;
                events.push(Bytes::copy_from_slice(value.value()));
            }

            Ok(events)
        })
        .await
        .map_err(|e| RawEventPublishError::Transport { message: format!("Task panicked: {}", e) })?
    }
}
