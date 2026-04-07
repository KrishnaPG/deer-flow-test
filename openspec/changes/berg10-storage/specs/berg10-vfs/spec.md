## ADDED Requirements

### Requirement: Pluggable storage backends via OpenDAL
The VFS layer SHALL use OpenDAL (0.55.0) as the unified storage backend interface, supporting 50+ storage services including S3, GCS, Azure Blob, LakeFS, Google Drive, OneDrive, WebDAV, SFTP, HDFS, and local filesystem through a single `Operator` API.

#### Scenario: Initialize with S3 backend
- **WHEN** the VFS is configured with `backend: "s3"` and bucket credentials
- **THEN** an OpenDAL S3 Operator is created and used for all storage operations

#### Scenario: Initialize with local filesystem backend
- **WHEN** the VFS is configured with `backend: "fs"` and a root path
- **THEN** an OpenDAL Fs Operator is created for local filesystem operations

#### Scenario: Switch backend via configuration
- **WHEN** the storage backend configuration is changed from `s3` to `gcs`
- **THEN** the VFS reinitializes with a GCS Operator without requiring code changes

### Requirement: Content-addressed write
The VFS SHALL store content using the content hash as the object key. When writing a file, the system SHALL compute `base58(blake3(payload))` and use the resulting string as the storage key.

#### Scenario: Write content and retrieve by hash
- **WHEN** content is written to the VFS
- **THEN** it is stored at key `<b58_hash>` and can be retrieved using the same hash

#### Scenario: Deduplicate identical content
- **WHEN** the same content is written twice
- **THEN** only one physical object exists in storage (identified by the same content hash)

#### Scenario: Changed payload creates a new blob
- **WHEN** a payload changes
- **THEN** a new content hash is produced and the VFS stores the changed payload as a new blob rather than overwriting the previous blob

### Requirement: Content retrieval by hash
The VFS SHALL support retrieving content by its base58-encoded blake3 hash. The system SHALL return the raw bytes of the stored object.

#### Scenario: Retrieve content by hash
- **WHEN** a retrieval request is made for content hash `7bWpKq9xR3mNvHf2Tc8Yd...`
- **THEN** the VFS returns the raw bytes stored at that key

#### Scenario: Handle missing content
- **WHEN** a retrieval request is made for a hash that does not exist in storage
- **THEN** the VFS returns a not-found error

#### Scenario: Retrieval does not require physical location
- **WHEN** a caller needs to retrieve a blob
- **THEN** the caller supplies `content_hash` directly and the VFS resolves the backend object without needing any external physical location metadata

### Requirement: OpenDAL middleware layers
The VFS SHALL apply OpenDAL middleware layers for retry logic and tracing on storage operations.

#### Scenario: Automatic retry on transient failure
- **WHEN** a storage operation fails with a transient error
- **THEN** the RetryLayer automatically retries with exponential backoff

#### Scenario: Tracing on storage operations
- **WHEN** a storage operation is executed
- **THEN** the TracingLayer records OpenTelemetry spans for the operation

### Requirement: LakeFS write support
When LakeFS is the storage backend, the VFS SHALL use the LakeFS REST API for write operations that are not supported through the S3-compatible interface. Reads MAY continue to use the S3-compatible path where appropriate.

#### Scenario: Write to LakeFS branch
- **WHEN** content is written with LakeFS as the backend
- **THEN** the VFS uses the LakeFS REST API to upload the object to the current branch

### Requirement: Backend storage remains an implementation detail
The VFS SHALL expose a blob-oriented API keyed by `content_hash`. Backend-specific object paths or URIs SHALL be treated as implementation details and SHALL NOT be required by higher-level programmatic consumers.

#### Scenario: Catalog and warm cache use content_hash
- **WHEN** the catalog or warm cache needs to retrieve content
- **THEN** they use `content_hash` as the retrieval key instead of parsing backend-specific storage paths
