## Why

The current storage service computes hierarchical file paths at ingestion time and treats the path as the file's identity. This couples semantic classification (hierarchy, level, plane, payload kind) directly to physical storage layout, making it impossible to:

- **Reorganize data without moving files** — a user who wants to browse MP3s by `<year>/<singer>` today and `<singer>/<year>` tomorrow cannot do so.
- **Mount virtual hierarchies as tables** — downstream tools (DuckDB, ClickHouse) cannot query files by arbitrary attributes because the attributes are encoded in directory paths, not in a queryable catalog.
- **Use pluggable storage backends** — the path builder assumes a directory-like structure, locking us into S3-compatible key namespaces.

The solution is to introduce **Berg10 Storage**: a content-addressed virtual filesystem backed by an Iceberg catalog, with a local warm cache that materializes user-defined views as symlink trees. Data generators (DeerFlow, agents, workers) continue emitting the same append requests they do today — they never learn where or how bytes are physically stored.

## What Changes

- **Introduce `berg10-storage-catalog` crate** — an Iceberg-backed content registry that stores file metadata (content hash, semantic attributes, physical location, lineage) as Iceberg tables. Catalog backend is configurable: SQLite (default, embedded) or REST (Lakekeeper/Polaris for production).
- **Introduce `berg10-storage-vfs` crate** — a virtual filesystem layer using OpenDAL for pluggable storage backends (S3, GCS, Azure, LakeFS, Google Drive, OneDrive, WebDAV, SFTP, and 50+ more via a single package). Content is addressed by `base58(blake3(payload))`.
- **Introduce `berg10-warm-cache` crate** — manages local filesystem checkouts under `base_dir/checkouts/`. Users define views (virtual hierarchies) as Iceberg table rows; checking out a view materializes it as a symlink tree. Multiple views can be active simultaneously.
- **Evolve `FileSaved` contract** — add `content_hash` (base58-encoded blake3) and `physical_location` fields. The existing `relative_path` becomes view-relative, not physical.
- **Repurpose `path_builder.rs`** — no longer builds storage paths; becomes a view path builder for warm cache materialization.
- **Keep `deer-storage-core` as the ingestion protocol** — validation, admission control, durable publish, topic routing, and downstream handoff all remain unchanged. Data generators continue working exactly as before.
- **Align Iceberg snapshots with LakeFS commits** — when LakeFS is the storage backend, Iceberg table snapshots are coordinated with LakeFS branch commits for full version control of both content and catalog metadata.

## Capabilities

### New Capabilities

- `berg10-catalog`: Iceberg-backed content registry with configurable catalog backend (SQLite default, REST optional). Stores file metadata as queryable tables (`berg10.files`, `berg10.views`).
- `berg10-vfs`: Virtual filesystem with pluggable storage backends via OpenDAL. Content-addressed storage using base58-encoded blake3 hashes.
- `berg10-warm-cache`: Local filesystem view materialization. Manages user-defined virtual hierarchies as symlink trees under `base_dir/checkouts/`. Supports multiple simultaneous checkouts.
- `berg10-content-addressing`: Content hashing (blake3), base58 encoding, and content-addressed storage/retrieval.

### Modified Capabilities

- `storage-contracts`: `FileSaved` gains `content_hash` and `physical_location` fields. `relative_path` becomes optional/view-relative rather than physical.

## Impact

- **New crates**: `berg10-storage-catalog`, `berg10-storage-vfs`, `berg10-warm-cache` added to workspace
- **Modified crates**: `deer-foundation-contracts` (storage contracts), `deer-storage-core` (path builder repurposed)
- **New dependencies**: `iceberg` (0.9.0), `iceberg-catalog-sql` (0.9.0), `iceberg-catalog-rest` (0.9.0), `iceberg-storage-opendal` (0.9.0), `opendal` (0.55.0), `blake3` (1.8.4), `bs58` (0.5.1), `iceberg-datafusion` (0.9.0)
- **Breaking**: `FileSaved` contract gains new required fields — downstream consumers must handle `content_hash` and `physical_location`
- **Non-breaking**: DeerFlow driver and all data generators continue emitting the same `AppendDataRequest` — no changes needed
