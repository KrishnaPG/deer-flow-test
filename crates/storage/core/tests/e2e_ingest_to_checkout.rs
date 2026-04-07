use std::path::Path;

use berg10_storage_catalog::{Berg10Catalog, CatalogConfig, VirtualFolderHierarchy};
use berg10_storage_vfs::{StorageBackend, StorageConfig};
use berg10_warm_cache::WarmCacheConfig;
use chrono::Utc;
use deer_foundation_contracts::{
    AppendDataRequest, CanonicalLevel, CanonicalPlane, IdempotencyKey, LogicalWriteId,
    MissionId, RunId, AgentId, ArtifactId, TraceId,
    StorageCorrelationIds, StorageHierarchyTag, StorageLayout, StorageLineageRefs,
    StoragePayloadDescriptor, StoragePayloadFormat, StoragePayloadKind,
    StorageQosPolicy, StorageQosTemplate, StorageRequestMetadata, WriterId,
};
use deer_storage_core::ingestion::IngestionBridge;

fn make_test_request(
    payload: Vec<u8>,
    hierarchy: &str,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    payload_kind: &str,
    format: &str,
    logical_filename: Option<&str>,
    routing_tags: Vec<(String, String)>,
    write_id: &str,
) -> AppendDataRequest {
    AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new(hierarchy),
            level,
            plane,
            payload_kind: StoragePayloadKind::new(payload_kind),
            format: StoragePayloadFormat::new(format),
            partition_tags: routing_tags.clone(),
        },
        metadata: StorageRequestMetadata {
            logical_write_id: Some(LogicalWriteId::new(write_id)),
            idempotency_key: Some(IdempotencyKey::new(&format!("idem-{}", write_id))),
            writer_identity: Some(WriterId::new("e2e-tester")),
            logical_filename: logical_filename.map(String::from),
            correlation: StorageCorrelationIds {
                mission_id: Some(MissionId::new("m1")),
                run_id: Some(RunId::new("r1")),
                agent_id: Some(AgentId::new("a1")),
                artifact_id: Some(ArtifactId::new("art1")),
                trace_id: Some(TraceId::new("t1")),
                extra: routing_tags,
            },
            lineage: StorageLineageRefs::default(),
            known_content_hash: None,
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes { bytes: payload },
    }
}

