# Design: Modular Toolkit - Correlation And Identity Contract

**Date:** 2026-03-31
**Status:** Approved

## Required Correlation Fields

- `mission_id`
- `run_id`
- `agent_id`
- `artifact_id`
- `trace_id`

## Additional Common IDs

- `thread_id`
- `task_id`
- `event_id`
- `chunk_hash`
- `as_is_hash`
- `embedding_basis_hash`

## Rule

Every major record family must declare:

- primary identity
- parent/source identity references
- correlation fields it requires
- source cardinality assumptions

For `as_is`, `chunks`, and `embeddings` planes, primary identity must include
the relevant hash anchor, not only a synthetic UUID or row identifier.

`EmbeddingRecord` identity must be based on the exact chunk payload basis used
for vectorization, together with its parent `chunk_hash` and parent
`as_is_hash`.

Identity-bearing truth must remain immutable once written.

Conditional runtime/readiness state should not be treated as record identity.
