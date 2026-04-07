use berg10_storage_catalog::{Berg10Catalog, FileRecord};
use berg10_storage_vfs::{hash_content, StorageBackend};
use chrono::Utc;
use deer_foundation_contracts::{AppendDataRequest, CanonicalPlane, FileSaved};

use crate::downstream_handoff::make_file_saved;
use crate::view_path_builder::build_relative_path;

/// Bridge: AppendDataRequest -> VFS write -> catalog registration -> FileSaved emission.
pub struct IngestionBridge {
    catalog: Berg10Catalog,
    vfs: StorageBackend,
}

impl IngestionBridge {
    pub fn new(catalog: Berg10Catalog, vfs: StorageBackend) -> Self {
        Self { catalog, vfs }
    }

    pub async fn ingest(&self, request: &AppendDataRequest) -> Result<FileSaved, String> {
        let payload_bytes = match &request.payload {
            deer_foundation_contracts::StoragePayloadDescriptor::InlineBytes { bytes } => bytes.clone(),
            deer_foundation_contracts::StoragePayloadDescriptor::ExternalRef { uri } => {
                return Err(format!("External refs not yet supported in direct ingestion: {}", uri));
            }
        };

        // Hash the content
        let content_hash = hash_content(&payload_bytes);
        let hash_str = content_hash.as_str().to_string();

        // Write to VFS with hash-based key
        self.vfs.write(&hash_str, payload_bytes.clone()).await
            .map_err(|e| format!("VFS write failed: {:?}", e))?;

        // Build view-relative path
        let relative_path = build_relative_path(
            &request.layout.hierarchy,
            request.layout.level,
            request.layout.plane,
            &request.layout.payload_kind,
            &request.layout.format,
            &request.layout.partition_tags,
            &hash_str,
        );

        // Build physical location
        let physical_location = format!("berg10://content/{}", hash_str);

        // Register in catalog
        let record = FileRecord {
            content_hash: hash_str.clone(),
            hierarchy: request.layout.hierarchy.as_str().to_string(),
            level: format!("{:?}", request.layout.level),
            plane: match request.layout.plane {
                CanonicalPlane::AsIs => "as-is".to_string(),
                CanonicalPlane::Chunks => "chunks".to_string(),
                CanonicalPlane::Embeddings => "embeddings".to_string(),
            },
            payload_kind: request.layout.payload_kind.as_str().to_string(),
            payload_format: request.layout.format.as_str().to_string(),
            payload_size_bytes: payload_bytes.len() as u64,
            physical_location: physical_location.clone(),
            correlation_ids: request.metadata.correlation.extra.clone(),
            lineage_refs: request.metadata.lineage.parent_refs.clone(),
            routing_tags: Vec::new(),
            written_at: Utc::now(),
            writer_identity: request.metadata.writer_identity.as_ref().map(|w| w.to_string()),
        };

        self.catalog.register_file(&record).await
            .map_err(|e| format!("Catalog registration failed: {:?}", e))?;

        // Emit FileSaved
        let correlation_tuples: Vec<(String, String)> = request.metadata.correlation.extra.clone();
        let lineage_strings: Vec<String> = request.metadata.lineage.parent_refs.clone();

        Ok(make_file_saved(
            request.metadata.logical_write_id.clone().unwrap_or_else(|| deer_foundation_contracts::LogicalWriteId::new("unknown")),
            &relative_path,
            request.layout.hierarchy.clone(),
            request.layout.level,
            request.layout.plane,
            request.layout.payload_kind.clone(),
            request.layout.format.clone(),
            correlation_tuples,
            lineage_strings,
            Vec::new(),
            hash_str,
            physical_location,
        ))
    }
}
