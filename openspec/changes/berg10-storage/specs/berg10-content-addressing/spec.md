## ADDED Requirements

### Requirement: Blake3 content hashing
The system SHALL compute the blake3 hash of file payloads to produce a 32-byte digest. This digest serves as the content's unique identifier.

#### Scenario: Hash a file payload
- **WHEN** a file payload is hashed with blake3
- **THEN** a deterministic 32-byte digest is produced

#### Scenario: Identical content produces identical hash
- **WHEN** the same content is hashed twice
- **THEN** the resulting digests are identical

#### Scenario: Different content produces different hash
- **WHEN** two different payloads are hashed
- **THEN** the resulting digests are different

### Requirement: Base58 hash encoding
The system SHALL encode the 32-byte blake3 digest as a base58 string. The base58-encoded string SHALL be used as the content key everywhere in the system (catalog, storage, warm cache).

#### Scenario: Encode hash as base58
- **WHEN** a 32-byte blake3 digest is base58-encoded
- **THEN** the result is a human-readable string containing only base58 characters (no 0, O, I, l)

#### Scenario: Decode base58 to original hash
- **WHEN** a base58-encoded hash is decoded
- **THEN** the original 32-byte blake3 digest is recovered

### Requirement: Content address format
All content addresses throughout the system SHALL use the base58-encoded blake3 hash format. The format SHALL be `b58:<base58_string>` when a prefix is needed for disambiguation, and `<base58_string>` when the context is clear.

#### Scenario: Content key in catalog
- **WHEN** a file is registered in the `berg10.files` table
- **THEN** the `content_hash` column contains the base58-encoded blake3 hash without prefix

#### Scenario: Content key in storage
- **WHEN** content is stored in the VFS layer
- **THEN** the object key is the base58-encoded blake3 hash

#### Scenario: Content key in warm cache
- **WHEN** a content blob is cached locally
- **THEN** the local warm-cache blob path is derived from the base58-encoded blake3 hash

### Requirement: Content hash is the source of truth
The base58-encoded blake3 hash SHALL be the canonical identity for a blob across ingestion, storage, catalog lookup, warm-cache fetch, and programmatic retrieval.

#### Scenario: Programmatic retrieval by content hash
- **WHEN** a tool or application needs to retrieve content directly
- **THEN** it uses `content_hash` as the authoritative lookup key without requiring any virtual folder hierarchy or physical storage path

### Requirement: Immutable content addressing
Stored content SHALL be immutable. The same `content_hash` always refers to the same bytes, and any changed payload SHALL produce a new `content_hash`.

#### Scenario: Immutable blob identity
- **WHEN** a blob has content hash `abc123...`
- **THEN** that hash always resolves to the same bytes and is never overwritten with different bytes

### Requirement: Sharded warm-cache blob path
The local warm-cache content store SHALL shard blob paths using the content hash as `base_dir/content/<hash[0..2]>/<hash[2..4]>/<hash[4..]>.blob`.

#### Scenario: Shard local blob path by content hash
- **WHEN** content with hash `7bWpKq9xR3mNvHf2Tc8Yd...` is cached locally
- **THEN** the blob is written under `base_dir/content/7b/Wp/Kq9xR3mNvHf2Tc8Yd....blob`
