## 1. Workspace and dependency setup

- [ ] 1.1 Add `berg10-storage-catalog`, `berg10-storage-vfs`, and `berg10-warm-cache` to workspace `Cargo.toml` members
- [ ] 1.2 Add dependencies to workspace: `iceberg` 0.9.0, `iceberg-catalog-sql` 0.9.0, `iceberg-catalog-rest` 0.9.0, `iceberg-storage-opendal` 0.9.0, `opendal` 0.55.0, `blake3` 1.8.4, `bs58` 0.5.1, `iceberg-datafusion` 0.9.0
- [ ] 1.3 Create `crates/berg10/storage-catalog/Cargo.toml` with dependencies on `iceberg`, `iceberg-catalog-sql`, `iceberg-catalog-rest`, `iceberg-storage-opendal`, `opendal`, `deer-foundation-contracts`
- [ ] 1.4 Create `crates/berg10/storage-vfs/Cargo.toml` with dependencies on `opendal`, `blake3`, `bs58`, `deer-foundation-contracts`
- [ ] 1.5 Create `crates/berg10/warm-cache/Cargo.toml` with dependencies on `berg10-storage-catalog`, `berg10-storage-vfs`, `deer-foundation-contracts`
- [ ] 1.6 Add `berg10-storage-catalog` and `berg10-storage-vfs` as dependencies to `deer-storage-core/Cargo.toml`

## 2. Content addressing module (berg10-storage-vfs)

- [ ] 2.1 Create `crates/berg10/storage-vfs/src/lib.rs` with module exports
- [ ] 2.2 Implement `hash_content(data: &[u8]) -> String` using blake3 + bs58 encoding
- [ ] 2.3 Implement `decode_content_hash(b58: &str) -> Result<[u8; 32], Error>` for round-trip verification
- [ ] 2.4 Write unit tests for hash_content determinism, uniqueness, and base58 round-trip
- [ ] 2.5 Write unit test verifying identical content produces identical hash

## 3. VFS storage backend (berg10-storage-vfs)

- [ ] 3.1 Create `crates/berg10/storage-vfs/src/backend.rs` with `StorageBackend` trait wrapping OpenDAL `Operator`
- [ ] 3.2 Implement `StorageBackend::new(config: &StorageConfig) -> Result<Self>` supporting `s3`, `fs`, `gcs`, `azure`, `lakefs` backend types
- [ ] 3.3 Implement `StorageBackend::write(key: &str, data: &[u8]) -> Result<()>` for content-addressed writes
- [ ] 3.4 Implement `StorageBackend::read(key: &str) -> Result<Vec<u8>>` for content retrieval by hash
- [ ] 3.5 Implement `StorageBackend::exists(key: &str) -> Result<bool>` for content existence checks
- [ ] 3.6 Apply OpenDAL RetryLayer and TracingLayer to all backend operations
- [ ] 3.7 Implement LakeFS write path using LakeFS REST API (since OpenDAL LakeFS service is read-only)
- [ ] 3.8 Write integration tests for S3 backend (using MinIO or mock)
- [ ] 3.9 Write integration tests for local filesystem backend

## 4. Iceberg catalog initialization (berg10-storage-catalog)

- [ ] 4.1 Create `crates/berg10/storage-catalog/src/lib.rs` with module exports
- [ ] 4.2 Create `crates/berg10/storage-catalog/src/config.rs` with `CatalogConfig` struct supporting `sql` and `rest` backends
- [ ] 4.3 Implement `Catalog::new(config: &CatalogConfig) -> Result<Self>` that initializes the appropriate catalog backend
- [ ] 4.4 Implement SQLite catalog initialization using `iceberg-catalog-sql` with database at configurable path (default: `base_dir/catalog/iceberg.sqlite`)
- [ ] 4.5 Implement REST catalog initialization using `iceberg-catalog-rest` with configurable URI and auth token
- [ ] 4.6 Create `berg10.files` Iceberg table schema with columns: content_hash (PK), hierarchy, level, plane, payload_kind, payload_format, payload_size_bytes, physical_location, correlation_ids (map), lineage_refs (array), routing_tags (map), written_at (timestamp), writer_identity
- [ ] 4.7 Create `berg10.views` Iceberg table schema with columns: view_name (PK), hierarchy_order (array), filter_expr (string), status (string), created_at (timestamp), updated_at (timestamp)
- [ ] 4.8 Write tests for catalog initialization with SQLite backend
- [ ] 4.9 Write tests for table creation and schema verification

## 5. File registration in catalog (berg10-storage-catalog)

- [ ] 5.1 Implement `Catalog::register_file(record: &FileRecord) -> Result<()>` to insert into `berg10.files`
- [ ] 5.2 Implement `Catalog::get_file(content_hash: &str) -> Result<Option<FileRecord>>` for hash-based lookup
- [ ] 5.3 Implement `Catalog::query_files(filter: &str) -> Result<Vec<FileRecord>>` for attribute-based queries
- [ ] 5.4 Implement LakeFS snapshot alignment: after each Iceberg commit, trigger LakeFS commit on current branch
- [ ] 5.5 Write tests for file registration and retrieval
- [ ] 5.6 Write tests for attribute-based queries (filter by payload_kind, routing_tags, etc.)
- [ ] 5.7 Write tests for LakeFS snapshot alignment (mock LakeFS API)

