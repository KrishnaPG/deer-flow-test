use berg10_storage_vfs::{hash_content, StorageBackend, StorageConfig};

#[tokio::test]
async fn end_to_end_write_read_cycle() {
    let config = StorageConfig::memory();
    let backend = StorageBackend::new(&config).unwrap();

    let data = b"integration test content";
    let hash = hash_content(data);

    backend.write(hash.as_str(), data).await.unwrap();
    let retrieved = backend.read(hash.as_str()).await.unwrap();
    assert_eq!(retrieved, data);
}

#[tokio::test]
async fn end_to_end_local_fs_write_read_cycle() {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = StorageConfig::local_fs(tmp.path().to_str().unwrap());
    let backend = StorageBackend::new(&config).unwrap();

    let data = b"local fs test content";
    let hash = hash_content(data);

    backend.write(hash.as_str(), data).await.unwrap();
    let retrieved = backend.read(hash.as_str()).await.unwrap();
    assert_eq!(retrieved, data);
}

#[tokio::test]
async fn deduplication_identical_content_writes_to_same_key() {
    let config = StorageConfig::memory();
    let backend = StorageBackend::new(&config).unwrap();

    let data = b"deduplication test content";
    let hash = hash_content(data);

    backend.write(hash.as_str(), data).await.unwrap();
    backend.write(hash.as_str(), data).await.unwrap();

    assert!(backend.exists(hash.as_str()).await.unwrap());
    let retrieved = backend.read(hash.as_str()).await.unwrap();
    assert_eq!(retrieved, data);
}

#[tokio::test]
async fn deduplication_different_content_has_different_keys() {
    let config = StorageConfig::memory();
    let backend = StorageBackend::new(&config).unwrap();

    let data1 = b"content one";
    let data2 = b"content two";
    let hash1 = hash_content(data1);
    let hash2 = hash_content(data2);

    assert_ne!(hash1.as_str(), hash2.as_str());

    backend.write(hash1.as_str(), data1).await.unwrap();
    backend.write(hash2.as_str(), data2).await.unwrap();

    assert!(backend.exists(hash1.as_str()).await.unwrap());
    assert!(backend.exists(hash2.as_str()).await.unwrap());
    assert_eq!(backend.read(hash1.as_str()).await.unwrap(), data1);
    assert_eq!(backend.read(hash2.as_str()).await.unwrap(), data2);
}
