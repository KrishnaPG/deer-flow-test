use berg10_storage_catalog::{Berg10Catalog, CatalogConfig, FileRecord, ViewDefinition};
use chrono::Utc;
use tempfile::TempDir;

#[tokio::test]
async fn end_to_end_file_registration_and_query() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig {
        warehouse_path: tmp.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let record = FileRecord {
        content_hash: "test_hash_e2e".to_string(),
        hierarchy: "A".to_string(),
        level: "L0".to_string(),
        plane: "as-is".to_string(),
        payload_kind: "chat-note".to_string(),
        payload_format: "jsonl".to_string(),
        payload_size_bytes: 42,
        physical_location: "s3://berg10-storage/test_hash_e2e".to_string(),
        correlation_ids: vec![("mission_id".to_string(), "m1".to_string())],
        lineage_refs: vec!["parent_1".to_string()],
        routing_tags: vec![("env".to_string(), "test".to_string())],
        written_at: Utc::now(),
        writer_identity: Some("integration-test".to_string()),
    };

    catalog.register_file(&record).await.unwrap();
    let retrieved = catalog.get_file("test_hash_e2e").await.unwrap().unwrap();
    assert_eq!(retrieved.content_hash, "test_hash_e2e");
    assert_eq!(retrieved.payload_kind, "chat-note");
    assert_eq!(retrieved.physical_location, "s3://berg10-storage/test_hash_e2e");
}

#[tokio::test]
async fn end_to_end_view_management() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig {
        warehouse_path: tmp.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let view = ViewDefinition {
        view_name: "integration-test-view".to_string(),
        hierarchy_order: vec!["level".to_string(), "payload_kind".to_string()],
        filter_expr: Some("payload_kind = 'chat-note'".to_string()),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    catalog.create_view(&view).await.unwrap();

    let views = catalog.list_views(Some("active")).await.unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].view_name, "integration-test-view");
    assert_eq!(views[0].hierarchy_order, vec!["level", "payload_kind"]);

    catalog.update_view_status("integration-test-view", "inactive").await.unwrap();
    let active = catalog.list_views(Some("active")).await.unwrap();
    assert_eq!(active.len(), 0);
}
