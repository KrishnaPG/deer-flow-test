## Context

The current `deer-storage-core` crate implements an ingestion protocol: it validates `AppendDataRequest`s, manages admission control, publishes durable intent events via `DurablePublisher`, and emits `FileSaved` handoff events. Path construction (`path_builder.rs`) produces hierarchical paths like `B/L4/as-is/thumbnail/64x64/abc123.png` from layout dimensions. This works for simple append-only storage but cannot support:

1. **Virtual hierarchies** — users cannot reorganize data views without moving files.
2. **Queryable metadata** — attributes are encoded in paths, not in a queryable catalog.
3. **Pluggable backends** — path construction assumes directory-like key namespaces.
4. **Content-addressed storage** — the file's identity is its path, not its content hash.

The existing `deer-storage-core` crate will be preserved as the ingestion protocol layer. Three new crates layer on top: `berg10-storage-catalog`, `berg10-storage-vfs`, and `berg10-warm-cache`.

The platform uses a user-configurable `base_dir` as the root for all subsystems. The warm cache lives at `base_dir/checkouts/`. The catalog defaults to SQLite at `base_dir/catalog/iceberg.sqlite`. Content blobs are cached locally at `base_dir/content/`.

## Goals / Non-Goals

**Goals:**
- Content-addressed storage using `base58(blake3(payload))` as file identity
- Iceberg catalog (`berg10.files`, `berg10.views`) as the queryable metadata layer
- Configurable catalog backend: SQLite (default, embedded) or REST (Lakekeeper/Polaris)
- Pluggable storage backends via OpenDAL (50+ services: S3, GCS, Azure, LakeFS, Google Drive, OneDrive, WebDAV, SFTP, etc.)
- Warm cache: user-defined views materialized as symlink trees under `base_dir/checkouts/`
- Multiple simultaneous view checkouts managed via UI
- Iceberg snapshot alignment with LakeFS commits when LakeFS is the backend
- Zero custom storage code — use `iceberg` (0.9.0), `opendal` (0.55.0), `blake3` (1.8.4), `bs58` (0.5.1)
- Preserve existing `deer-storage-core` ingestion protocol; data generators unchanged

**Non-Goals:**
- FUSE filesystem mounting (future consideration)
- Real-time view synchronization (views materialized on checkout, not live-synced)
- Reflink/hardlink strategies (symlinks are the default; other strategies are future work)
- Writing custom storage adapters — OpenDAL provides all backends
- Modifying DeerFlow driver or any data generator
- Implementing the actual durable publish transport (Redpanda/NATS integration is out of scope)

## Decisions

### 1. Iceberg as the metadata catalog

**Decision:** Use Apache Iceberg (via `iceberg` crate 0.9.0) as the content and view registry. Two tables: `berg10.files` (content registry) and `berg10.views` (virtual hierarchy definitions).

**Rationale:** Iceberg gives us a standard SQL interface, time-travel queries, schema evolution, and native integration with DuckDB, ClickHouse, Spark, and Trino. Downstream tools can query file metadata directly without custom APIs.

**Alternatives considered:**
- SQLite alone: simpler but not queryable by external analytics tools.
- Custom JSON manifests: lightweight but no schema evolution or time travel.
- Iceberg wins because it serves both as our internal catalog AND as the external query surface.

### 2. Catalog backend: SQL default, REST optional

**Decision:** Default to `iceberg-catalog-sql` with SQLite for embedded operation. Support `iceberg-catalog-rest` for connecting to Lakekeeper/Polaris in production. Selected at startup from configuration.

**Rationale:** SQLite requires zero infrastructure, is battle-tested, and works for development and single-node deployments. The REST catalog interface is standardized — switching to Lakekeeper later requires only config changes.

### 3. OpenDAL as the storage abstraction layer

**Decision:** Use `opendal` (0.55.0) as the unified storage backend interface, bridged to Iceberg via `iceberg-storage-opendal` (0.9.0).

**Rationale:** OpenDAL supports 50+ backends through a single `Operator` API with composable middleware (retry, tracing, caching). `object_store` only supports 6 backends. The `iceberg-storage-opendal` crate is the official bridge — no custom FileIO code needed.

### 4. Content addressing: base58(blake3(payload))

**Decision:** Hash all content with blake3 (1.8.4), encode the 32-byte digest as base58 (via `bs58` 0.5.1), and use the resulting string as the content key.

