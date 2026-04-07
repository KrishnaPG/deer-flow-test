use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub content_hash: String,
    pub hierarchy: String,
    pub level: String,
    pub plane: String,
    pub payload_kind: String,
    pub payload_format: String,
    pub payload_size_bytes: u64,
    pub physical_location: String,
    pub correlation_ids: Vec<(String, String)>,
    pub lineage_refs: Vec<String>,
    pub routing_tags: Vec<(String, String)>,
    pub written_at: DateTime<Utc>,
    pub writer_identity: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub view_name: String,
    pub hierarchy_order: Vec<String>,
    pub filter_expr: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CheckoutReceipt {
    pub view_name: String,
    pub checkout_path: String,
    pub file_count: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CheckoutInfo {
    pub view_name: String,
    pub checkout_path: String,
    pub file_count: usize,
    pub status: String,
}
