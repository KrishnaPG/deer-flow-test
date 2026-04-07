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

### Requirement: Content retrieval by hash
The VFS SHALL support retrieving content by its base58-encoded blake3 hash. The system SHALL return the raw bytes of the stored object.

#### Scenario: Retrieve content by hash
- **WHEN** a retrieval request is made for content hash `7bWpKq9xR3mNvHf2Tc8Yd...`
- **THEN** the VFS returns the raw bytes stored at that key

#### Scenario: Handle missing content
- **WHEN** a retrieval request is made for a hash that does not exist in storage
- **THEN** the VFS returns a not-found error

### Requirement: OpenDAL middleware layers
The VFS SHALL apply OpenDAL middleware layers for retry logic, tracing, and content caching on all storage operations.

#### Scenario: Automatic retry on transient failure
- **WHEN** a storage operation fails with a transient error
- **THEN** the RetryLayer automatically retries with exponential backoff

#### Scenario: Tracing on storage operations
- **WHEN** a storage operation is executed
- **THEN** the TracingLayer records OpenTelemetry spans for the operation

### Requirement: LakeFS write support
When LakeFS is the storage backend, the VFS SHALL use the LakeFS REST API for write operations (since the OpenDAL LakeFS service is read-only). Writes SHALL be performed to the current LakeFS branch.

#### Scenario: Write to LakeFS branch
- **WHEN** content is written with LakeFS as the backend
- **THEN** the VFS uses the LakeFS REST API to upload the object to the current branch
