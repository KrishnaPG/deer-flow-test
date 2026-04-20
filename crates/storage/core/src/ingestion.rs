use berg10_storage_catalog::{
    Berg10Catalog, Berg10DataHierarchy, Berg10DataLevel, Berg10PayloadFormat, Berg10PayloadKind,
    Berg10StoragePlane, ContentRecord, ContentTag, LineageRef, LogicalFilename,
};
use berg10_storage_vfs::StorageBackend;
use chrono::Utc;
use deer_foundation_contracts::{AppendDataRequest, FileSaved};

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
            deer_foundation_contracts::StoragePayloadDescriptor::InlineBytes { bytes } => {
                bytes.clone()
            }
            deer_foundation_contracts::StoragePayloadDescriptor::ExternalRef { uri } => {
                return Err(format!(
                    "External refs not yet supported in direct ingestion: {}",
                    uri
                ));
            }
        };

        // Store content in VFS using content-addressed blob storage.
        let content_hash = self
            .vfs
            .put_blob(payload_bytes.clone())
            .await
            .map_err(|e| format!("VFS write failed: {:?}", e))?;
        let hash_str = content_hash.as_str().to_string();

        // Build virtual-folder-hierarchy-relative path
        let relative_path = build_relative_path(
            &request.layout.hierarchy,
            request.layout.level,
            request.layout.plane,
            &request.layout.payload_kind,
            &request.layout.format,
            &request.layout.partition_tags,
            &hash_str,
        );

        let logical_filename = request.metadata.logical_filename.clone();

        // Register in catalog
        let record = ContentRecord {
            content_hash: content_hash.clone(),
            data_hierarchy: map_storage_hierarchy(&request.layout.hierarchy),
            data_level: Berg10DataLevel::from(request.layout.level),
            storage_plane: Berg10StoragePlane::from(request.layout.plane),
            payload_kind: Berg10PayloadKind::from(request.layout.payload_kind.clone()),
            payload_format: Berg10PayloadFormat::from(request.layout.format.clone()),
            payload_size_bytes: payload_bytes.len() as u64,
            correlation_ids: request
                .metadata
                .correlation
                .extra
                .iter()
                .cloned()
                .map(|(key, value)| ContentTag::new(key, value))
                .collect(),
            lineage_refs: request
                .metadata
                .lineage
                .parent_refs
                .iter()
                .cloned()
                .map(LineageRef::new)
                .collect(),
            routing_tags: request
                .layout
                .partition_tags
                .iter()
                .cloned()
                .map(|(key, value)| ContentTag::new(key, value))
                .collect(),
            written_at: Utc::now(),
            writer_identity: request.metadata.writer_identity.clone(),
            logical_filename: logical_filename.map(LogicalFilename::from),
        };

        self.catalog
            .register_content(&record)
            .await
            .map_err(|e| format!("Catalog registration failed: {:?}", e))?;

        // Emit FileSaved
        let correlation_tuples: Vec<(String, String)> = request.metadata.correlation.extra.clone();
        let lineage_strings: Vec<String> = request.metadata.lineage.parent_refs.clone();

        Ok(make_file_saved(
            request
                .metadata
                .logical_write_id
                .clone()
                .unwrap_or_else(|| deer_foundation_contracts::LogicalWriteId::new("unknown")),
            &relative_path,
            request.layout.hierarchy.clone(),
            request.layout.level,
            request.layout.plane,
            request.layout.payload_kind.clone(),
            request.layout.format.clone(),
            correlation_tuples,
            lineage_strings,
            request.layout.partition_tags.clone(),
            hash_str,
        ))
    }

    /// Deconstruct the bridge to recover catalog and VFS for downstream use.
    pub fn into_parts(self) -> (Berg10Catalog, StorageBackend) {
        (self.catalog, self.vfs)
    }
}

fn map_storage_hierarchy(
    hierarchy: &deer_foundation_contracts::StorageHierarchyTag,
) -> Berg10DataHierarchy {
    match hierarchy.as_str() {
        "orchestration" | "A" | "a" => Berg10DataHierarchy::Orchestration,
        "artifact_content" | "artifact" | "content" | "B" | "b" => {
            Berg10DataHierarchy::ArtifactContent
        }
        "presentation" | "view" | "C" | "c" => Berg10DataHierarchy::Presentation,
        _ => Berg10DataHierarchy::ArtifactContent,
    }
}
