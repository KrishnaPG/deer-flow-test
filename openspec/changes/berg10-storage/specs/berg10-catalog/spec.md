## ADDED Requirements

### Requirement: Iceberg file registry
The system SHALL maintain an Iceberg table named `berg10.files` that serves as the authoritative content registry. Each row represents one stored file with its content hash, semantic attributes, physical location, and lineage information.

#### Scenario: Register a new file in the catalog
- **WHEN** a file is successfully stored via the VFS layer
- **THEN** a row is inserted into `berg10.files` with content_hash, hierarchy, level, plane, payload_kind, payload_format, payload_size_bytes, physical_location, correlation_ids, lineage_refs, routing_tags, written_at, and writer_identity

#### Scenario: Query files by semantic attributes
- **WHEN** a downstream tool queries `SELECT * FROM berg10.files WHERE payload_kind = 'mp3' AND routing_tags['singer'] = 'Adele'`
- **THEN** the catalog returns all matching file records with their physical locations

#### Scenario: Look up file by content hash
- **WHEN** a lookup is performed by content_hash
- **THEN** the catalog returns the single matching file record including its physical_location

### Requirement: Configurable catalog backend
The catalog SHALL support pluggable backends: `sql` (default, using `iceberg-catalog-sql` with SQLite) and `rest` (using `iceberg-catalog-rest` to connect to Lakekeeper/Polaris). The backend is selected at startup from configuration.

#### Scenario: Default SQLite backend initialization
- **WHEN** the catalog is initialized with no explicit backend configuration
- **THEN** it SHALL use `iceberg-catalog-sql` with a SQLite database at `base_dir/catalog/iceberg.sqlite`

#### Scenario: REST backend configuration
- **WHEN** the catalog is configured with `backend: "rest"` and a REST URI
- **THEN** it SHALL use `iceberg-catalog-rest` to connect to the specified Lakekeeper/Polaris endpoint

### Requirement: Iceberg view definitions table
The system SHALL maintain an Iceberg table named `berg10.views` that stores virtual hierarchy definitions. Each row defines a named view with its hierarchy ordering, optional filter expression, and status.

#### Scenario: Create a new view definition
- **WHEN** a user creates a view named `music-by-year` with hierarchy_order `["year", "singer"]` and filter `payload_kind = 'mp3'`
- **THEN** a row is inserted into `berg10.views` with status `active`

#### Scenario: List active views
- **WHEN** a query is executed: `SELECT * FROM berg10.views WHERE status = 'active'`
- **THEN** all active view definitions are returned with their hierarchy orderings and filters

#### Scenario: Deactivate a view
- **WHEN** a user deactivates an existing view
- **THEN** the view's status is updated to `inactive` and its checkout is removed from the warm cache

### Requirement: Time-travel queries
The catalog SHALL support Iceberg time-travel queries, allowing retrieval of file and view metadata as of a specific snapshot or timestamp.

#### Scenario: Query files as of a past timestamp
- **WHEN** a query includes `TIMESTAMP AS OF '2026-04-01T00:00:00Z'`
- **THEN** the catalog returns file records as they existed at that timestamp

### Requirement: LakeFS snapshot alignment
When the storage backend is LakeFS, the catalog SHALL align Iceberg table commits with LakeFS branch commits, ensuring that Iceberg snapshots correspond to LakeFS commit points.

#### Scenario: Align Iceberg commit with LakeFS commit
- **WHEN** an Iceberg table commit completes and the backend is LakeFS
- **THEN** a LakeFS commit is triggered on the current branch with a message referencing the Iceberg snapshot ID