## 6. View management (berg10-storage-catalog)

- [ ] 6.1 Implement `Catalog::create_view(view: &ViewDefinition) -> Result<()>` to insert into `berg10.views`
- [ ] 6.2 Implement `Catalog::list_views(status: Option<&str>) -> Result<Vec<ViewDefinition>>` for listing views
- [ ] 6.3 Implement `Catalog::update_view_status(view_name: &str, status: &str) -> Result<()>` for activating/deactivating views
- [ ] 6.4 Implement `Catalog::delete_view(view_name: &str) -> Result<()>` for removing view definitions
- [ ] 6.5 Implement `Catalog::resolve_view(view_name: &str) -> Result<Vec<FileRecord>>` to get files matching a view's filter and hierarchy
- [ ] 6.6 Write tests for view CRUD operations
- [ ] 6.7 Write tests for view resolution with filter expressions

## 7. Warm cache implementation (berg10-warm-cache)

- [ ] 7.1 Create `crates/berg10/warm-cache/src/lib.rs` with module exports
- [ ] 7.2 Create `crates/berg10/warm-cache/src/config.rs` with `WarmCacheConfig` (base_dir, checkouts_dir)
- [ ] 7.3 Implement `WarmCache::new(config: &WarmCacheConfig, catalog: &Catalog, vfs: &Vfs) -> Result<Self>`
- [ ] 7.4 Implement `WarmCache::checkout(view_name: &str) -> Result<CheckoutReceipt>` that materializes a view as symlink tree
- [ ] 7.5 Implement view path builder: given a view's hierarchy_order and a file's attributes, produce the symlink path within the checkout directory
- [ ] 7.6 Implement content cache population: fetch missing blobs from cold storage via VFS before creating symlinks
- [ ] 7.7 Implement `WarmCache::list_checkouts() -> Result<Vec<CheckoutInfo>>` for listing active checkouts
- [ ] 7.8 Implement `WarmCache::deactivate_checkout(view_name: &str) -> Result<()>` to remove symlink tree and update view status
- [ ] 7.9 Implement `WarmCache::remove_checkout(view_name: &str) -> Result<()>` to delete symlink tree and remove view definition
- [ ] 7.10 Write tests for view checkout with single-file hierarchy
- [ ] 7.11 Write tests for multiple simultaneous view checkouts
- [ ] 7.12 Write tests for content cache population (mock VFS fetch)
- [ ] 7.13 Write tests for checkout deactivation and removal

## 8. Evolve FileSaved contract (deer-foundation-contracts)

- [ ] 8.1 Add `content_hash: String` field to `FileSaved` struct in `crates/foundation/contracts/src/storage.rs`
- [ ] 8.2 Add `physical_location: String` field to `FileSaved` struct
- [ ] 8.3 Update `DerivationTrigger` to include `content_hash` and `physical_location` fields
- [ ] 8.4 Update `FileSavedTarget` documentation to clarify `relative_path` is view-relative, not physical
- [ ] 8.5 Update `crates/foundation/contracts/tests/storage_contracts.rs` tests for new FileSaved fields
- [ ] 8.6 Run `cargo test -p deer-foundation-contracts` to verify all contract tests pass

## 9. Repurpose path_builder for view materialization (deer-storage-core)

- [ ] 9.1 Rename `path_builder.rs` to `view_path_builder.rs` in `crates/storage/core/src/`
- [ ] 9.2 Update `build_relative_path` to accept a `hierarchy_order: &[String]` parameter and file attributes, producing view-relative paths
- [ ] 9.3 Update `crates/storage/core/tests/path_builder.rs` to test view path construction with different hierarchy orderings
- [ ] 9.4 Run `cargo test -p deer-storage-core` to verify all tests pass

## 10. Wire ingestion flow through catalog and VFS

- [ ] 10.1 Create `crates/storage/core/src/ingestion.rs` that bridges `AppendDataRequest` → VFS write → catalog registration → `FileSaved` emission
- [ ] 10.2 Implement content hashing of payload in ingestion path
- [ ] 10.3 Implement VFS content write with hash-based key
- [ ] 10.4 Implement catalog file registration with full metadata
- [ ] 10.5 Emit `FileSaved` with `content_hash`, `physical_location`, and view-relative path
- [ ] 10.6 Write integration test for full ingestion flow (mock VFS and catalog)

## 11. Integration tests and full verification

- [ ] 11.1 Create `crates/berg10/storage-catalog/tests/integration.rs` with end-to-end catalog tests
- [ ] 11.2 Create `crates/berg10/storage-vfs/tests/integration.rs` with end-to-end VFS tests
- [ ] 11.3 Create `crates/berg10/warm-cache/tests/integration.rs` with end-to-end warm cache tests
- [ ] 11.4 Run `cargo test -p berg10-storage-catalog -p berg10-storage-vfs -p berg10-warm-cache` to verify all new crate tests pass
- [ ] 11.5 Run `cargo test -p deer-storage-core` to verify ingestion protocol tests pass
- [ ] 11.6 Run `cargo test -p deer-foundation-contracts` to verify contract tests pass
- [ ] 11.7 Run `cargo test -v` for full workspace verification with no regressions
- [ ] 11.8 Run `cargo clippy --workspace` for lint compliance
- [ ] 11.9 Run `cargo build --workspace` to verify full workspace compilation
