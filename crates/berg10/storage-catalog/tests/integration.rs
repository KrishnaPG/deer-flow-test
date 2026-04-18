use berg10_storage_catalog::{
    Berg10Catalog,
    Berg10DataHierarchy,
    Berg10DataLevel,
    Berg10StoragePlane,
    CatalogConfig,
    ContentRecord,
    ContentTag,
    HierarchyPathSegment,
    HierarchyStatus,
    LineageRef,
    VirtualFolderHierarchy,
};
use berg10_storage_vfs::ContentHash;
use chrono::Utc;
use tempfile::TempDir;

#[tokio::test]
async fn end_to_end_file_registration_and_query() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig::with_base_dir(tmp.path().to_str().unwrap());

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let record = ContentRecord {
        content_hash: ContentHash::new(b"test_hash_e2e"),
        data_hierarchy: Berg10DataHierarchy::Orchestration,
        data_level: Berg10DataLevel::L0,
        storage_plane: Berg10StoragePlane::AS_IS,
        payload_kind: "chat-note".into(),
        payload_format: "jsonl".into(),
        payload_size_bytes: 42,
        correlation_ids: vec![ContentTag::new("mission_id", "m1")],
        lineage_refs: vec![LineageRef::new("parent_1")],
        routing_tags: vec![ContentTag::new("env", "test")],
        written_at: Utc::now(),
        writer_identity: Some("integration-test".into()),
        logical_filename: Some("note.jsonl".into()),
    };

    catalog.register_content(&record).await.unwrap();
    let retrieved = catalog.get_content(record.content_hash.as_str()).await.unwrap().unwrap();
    assert_eq!(retrieved.content_hash, record.content_hash);
    assert_eq!(retrieved.payload_kind.as_str(), "chat-note");
    assert_eq!(retrieved.logical_filename.as_ref().map(|v| v.as_str()), Some("note.jsonl"));
}

#[tokio::test]
async fn end_to_end_virtual_folder_hierarchy_management() {
    let tmp = TempDir::new().unwrap();
    let config = CatalogConfig::with_base_dir(tmp.path().to_str().unwrap());

    let catalog = Berg10Catalog::new(&config).await.unwrap();

    let hierarchy = VirtualFolderHierarchy {
        hierarchy_name: "integration-test-hierarchy".into(),
        hierarchy_order: vec![HierarchyPathSegment::DataLevel, HierarchyPathSegment::PayloadKind],
        filter_expr: Some("payload_kind = 'chat-note'".to_string()),
        status: HierarchyStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    catalog.create_virtual_folder_hierarchy(&hierarchy).await.unwrap();

    let hierarchies = catalog.list_virtual_folder_hierarchies(Some(HierarchyStatus::Active)).await.unwrap();
    assert_eq!(hierarchies.len(), 1);
    assert_eq!(hierarchies[0].hierarchy_name.as_str(), "integration-test-hierarchy");
    assert_eq!(hierarchies[0].hierarchy_order, vec![HierarchyPathSegment::DataLevel, HierarchyPathSegment::PayloadKind]);

    catalog.update_virtual_folder_hierarchy_status("integration-test-hierarchy", HierarchyStatus::Inactive).await.unwrap();
    let active = catalog.list_virtual_folder_hierarchies(Some(HierarchyStatus::Active)).await.unwrap();
    assert_eq!(active.len(), 0);
}
