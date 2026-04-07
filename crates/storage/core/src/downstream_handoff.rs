use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, DerivationTrigger, FileSaved, LogicalWriteId,
    StorageCorrelationIds, StorageHierarchyTag, StorageLineageRefs, StoragePayloadFormat,
    StoragePayloadKind,
};

/// Assemble a `FileSaved` event with full routing context for downstream consumers.
pub fn make_file_saved(
    logical_write_id: LogicalWriteId,
    relative_path: &str,
    hierarchy: StorageHierarchyTag,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    payload_kind: StoragePayloadKind,
    format: StoragePayloadFormat,
    correlation_ids: Vec<(String, String)>,
    lineage_refs: Vec<String>,
    routing_tags: Vec<(String, String)>,
    content_hash: String,
) -> FileSaved {
    FileSaved {
        logical_write_id,
        target: deer_foundation_contracts::FileSavedTarget::SingleFile {
            relative_path: relative_path.into(),
        },
        hierarchy,
        level,
        plane,
        payload_kind,
        format,
        correlation_ids: StorageCorrelationIds::from_tuples(correlation_ids),
        lineage_refs: StorageLineageRefs::from_parent_refs(lineage_refs),
        routing_tags,
        content_hash,
    }
}

/// Emit a `DerivationTrigger` pointing at a finalized manifest, not at partial members.
pub fn derive_trigger_from_manifest(
    logical_write_id: &str,
    manifest_path: &str,
    correlation_ids: Vec<(String, String)>,
    lineage_refs: Vec<String>,
    routing_tags: Vec<(String, String)>,
    content_hash: String,
) -> DerivationTrigger {
    DerivationTrigger {
        logical_write_id: LogicalWriteId::new(logical_write_id),
        relative_target: manifest_path.into(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L3,
        plane: CanonicalPlane::Chunks,
        payload_kind: StoragePayloadKind::new("transcript"),
        format: StoragePayloadFormat::new("parquet"),
        correlation_ids: StorageCorrelationIds::from_tuples(correlation_ids),
        lineage_refs: StorageLineageRefs::from_parent_refs(lineage_refs),
        routing_tags,
        content_hash,
    }
}
