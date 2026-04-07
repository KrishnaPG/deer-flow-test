## MODIFIED Requirements

### Requirement: FileSaved event carries content identity and virtual target
The `FileSaved` event SHALL include a `content_hash` field containing the base58-encoded blake3 hash of the file payload. The `target.relative_path` field SHALL represent the virtual-folder-hierarchy-relative path used for warm-cache materialization, not any physical storage path.

`physical_location`, if retained, SHALL be treated as optional implementation metadata and SHALL NOT be required by programmatic consumers for retrieval.

#### Scenario: FileSaved includes content hash and virtual target
- **WHEN** a file is successfully stored and registered in the catalog
- **THEN** the `FileSaved` event contains `content_hash` (base58 blake3) and `target` with the virtual-folder-hierarchy-relative path

#### Scenario: FileSaved supports single-file targets
- **WHEN** a single file is saved
- **THEN** `FileSaved.target` is `SingleFile { relative_path }` with the virtual-folder-hierarchy-relative path

#### Scenario: FileSaved supports manifest targets
- **WHEN** a multi-file commit is saved
- **THEN** `FileSaved.target` is `CommitManifest { manifest_path, member_count }` with the manifest's virtual-folder-hierarchy-relative path

#### Scenario: Retrieval does not depend on physical location
- **WHEN** a downstream consumer needs to work with the saved content
- **THEN** it uses `content_hash` as the retrieval key and does not require `physical_location`
