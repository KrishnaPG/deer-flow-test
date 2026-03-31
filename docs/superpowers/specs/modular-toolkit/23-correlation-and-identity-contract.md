# Design: Modular Toolkit - Correlation And Identity Contract

**Date:** 2026-03-31
**Status:** Draft revision

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

## Rule

Every major record family must declare:

- primary identity
- parent/source identity references
- correlation fields it requires
- source cardinality assumptions

Identity-bearing truth must remain immutable once written.

Conditional runtime/readiness state should not be treated as record identity.
