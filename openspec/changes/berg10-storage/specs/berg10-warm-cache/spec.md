## ADDED Requirements

### Requirement: View checkout as symlink tree
The warm cache SHALL materialize a view definition as a symlink tree under `base_dir/checkouts/<view-name>/`. Each symlink SHALL point to the corresponding content blob in `base_dir/content/<b58hash>`. The directory structure within the checkout follows the view's hierarchy_order attribute array.

#### Scenario: Checkout a view with hierarchy [year, singer]
- **WHEN** a view named `music-by-year` with hierarchy_order `["year", "singer"]` is checked out
- **THEN** symlinks are created at `base_dir/checkouts/music-by-year/2024/Adele/song.mp3` pointing to `base_dir/content/<b58hash>`

#### Scenario: Checkout a view with reversed hierarchy [singer, year]
- **WHEN** a view named `music-by-singer` with hierarchy_order `["singer", "year"]` is checked out
- **THEN** symlinks are created at `base_dir/checkouts/music-by-singer/Adele/2024/song.mp3` pointing to the same content blobs as the year-first view

#### Scenario: Multiple views checked out simultaneously
- **WHEN** both `music-by-year` and `music-by-singer` views are checked out
- **THEN** both symlink trees exist under `base_dir/checkouts/` and reference the same underlying content blobs

### Requirement: View filter application
When materializing a view, the warm cache SHALL apply the view's filter expression to select which files from `berg10.files` are included in the checkout.

#### Scenario: Filter by payload_kind
- **WHEN** a view with filter `payload_kind = 'mp3'` is checked out
- **THEN** only files with `payload_kind = 'mp3'` are included in the symlink tree

#### Scenario: Filter by routing tags
- **WHEN** a view with filter `routing_tags['mission_id'] = 'mission_7'` is checked out
- **THEN** only files tagged with that mission_id are included

### Requirement: View lifecycle management
The warm cache SHALL support listing, activating, deactivating, and removing view checkouts. The UI SHALL be able to manage currently active checkouts through the warm cache API.

#### Scenario: List active checkouts
- **WHEN** the list checkouts API is called
- **THEN** it returns all views with status `active` and their checkout paths

#### Scenario: Deactivate a checkout
- **WHEN** a view checkout is deactivated
- **THEN** the symlink tree under `base_dir/checkouts/<view-name>/` is removed and the view status is set to `inactive`

#### Scenario: Remove a checkout
- **WHEN** a checkout is removed
- **THEN** the symlink tree is deleted and the view definition is removed from `berg10.views`

### Requirement: Content cache population
The warm cache SHALL ensure that content blobs exist in `base_dir/content/` before creating symlinks. If a content blob is not present locally, it SHALL be fetched from cold storage via the VFS layer.

#### Scenario: Lazy content fetch on checkout
- **WHEN** a view is checked out and a file's content blob is not in the local content cache
- **THEN** the blob is fetched from cold storage via the VFS and stored at `base_dir/content/<b58hash>` before the symlink is created

#### Scenario: Reuse cached content
- **WHEN** a view is checked out and the content blob already exists in `base_dir/content/`
- **THEN** no fetch from cold storage is performed; the existing blob is used

### Requirement: Third-party tool compatibility
The warm cache checkout directories SHALL be readable by third-party tools (DuckDB, ClickHouse, etc.) as regular filesystem directories. Symlinks SHALL be followed by default by these tools.

#### Scenario: DuckDB reads from checkout directory
- **WHEN** DuckDB executes `SELECT * FROM read_parquet('base_dir/checkouts/all-transcripts/**/*.parquet')`
- **THEN** it successfully reads all parquet files through the symlinks
