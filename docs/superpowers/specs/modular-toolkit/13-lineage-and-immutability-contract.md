# Design: Modular Toolkit - Lineage And Immutability Contract

**Date:** 2026-03-31
**Status:** Draft revision

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

## Minimum Metadata Envelopes

The discovery tranche should define shared metadata envelopes equivalent to:

- `LineageMeta`
- `StoragePlacementMeta`
- `CorrelationMeta`
- `DownstreamConsumerMeta`

These can later be mapped into strongly typed Rust structs.

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
