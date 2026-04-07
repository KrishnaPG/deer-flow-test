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
