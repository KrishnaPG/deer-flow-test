## ADDED Requirements

### Requirement: SQLite-backed blob metadata catalog
The system SHALL maintain a SQLite-backed catalog that serves as the authoritative blob metadata registry. Each row represents one immutable stored blob with its `content_hash`, semantic attributes, optional user-facing filename metadata, and lineage information.

#### Scenario: Register a new blob in the catalog
- **WHEN** a blob is successfully stored via the VFS layer
- **THEN** a row is inserted into the `blobs` table with content_hash, hierarchy, level, plane, payload_kind, payload_format, payload_size_bytes, correlation_ids, lineage_refs, routing_tags, written_at, writer_identity, and any user-facing filename metadata needed for hierarchy checkout materialization

#### Scenario: Query blobs by semantic attributes
- **WHEN** a downstream tool queries the catalog for blobs matching `payload_kind = 'mp3' AND routing_tags['singer'] = 'Adele'`
- **THEN** the catalog returns all matching blob records keyed by `content_hash`

#### Scenario: Look up blob by content hash
- **WHEN** a lookup is performed by content_hash
- **THEN** the catalog returns the single matching blob record and its associated semantic metadata

### Requirement: Immutable content records
The catalog SHALL treat stored blobs as immutable. A changed payload produces a new `content_hash` and therefore a new catalog row; existing rows are never overwritten to point at new content.

#### Scenario: Updated payload creates new row
- **WHEN** a payload changes and is ingested again
- **THEN** the new payload receives a new `content_hash` and is registered as a new blob record rather than overwriting the previous one

### Requirement: Configurable catalog backend
The catalog SHALL support pluggable backends: `sql` (default, using SQLite) and `rest` (for future external catalog servers). The backend is selected at startup from configuration.

#### Scenario: Default SQLite backend initialization
- **WHEN** the catalog is initialized with no explicit backend configuration
- **THEN** it SHALL use a SQLite database at `base_dir/catalog/berg10.sqlite`

#### Scenario: REST backend configuration (future)
- **WHEN** the catalog is configured with `backend: "rest"` and a REST URI
- **THEN** the system SHALL return a clear error indicating the REST backend is not yet implemented

### Requirement: Virtual folder hierarchy definitions table
The system SHALL maintain a catalog table for virtual folder hierarchy definitions. Each row defines a named hierarchy with its ordering, optional filter expression, and status.

#### Scenario: Create a new virtual folder hierarchy definition
- **WHEN** a user creates a hierarchy named `music-by-year` with ordering `["year", "singer"]` and filter `payload_kind = 'mp3'`
- **THEN** a row is inserted into the hierarchy definitions table with status `active`

#### Scenario: List active virtual folder hierarchies
- **WHEN** a query is executed for hierarchy definitions with status `active`
- **THEN** all active hierarchy definitions are returned with their ordering and filters

#### Scenario: Deactivate a virtual folder hierarchy
- **WHEN** a user deactivates an existing hierarchy
- **THEN** the hierarchy's status is updated to `inactive` and its checkout is removed from the warm cache

#### Scenario: Multiple hierarchies reuse the same blob set
- **WHEN** multiple hierarchy definitions match the same blob records
- **THEN** each hierarchy definition remains independent in catalog metadata while still referring to the same immutable blobs by `content_hash`

### Requirement: Programmatic access does not require hierarchy checkout
Tools and applications SHALL be able to query the blob catalog directly and work with blobs by `content_hash` without creating or traversing a virtual folder hierarchy checkout.

#### Scenario: Program reads by query and content hash
- **WHEN** an application queries the catalog to locate all matching blobs
- **THEN** it can use the returned `content_hash` values directly to retrieve content without depending on a hierarchy checkout

### Requirement: LakeFS snapshot alignment (deferred)
When the storage backend is LakeFS, the catalog SHALL eventually align catalog commits with LakeFS branch commits, ensuring that catalog snapshots correspond to LakeFS commit points.

#### Scenario: Snapshot policy remains under discussion
- **WHEN** the system is running with non-LakeFS backends
- **THEN** blob storage and catalog registration proceed without requiring an additional storage snapshot policy
