## ADDED Requirements

### Requirement: Iceberg file registry
The system SHALL maintain an Iceberg table named `berg10.files` that serves as the authoritative content registry. Each row represents one immutable stored blob with its `content_hash`, semantic attributes, optional user-facing filename metadata, and lineage information.

#### Scenario: Register a new file in the catalog
- **WHEN** a file is successfully stored via the VFS layer
- **THEN** a row is inserted into `berg10.files` with content_hash, hierarchy, level, plane, payload_kind, payload_format, payload_size_bytes, correlation_ids, lineage_refs, routing_tags, written_at, writer_identity, and any user-facing filename metadata needed for hierarchy checkout materialization

#### Scenario: Query files by semantic attributes
- **WHEN** a downstream tool queries `SELECT * FROM berg10.files WHERE payload_kind = 'mp3' AND routing_tags['singer'] = 'Adele'`
- **THEN** the catalog returns all matching file records keyed by `content_hash`

#### Scenario: Look up file by content hash
- **WHEN** a lookup is performed by content_hash
- **THEN** the catalog returns the single matching file record and its associated semantic metadata

### Requirement: Immutable content records
The catalog SHALL treat stored blobs as immutable. A changed payload produces a new `content_hash` and therefore a new catalog row; existing rows are never overwritten to point at new content.

#### Scenario: Updated payload creates new row
- **WHEN** a payload changes and is ingested again
- **THEN** the new payload receives a new `content_hash` and is registered as a new blob record rather than overwriting the previous one

### Requirement: Configurable catalog backend
The catalog SHALL support pluggable backends: `sql` (default, using `iceberg-catalog-sql` with SQLite) and `rest` (using `iceberg-catalog-rest` to connect to Lakekeeper/Polaris). The backend is selected at startup from configuration.

#### Scenario: Default SQLite backend initialization
- **WHEN** the catalog is initialized with no explicit backend configuration
- **THEN** it SHALL use `iceberg-catalog-sql` with a SQLite database at `base_dir/catalog/iceberg.sqlite`

#### Scenario: REST backend configuration
- **WHEN** the catalog is configured with `backend: "rest"` and a REST URI
- **THEN** it SHALL use `iceberg-catalog-rest` to connect to the specified Lakekeeper/Polaris endpoint

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
Tools and applications SHALL be able to query `berg10.files` directly and work with blobs by `content_hash` without creating or traversing a virtual folder hierarchy checkout.

#### Scenario: Program reads by query and content hash
- **WHEN** an application queries `berg10.files` to locate all matching blobs
- **THEN** it can use the returned `content_hash` values directly to retrieve content without depending on a hierarchy checkout

### Requirement: Time-travel queries
The catalog SHALL support Iceberg time-travel queries, allowing retrieval of file and virtual folder hierarchy metadata as of a specific snapshot or timestamp.

#### Scenario: Query files as of a past timestamp
- **WHEN** a query includes `TIMESTAMP AS OF '2026-04-01T00:00:00Z'`
- **THEN** the catalog returns file records as they existed at that timestamp

### Requirement: LakeFS snapshot alignment
When the storage backend is LakeFS, the catalog SHALL align Iceberg table commits with LakeFS branch commits, ensuring that Iceberg snapshots correspond to LakeFS commit points.

#### Scenario: Align Iceberg commit with LakeFS commit
- **WHEN** an Iceberg table commit completes and the backend is LakeFS
- **THEN** a LakeFS commit is triggered on the current branch with a message referencing the Iceberg snapshot ID

#### Scenario: Snapshot policy remains under discussion
- **WHEN** the system is running with non-LakeFS backends
- **THEN** blob storage and catalog registration proceed without requiring an additional storage snapshot policy
