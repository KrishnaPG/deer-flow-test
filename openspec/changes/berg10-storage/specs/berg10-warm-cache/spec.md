## ADDED Requirements

### Requirement: Virtual folder hierarchy checkout as symlink tree
The warm cache SHALL materialize a virtual folder hierarchy as a symlink tree under `base_dir/checkouts/<hierarchy-name>/`. Each symlink SHALL point to a content blob stored in the local warm-cache content store. The directory structure within the checkout follows the user-defined hierarchy ordering.

#### Scenario: Checkout a virtual folder hierarchy with ordering [year, singer]
- **WHEN** a hierarchy named `music-by-year` with ordering `["year", "singer"]` is checked out
- **THEN** symlinks are created at `base_dir/checkouts/music-by-year/2024/Adele/song1.mp3` pointing to the hashed blob path under `base_dir/content/`

#### Scenario: Checkout a virtual folder hierarchy with reversed ordering [singer, year]
- **WHEN** a hierarchy named `music-by-singer` with ordering `["singer", "year"]` is checked out
- **THEN** symlinks are created at `base_dir/checkouts/music-by-singer/Adele/2024/song1.mp3` pointing to the same underlying blobs as the year-first hierarchy

#### Scenario: Multiple virtual folder hierarchies checked out simultaneously
- **WHEN** both `music-by-year` and `music-by-singer` hierarchies are checked out
- **THEN** both symlink trees exist under `base_dir/checkouts/` and reference the same underlying blobs in the warm-cache content store

### Requirement: Virtual folder hierarchy filter application
When materializing a virtual folder hierarchy, the warm cache SHALL apply that hierarchy's filter expression to select which blobs from the catalog are included in the checkout.

#### Scenario: Filter by payload_kind
- **WHEN** a hierarchy with filter `payload_kind = 'mp3'` is checked out
- **THEN** only blobs with `payload_kind = 'mp3'` are included in the symlink tree

#### Scenario: Filter by routing tags
- **WHEN** a hierarchy with filter `routing_tags['mission_id'] = 'mission_7'` is checked out
- **THEN** only blobs tagged with that mission_id are included

### Requirement: Virtual folder hierarchy lifecycle management
The warm cache SHALL support listing, activating, deactivating, and removing virtual folder hierarchy checkouts. The UI SHALL be able to manage currently active checkouts through the warm cache API.

#### Scenario: List active checkouts
- **WHEN** the list checkouts API is called
- **THEN** it returns all virtual folder hierarchies with status `active` and their checkout paths

#### Scenario: Deactivate a checkout
- **WHEN** a hierarchy checkout is deactivated
- **THEN** the symlink tree under `base_dir/checkouts/<hierarchy-name>/` is removed and the hierarchy status is set to `inactive`

#### Scenario: Remove a checkout
- **WHEN** a checkout is removed
- **THEN** the symlink tree is deleted and the hierarchy definition is removed from the catalog metadata store

### Requirement: Content cache population
The warm cache SHALL ensure that content blobs exist in `base_dir/content/` before creating symlinks. If a content blob is not present locally, it SHALL be fetched from cold storage via the VFS layer by `content_hash`.

The local warm-cache blob path SHALL be sharded by hash as `base_dir/content/<hash[0..2]>/<hash[2..4]>/<hash[4..]>.blob`.

#### Scenario: Lazy content fetch on checkout
- **WHEN** a hierarchy is checked out and a blob's content is not in the local content cache
- **THEN** the blob is fetched from cold storage via the VFS using `content_hash` and stored at `base_dir/content/<hash[0..2]>/<hash[2..4]>/<hash[4..]>.blob` before the symlink is created

#### Scenario: Reuse cached content
- **WHEN** a hierarchy is checked out and the content blob already exists in `base_dir/content/`
- **THEN** no fetch from cold storage is performed; the existing blob is used

#### Scenario: Same blob reused across multiple hierarchies
- **WHEN** the same `content_hash` appears in multiple checked out hierarchies
- **THEN** only one blob copy exists in the warm-cache content store and all hierarchy checkouts symlink to that blob path

### Requirement: User-facing filenames in hierarchy checkouts
The leaf filename exposed in a checked out virtual folder hierarchy SHALL be a user-facing logical filename with the correct file extension when that metadata is available. The warm-cache blob filename remains hash-derived and internal.

#### Scenario: User sees original-style filename
- **WHEN** a blob with logical filename `song1.mp3` is included in a checked out hierarchy
- **THEN** the symlink path exposed to the user ends in `song1.mp3` while the symlink target points to `base_dir/content/<hash[0..2]>/<hash[2..4]>/<hash[4..]>.blob`

### Requirement: Virtual folder hierarchies are optional user convenience
Virtual folder hierarchies SHALL exist for direct human access and interoperability with filesystem-oriented tools. Tools and apps SHALL be able to work directly with catalog metadata or `content_hash` without requiring any hierarchy checkout.

#### Scenario: Programmatic access bypasses hierarchy checkout
- **WHEN** an application needs to work with stored content programmatically
- **THEN** it MAY query the catalog or fetch blobs directly by `content_hash` without creating any virtual folder hierarchy checkout

### Requirement: Third-party tool compatibility
The warm cache checkout directories SHALL be readable by third-party tools (DuckDB, ClickHouse, etc.) as regular filesystem directories. Symlinks SHALL be followed by default by these tools.

#### Scenario: DuckDB reads from checkout directory
- **WHEN** DuckDB executes `SELECT * FROM read_parquet('base_dir/checkouts/all-transcripts/**/*.parquet')`
- **THEN** it successfully reads all parquet files through the symlinks
