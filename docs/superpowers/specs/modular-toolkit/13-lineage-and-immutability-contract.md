# Design: Modular Toolkit - Lineage And Immutability Contract

**Date:** 2026-03-31
**Status:** Approved

## Why This File Exists

Stable IDs are not enough.

The toolkit must preserve immutable provenance and append-only lineage.

## Core Rule

Important records are append-only in meaning.

New observations, transforms, predictions, and prescriptions append new lineage;
they do not rewrite prior truth.

## Required Provenance Dimensions

Each important record family should be able to carry:

- record identity
- source record identities
- source event identities
- content hash or storage object hash
- storage URI or object reference
- transform stage
- transform version or strategy identifier
- producer identity
- correlation identifiers

## Correlation Contract

The toolkit should preserve correlation fields compatible with the state-server
observability contract:

- `mission_id`
- `run_id`
- `agent_id`
- `artifact_id`
- `trace_id`

Additional correlation is allowed, such as:

- `thread_id`
- `task_id`
- `chunk_hash`
- `parent_event_id`

## Lineage Chain

A record should be able to answer:

- what source object or event produced me?
- what transform produced me?
- what later records depend on me?
- what view, insight, prediction, or prescription chain am I part of?

## Metadata Envelope Rule

Shared metadata envelopes must be composable and immutable-safe.

### Universal Envelopes

All first-class record families should carry:

- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`

### Conditional Envelopes

Families may additionally carry, when applicable:

- `LevelMeta`
- `RepresentationMeta`
- `TransformMeta`
- `PolicyVisibilityMeta`
- `ReadinessMeta`

## Immutable-Safe Rule

Metadata that changes over time must not be modeled as an in-place update to an
existing record.

That means:

- readiness/progress changes append new records
- exclusion or suppression appends new records
- conflict resolution appends new records
- forward links are reconstructed by query when possible

Mutable status must not be smuggled into old records under the name of metadata.

Storage-native truth records must not expose model-level `UPDATE` or `DELETE`
semantics that rewrite, replace, or physically remove previously written truth
records.

If truth needs correction, redaction, suppression, or supersession, the model
must append a new governance or lineage-bearing record that points at the prior
truth record and states the corrective effect.

The prior storage-native truth record remains immutable and queryable as the
lineage anchor, even when mediated reads later hide or qualify it.

## View Requirement

Derived view models should keep lineage, even if compacted.

At minimum, a view should retain enough metadata to support:

- drill-back to exact source records
- drill-sideways to sibling transforms
- drill-forward to downstream consumers or outcomes

## Projection Requirement

World projection and replay must preserve backlinks to their source records.

If a world marker or replay event cannot be traced back to source records and
transform steps, the system has lost provenance.

## Anti-Drift Rule

Do not define canonical/domain records as content-only structs.

They must be provenance-bearing structs.
