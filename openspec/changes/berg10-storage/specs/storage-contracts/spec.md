## MODIFIED Requirements

### Requirement: FileSaved event carries content identity and location
The `FileSaved` event SHALL include a `content_hash` field containing the base58-encoded blake3 hash of the file payload, and a `physical_location` field containing the URI of the stored content. The `target.relative_path` field SHALL represent the view-relative path (for warm cache materialization), not the physical storage path.

#### Scenario: FileSaved includes content hash and location
- **WHEN** a file is successfully stored and registered in the catalog
- **THEN** the `FileSaved` event contains `content_hash` (base58 blake3), `physical_location` (e.g., `s3://berg10-storage/<b58hash>`), and `target` with the view-relative path

#### Scenario: FileSaved supports single-file targets
- **WHEN** a single file is saved
- **THEN** `FileSaved.target` is `SingleFile { relative_path }` with the view-relative path

#### Scenario: FileSaved supports manifest targets
- **WHEN** a multi-file commit is saved
- **THEN** `FileSaved.target` is `CommitManifest { manifest_path, member_count }` with the manifest's view-relative path