#[tokio::test]
async fn e2e_single_file_ingest_and_checkout() {
    let tmp = tempfile::TempDir::new().unwrap();
    let base = tmp.path().to_str().unwrap();

    let catalog_config = CatalogConfig::with_base_dir(base);
    let catalog = Berg10Catalog::new(&catalog_config).await.unwrap();

    let vfs_root = format!("{}/blobs", base);
    std::fs::create_dir_all(&vfs_root).unwrap();
    let vfs_config = StorageConfig::local_fs(&vfs_root);
    let vfs = StorageBackend::new(&vfs_config).unwrap();
    let vfs_for_checkout = vfs.clone();

    let bridge = IngestionBridge::new(catalog, vfs);
    let request = make_test_request(
        b"hello from e2e single file test".to_vec(),
        "A",
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        "chat-note",
        "jsonl",
        Some("greeting.jsonl"),
        vec![("env".into(), "test".into())],
        "write-1",
    );
    let file_saved = bridge.ingest(&request).await.unwrap();

    assert_eq!(file_saved.content_hash.len(), 44);
    assert_eq!(file_saved.payload_kind.as_str(), "chat-note");

    let (catalog, _vfs) = bridge.into_parts();

    let record = catalog.get_file(&file_saved.content_hash).await.unwrap().unwrap();
    assert_eq!(record.content_hash, file_saved.content_hash);
    assert_eq!(record.payload_kind, "chat-note");
    assert_eq!(record.logical_filename.as_deref(), Some("greeting.jsonl"));
    assert_eq!(record.routing_tags, vec![("env".to_string(), "test".to_string())]);

    let hierarchy = VirtualFolderHierarchy {
        hierarchy_name: "test-hierarchy".to_string(),
        hierarchy_order: vec!["hierarchy".to_string(), "level".to_string(), "plane".to_string()],
        filter_expr: None,
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    catalog.create_virtual_folder_hierarchy(&hierarchy).await.unwrap();

    let warm_config = WarmCacheConfig::with_base_dir(base);
    let warm_cache = berg10_warm_cache::WarmCache::new(warm_config.clone(), catalog.clone(), vfs_for_checkout);

    let receipt = warm_cache.checkout_hierarchy("test-hierarchy").await.unwrap();
    assert_eq!(receipt.file_count, 1);

    let checkout_dir = Path::new(&receipt.checkout_path);
    assert!(checkout_dir.exists());

    let content_dir = Path::new(&warm_config.content_dir);
    assert!(content_dir.exists());

    let mut blob_count = 0;
    for entry in std::fs::read_dir(content_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            for sub in std::fs::read_dir(entry.path()).unwrap() {
                let sub = sub.unwrap();
                if sub.file_type().unwrap().is_dir() {
                    for blob in std::fs::read_dir(sub.path()).unwrap() {
                        let blob = blob.unwrap();
                        if blob.file_type().unwrap().is_file() {
                            blob_count += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(blob_count, 1);

    let mut symlink_count = 0;
    for entry in walk_dir(checkout_dir) {
        if entry.is_symlink() {
            symlink_count += 1;
            let target = std::fs::read_link(&entry).unwrap();
            let absolute_target = if target.is_relative() {
                entry.parent().unwrap().join(&target)
            } else {
                target
            };
            assert!(absolute_target.exists(), "Symlink target does not exist: {:?}", absolute_target);
            let content = std::fs::read(&absolute_target).unwrap();
            assert_eq!(content, b"hello from e2e single file test");
        }
    }
    assert_eq!(symlink_count, 1);
}

#[tokio::test]
async fn e2e_multiple_files_with_tag_filtering_and_hierarchy_ordering() {
    let tmp = tempfile::TempDir::new().unwrap();
    let base = tmp.path().to_str().unwrap();

    let catalog_config = CatalogConfig::with_base_dir(base);
    let catalog = Berg10Catalog::new(&catalog_config).await.unwrap();

    let vfs_root = format!("{}/blobs", base);
    std::fs::create_dir_all(&vfs_root).unwrap();
    let vfs_config = StorageConfig::local_fs(&vfs_root);
    let vfs = StorageBackend::new(&vfs_config).unwrap();
    let vfs_for_checkout = vfs.clone();

    let bridge = IngestionBridge::new(catalog, vfs);

    let requests = vec![
        make_test_request(
            b"adele song 2024".to_vec(),
            "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
            "mp3", "mp3",
            Some("rolling-in-the-deep.mp3"),
            vec![("singer".into(), "Adele".into()), ("year".into(), "2024".into())],
            "write-adele-2024",
        ),
        make_test_request(
            b"adele song 2023".to_vec(),
            "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
            "mp3", "mp3",
            Some("easy-on-me.mp3"),
            vec![("singer".into(), "Adele".into()), ("year".into(), "2023".into())],
            "write-adele-2023",
        ),
        make_test_request(
            b"beyonce song 2024".to_vec(),
            "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
            "mp3", "mp3",
            Some("break-my-soul.mp3"),
            vec![("singer".into(), "Beyonce".into()), ("year".into(), "2024".into())],
            "write-beyonce-2024",
        ),
        make_test_request(
            b"some transcript".to_vec(),
            "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
            "chat-note", "jsonl",
            Some("transcript.jsonl"),
            vec![("singer".into(), "N/A".into()), ("year".into(), "2024".into())],
            "write-transcript",
        ),
    ];

    let mut hashes = Vec::new();
    for req in &requests {
        let saved = bridge.ingest(req).await.unwrap();
        hashes.push(saved.content_hash);
    }

    let (catalog, _vfs) = bridge.into_parts();

    let hierarchy_by_singer = VirtualFolderHierarchy {
        hierarchy_name: "music-by-singer".to_string(),
        hierarchy_order: vec!["singer".to_string(), "year".to_string()],
        filter_expr: Some("payload_kind = 'mp3'".to_string()),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    catalog.create_virtual_folder_hierarchy(&hierarchy_by_singer).await.unwrap();

    let hierarchy_by_year = VirtualFolderHierarchy {
        hierarchy_name: "music-by-year".to_string(),
        hierarchy_order: vec!["year".to_string(), "singer".to_string()],
        filter_expr: Some("payload_kind = 'mp3'".to_string()),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    catalog.create_virtual_folder_hierarchy(&hierarchy_by_year).await.unwrap();

    let warm_config = WarmCacheConfig::with_base_dir(base);
    let warm_cache = berg10_warm_cache::WarmCache::new(warm_config.clone(), catalog.clone(), vfs_for_checkout);

    let singer_receipt = warm_cache.checkout_hierarchy("music-by-singer").await.unwrap();
    assert_eq!(singer_receipt.file_count, 3);

    let singer_checkout = Path::new(&singer_receipt.checkout_path);
    assert!(singer_checkout.exists());

    let mut adele_2024_found = false;
    let mut adele_2023_found = false;
    let mut beyonce_2024_found = false;
    for entry in walk_dir(singer_checkout) {
        if entry.is_symlink() {
            let path_str = entry.to_string_lossy();
            if path_str.contains("adele") && path_str.contains("2024") {
                adele_2024_found = true;
                assert!(path_str.ends_with("rolling-in-the-deep.mp3"));
            }
            if path_str.contains("adele") && path_str.contains("2023") {
                adele_2023_found = true;
                assert!(path_str.ends_with("easy-on-me.mp3"));
            }
            if path_str.contains("beyonce") && path_str.contains("2024") {
                beyonce_2024_found = true;
                assert!(path_str.ends_with("break-my-soul.mp3"));
            }
        }
    }
    assert!(adele_2024_found, "adele/2024 symlink not found");
    assert!(adele_2023_found, "adele/2023 symlink not found");
    assert!(beyonce_2024_found, "beyonce/2024 symlink not found");

    let year_receipt = warm_cache.checkout_hierarchy("music-by-year").await.unwrap();
    assert_eq!(year_receipt.file_count, 3);

    let year_checkout = Path::new(&year_receipt.checkout_path);
    assert!(year_checkout.exists());

    let mut year_2024_adele_found = false;
    let mut year_2024_beyonce_found = false;
    for entry in walk_dir(year_checkout) {
        if entry.is_symlink() {
            let path_str = entry.to_string_lossy();
            if path_str.contains("2024") && path_str.contains("adele") {
                year_2024_adele_found = true;
            }
            if path_str.contains("2024") && path_str.contains("beyonce") {
                year_2024_beyonce_found = true;
            }
        }
    }
    assert!(year_2024_adele_found, "2024/adele symlink not found in year hierarchy");
    assert!(year_2024_beyonce_found, "2024/beyonce symlink not found in year hierarchy");

    let files = catalog.query_files("payload_kind = 'chat-note'").await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].payload_kind, "chat-note");
    assert_eq!(files[0].logical_filename.as_deref(), Some("transcript.jsonl"));
}

#[tokio::test]
async fn e2e_content_deduplication_same_bytes_different_metadata() {
    let tmp = tempfile::TempDir::new().unwrap();
    let base = tmp.path().to_str().unwrap();

    let catalog_config = CatalogConfig::with_base_dir(base);
    let catalog = Berg10Catalog::new(&catalog_config).await.unwrap();

    let vfs_root = format!("{}/blobs", base);
    std::fs::create_dir_all(&vfs_root).unwrap();
    let vfs_config = StorageConfig::local_fs(&vfs_root);
    let vfs = StorageBackend::new(&vfs_config).unwrap();
    let vfs_for_read = vfs.clone();

    let bridge = IngestionBridge::new(catalog, vfs);

    let same_payload = b"identical content for dedup test".to_vec();

    let req1 = make_test_request(
        same_payload.clone(),
        "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
        "mp3", "mp3",
        Some("song-a.mp3"),
        vec![("singer".into(), "Adele".into())],
        "write-dedup-1",
    );
    let req2 = make_test_request(
        same_payload.clone(),
        "B", CanonicalLevel::L1, CanonicalPlane::AsIs,
        "mp3", "mp3",
        Some("song-b.mp3"),
        vec![("singer".into(), "Beyonce".into())],
        "write-dedup-2",
    );

    let saved1 = bridge.ingest(&req1).await.unwrap();
    let saved2 = bridge.ingest(&req2).await.unwrap();

    assert_eq!(saved1.content_hash, saved2.content_hash, "Identical content should produce identical hashes");

    let (catalog, _vfs) = bridge.into_parts();

    let record1 = catalog.get_file(&saved1.content_hash).await.unwrap().unwrap();
    assert_eq!(record1.logical_filename.as_deref(), Some("song-a.mp3"));

    let record2 = catalog.get_file(&saved2.content_hash).await.unwrap().unwrap();
    assert_eq!(record2.logical_filename.as_deref(), Some("song-a.mp3"));

    let _blob_path = format!("{}/{}", vfs_root, saved1.content_hash);
    let blob_count = std::fs::read_dir(&vfs_root).unwrap().count();
    assert_eq!(blob_count, 1, "Only one blob should exist for identical content");

    let data = vfs_for_read.read(&saved1.content_hash).await.unwrap();
    assert_eq!(data, same_payload);
}

#[tokio::test]
async fn e2e_filter_by_multiple_attributes() {
    let tmp = tempfile::TempDir::new().unwrap();
    let base = tmp.path().to_str().unwrap();

    let catalog_config = CatalogConfig::with_base_dir(base);
    let catalog = Berg10Catalog::new(&catalog_config).await.unwrap();

    let vfs_root = format!("{}/blobs", base);
    std::fs::create_dir_all(&vfs_root).unwrap();
    let vfs_config = StorageConfig::local_fs(&vfs_root);
    let vfs = StorageBackend::new(&vfs_config).unwrap();
    let vfs_for_checkout = vfs.clone();

    let bridge = IngestionBridge::new(catalog, vfs);

    let req_l0 = make_test_request(
        b"level zero content".to_vec(),
        "A", CanonicalLevel::L0, CanonicalPlane::AsIs,
        "chat-note", "jsonl",
        Some("l0-note.jsonl"),
        vec![("env".into(), "prod".into())],
        "write-l0",
    );
    let req_l1 = make_test_request(
        b"level one content".to_vec(),
        "A", CanonicalLevel::L1, CanonicalPlane::AsIs,
        "chat-note", "jsonl",
        Some("l1-note.jsonl"),
        vec![("env".into(), "prod".into())],
        "write-l1",
    );
    let req_l0_chunks = make_test_request(
        b"level zero chunks".to_vec(),
        "A", CanonicalLevel::L0, CanonicalPlane::Chunks,
        "chunks", "bin",
        Some("l0-chunks.bin"),
        vec![("env".into(), "prod".into())],
        "write-l0-chunks",
    );

    bridge.ingest(&req_l0).await.unwrap();
    bridge.ingest(&req_l1).await.unwrap();
    bridge.ingest(&req_l0_chunks).await.unwrap();

    let (catalog, _vfs) = bridge.into_parts();

    let l0_files = catalog.query_files("level = 'L0'").await.unwrap();
    assert_eq!(l0_files.len(), 2);

    let l0_as_is = catalog.query_files("level = 'L0' AND plane = 'as-is'").await.unwrap();
    assert_eq!(l0_as_is.len(), 1);
    assert_eq!(l0_as_is[0].logical_filename.as_deref(), Some("l0-note.jsonl"));

    let chunks_files = catalog.query_files("plane = 'chunks'").await.unwrap();
    assert_eq!(chunks_files.len(), 1);
    assert_eq!(chunks_files[0].payload_kind, "chunks");

    let hierarchy = VirtualFolderHierarchy {
        hierarchy_name: "by-level".to_string(),
        hierarchy_order: vec!["level".to_string(), "plane".to_string()],
        filter_expr: Some("level = 'L0'".to_string()),
        status: "active".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    catalog.create_virtual_folder_hierarchy(&hierarchy).await.unwrap();

    let warm_config = WarmCacheConfig::with_base_dir(base);
    let warm_cache = berg10_warm_cache::WarmCache::new(warm_config.clone(), catalog.clone(), vfs_for_checkout);

    let receipt = warm_cache.checkout_hierarchy("by-level").await.unwrap();
    assert_eq!(receipt.file_count, 2);

    let checkout_dir = Path::new(&receipt.checkout_path);
    let mut l0_as_is_found = false;
    let mut l0_chunks_found = false;
    for entry in walk_dir(checkout_dir) {
        if entry.is_symlink() {
            let path_str = entry.to_string_lossy();
            if path_str.contains("l0") && path_str.contains("as-is") {
                l0_as_is_found = true;
            }
            if path_str.contains("l0") && path_str.contains("chunks") {
                l0_chunks_found = true;
            }
        }
    }
    assert!(l0_as_is_found, "L0/as-is symlink not found");
    assert!(l0_chunks_found, "L0/chunks symlink not found");
}

fn walk_dir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut entries = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    entries.extend(walk_dir(&path));
                } else {
                    entries.push(path);
                }
            }
        }
    }
    entries
}
