use berg10_storage_catalog::{Berg10Catalog, CatalogConfig, FileRecord, VirtualFolderHierarchy};
use chrono::Utc;
use tempfile::TempDir;

#[tokio::test]
async fn end_to_end_file_registration_and_query() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig::with_base_dir(tmp.path().to_str().unwrap());

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let record = FileRecord {
        content_hash: "test_hash_e2e".to_string(),
        hierarchy: "A".to_string(),
        level: "L0".to_string(),
        plane: "as-is".to_string(),
        payload_kind: "chat-note".to_string(),
        payload_format: "jsonl".to_string(),
        payload_size_bytes: 42,
        correlation_ids: vec![("mission_id".to_string(), "m1".to_string())],
        lineage_refs: vec!["parent_1".to_string()],
        routing_tags: vec![("env".to_string(), "test".to_string())],
        written_at: Utc::now(),
        writer_identity: Some("integration-test".to_string()),
        logical_filename: Some("note.jsonl".to_string()),
    };

    catalog.register_file(&record).await.unwrap();
    let retrieved = catalog.get_file("test_hash_e2e").await.unwrap().unwrap();
    assert_eq!(retrieved.content_hash, "test_hash_e2e");
    assert_eq!(retrieved.payload_kind, "chat-note");
    assert_eq!(retrieved.logical_filename.as_deref(), Some("note.jsonl"));
}

#[tokio::test]
async fn end_to_end_virtual_folder_hierarchy_management() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig::with_base_dir(tmp.path().to_str().unwrap());

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let hierarchy = VirtualFolderHierarchy {
        hierarchy_name: "integration-test-hierarchy".to_string(),
        hierarchy_order: vec!["level".to_string(), "payload_kind".to_string()],
        filter_expr: Some("payload_kind = 'chat-note'".to_string()),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    catalog.create_virtual_folder_hierarchy(&hierarchy).await.unwrap();

    let hierarchies = catalog.list_virtual_folder_hierarchies(Some("active")).await.unwrap();
    assert_eq!(hierarchies.len(), 1);
    assert_eq!(hierarchies[0].hierarchy_name, "integration-test-hierarchy");
    assert_eq!(hierarchies[0].hierarchy_order, vec!["level", "payload_kind"]);

    catalog.update_virtual_folder_hierarchy_status("integration-test-hierarchy", "inactive").await.unwrap();
    let active = catalog.list_virtual_folder_hierarchies(Some("active")).await.unwrap();
    assert_eq!(active.len(), 0);
}