**Rationale:** blake3 is the fastest cryptographic hash with SIMD optimization (112M+ downloads). base58 eliminates ambiguous characters (0/O, I/l) making hashes human-readable and URL-safe. bs58 is the Solana/Bitcoin standard (62M+ downloads).

### 5. Warm cache: symlinks to local content store

**Decision:** Materialize views as symlink trees under `base_dir/checkouts/<view-name>/`. Symlinks point to `base_dir/content/<b58hash>` (local content cache). The content cache is populated on first access (lazy fetch from cold storage).

**Rationale:** Symlinks are zero-copy, universal on Unix, and allow multiple views to reference the same content blob. The local content cache decouples the warm cache from cold storage availability. When cold storage is a local mount (e.g., S3 via s3fs), the content cache can optionally skip local copies.

### 6. View definitions stored in Iceberg

**Decision:** Virtual hierarchies are defined as rows in the `berg10.views` Iceberg table, not as config files. Each view has a name, hierarchy ordering (array of attribute names), an optional filter expression, and a status.

**Rationale:** Storing views in Iceberg means they're queryable, versionable (via Iceberg snapshots), and manageable through the same SQL interface as file metadata. The UI can list, create, update, and delete views by querying the catalog.

### 7. Path builder repurposed for view materialization

**Decision:** The existing `path_builder.rs` in `deer-storage-core` is repurposed to build view-relative paths for warm cache materialization. It takes a view's hierarchy ordering and a file's attributes to produce the symlink path within the checkout directory.

**Rationale:** The sanitization and segment construction logic is reusable. We're not throwing it away — we're changing its purpose from "build storage path" to "build view path."

### 8. LakeFS snapshot alignment

**Decision:** When LakeFS is the storage backend, after each Iceberg table commit, trigger a LakeFS commit on the current branch. This aligns Iceberg snapshots with LakeFS commits, enabling version control for both content and catalog metadata.

**Rationale:** LakeFS provides branch-level isolation and atomic commits. Aligning with Iceberg snapshots means you can `git checkout` a LakeFS branch and see a consistent view of both the raw files and the catalog metadata.

## Risks / Trade-offs

| Risk                                                | Mitigation                                                                             |
| --------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Iceberg Rust 0.9.0 is pre-1.0; API changes possible | Pin exact versions; Iceberg spec is stable, so migration should be straightforward     |
| Symlinks not followed by some tools                 | Document limitation; add hardlink mode as future option for tools that need real files |
| OpenDAL LakeFS backend is read-only                 | For writes, use LakeFS REST API directly via `lakefs-client-rust` crate                |
| SQLite catalog doesn't support concurrent writers   | Use WAL mode; for multi-writer scenarios, use REST catalog with Lakekeeper             |
| Content cache can grow unbounded                    | Implement LRU eviction policy based on available disk space (future task)              |
| base58 encoding is slower than base64               | Negligible for hash encoding (32 bytes); correctness and readability matter more       |
| Iceberg tables require Arrow/Parquet dependencies   | Acceptable — Arrow is already widely used in the data ecosystem                        |

## Migration Plan

1. **Phase 1:** Add new crates (`berg10-storage-catalog`, `berg10-storage-vfs`, `berg10-warm-cache`) alongside existing `deer-storage-core`. No breaking changes to existing code.
2. **Phase 2:** Evolve `FileSaved` contract to include `content_hash` and `physical_location`. Update `deer-storage-core` tests.
3. **Phase 3:** Repurpose `path_builder.rs` for view materialization. Update its tests.
4. **Phase 4:** Wire ingestion flow through catalog + VFS. Data generators unchanged.
5. **Phase 5:** Integration tests and full workspace verification.

Rollback: Since new crates are additive, rollback is simply removing the new crates and reverting `FileSaved` to its previous shape. The existing `deer-storage-core` remains functional throughout.

## Open Questions

1. **Content cache eviction policy:** LRU by access time? LRU by size? Configurable? (Defer to implementation — start with no eviction, add later.)
2. **View checkout atomicity:** Should a checkout be atomic (all-or-nothing) or incremental (files appear as they're linked)? (Start with incremental for responsiveness; atomic can be added later.)
3. **Iceberg table partitioning:** Should `berg10.files` be partitioned by `level` or `hierarchy` for query performance? (Start unpartitioned; add partitioning when data volume warrants it.)
