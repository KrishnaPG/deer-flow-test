# Design: Modular Toolkit - Retrieval And Indexing Contract

**Date:** 2026-03-31
**Status:** Approved

## Why This File Exists

The toolkit needs to respect the tri-plane storage and mediated-read model when
supporting search, retrieval, replay, and artifact inspection.

## Retrieval Rule

The toolkit should assume that read surfaces may be backed by:

- hot-cache/live event windows
- warm-cache mediated responses
- DB/index views over storage
- direct authorized pointers to storage objects

These are not interchangeable and should be modeled explicitly.

## Tri-Plane Interpretation

The planes map to explicit typed representation/index extensions.

### As-Is Plane

Use for:

- original payload access
- canonical file preview
- provenance anchor

Representation extension:

- `AsIsRepresentationRecord`

### Chunks Plane

Use for:

- segmented search results
- timeline segments
- transcript slices
- partial content previews

Representation extension:

- `ChunkRecord`

### Embeddings Plane

Use for:

- semantic search
- nearest-neighbor discovery
- retrieval hints

Representation extension:

- `EmbeddingRecord`

Rule:

- embeddings point back to chunks, and chunks point back to As-Is truth
- embeddings are representation/index records only, never source truth
- `AsIsRepresentationRecord`, `ChunkRecord`, and `EmbeddingRecord` identities
  must preserve their hash-anchor chain end to end
- embedding identity must include the exact embedding basis hash for the chunk
  payload used to generate the vector, not only a parent pointer or ordinal

## Canonical Contract Needs

Records used by retrieval-aware views should preserve:

- source semantic or carrier record references
- linked `AsIsRepresentationRecord` references
- linked `ChunkRecord` references where chunking is justified
- linked `EmbeddingRecord` references where embedding/indexing is justified
- `as_is_hash`, `chunk_hash`, and `embedding_basis_hash` where applicable
- chunking strategy
- retrieval/index provider metadata
- readiness state

## Mediated Read Requirement

Artifact, replay, and search-oriented views should be designed around mediated
responses such as:

- authorized previews
- chunk-based summaries
- search hits with backlinks
- replay windows
- semantic recall results

Not direct local file assumptions.

## UI Implication

Inspectors and shelves should be able to show:

- where the visible payload came from
- whether it is a full source object, a chunk, or an embedding-backed retrieval
  result
- whether the payload is ready, indexed, or still in progress

## Anti-Drift Rule

Do not model chunking and embeddings as hidden implementation details.

They affect what the user is actually seeing and how trustworthy or partial that
view is.

Do not assume every semantic level automatically gets chunked and embedded.
