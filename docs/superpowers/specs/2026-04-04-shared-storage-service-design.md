# Shared Storage Service Design

**Date:** 2026-04-04
**Status:** Approved for planning
**Scope:** Define the shared storage service boundary, write contract, commit semantics, downstream handoff, and DeerFlow backend-driver stress test.

## Why This Exists

The system needs a shared storage service that can accept immutable writes from generators and workers, persist them with strong durability and low latency, and hand off committed storage truth to downstream exposure and indexing services.

This spec defines that boundary before implementation.

## Sources Reviewed

- `docs/architecture/state-server.md`
- `docs/architecture/stess-tests.md`
- `docs/architecture/troubleshoot.md`
- `docs/architecture/dual-data-heirarchies.md`
- `docs/superpowers/specs/modular-toolkit/42-new-generator-onboarding-contract.md`
- `docs/superpowers/specs/modular-toolkit/02-workspace-and-layering.md`

## Core Boundary

The shared storage service ends at:

- immutable storage commit
- durable event/log handoff
- `FileSaved` signaling for downstream consumers

The shared storage service does not own:

- A/B/C business semantics
- UI, state-server, or app behavior
- world projection or rendering
- Iceberg publication
- LakeFS versioning/catalog
- ClickHouse exposure
- Milvus indexing
- SQL view creation
- content search or vector search

## Hard Invariants

- storage is immutable once written
- stored objects are never updated in place
- stored objects are never deleted as part of ordinary operation
- repair means append new corrective facts or overlays, never mutate prior truth
- exclusions, invalidations, supersessions, and reprocess requests are modeled as new immutable writes
- the primary performance target is ultra-low-latency, tightly coupled, in-proc usage
- the core library prefers zero-copy or minimal-copy paths wherever possible

## LiveKit Exception Path

Real-time media ingress may bypass the storage service entirely through LiveKit egress or similar direct media transports.

In that path:

- LiveKit writes raw media directly to object-storage landing zones
- later hash/promote steps bring landed media into canonical `L0` truth

This is a parallel ingress lane for ultra-low-latency media, not a contradiction of the main storage-service contract.

## Durable Event Backbone

The storage service uses Redpanda with Tiered Storage as the durable event backbone.

It does not use NATS JetStream as the core WAL/durable ingress queue.

### Topic Classes

Start with one topic per class, partitioned by a stable routing key such as `mission_id`.

1. `write-intent`
   - durable ingest requests and commit intents
2. `progress-lifecycle`
   - internal lifecycle and diagnostic signals
3. `derivation-trigger`
   - post-commit signals for derived-level workers
4. `control-intent`
   - exclusions, invalidations, reprocess requests, overrides, and repairs

All topic classes obey the immutable-storage rule: no event implies in-place update or deletion of committed truth.

## Ingress Model

The storage service is library-first.

- primary interface: in-proc library/package
- optional adapters: HTTP, WebSocket, gRPC

Ingress modes:

- small payloads may be submitted inline
- large payloads are submitted by reference

All adapters translate into the same append-only core contract.

## Layout Dimensions

The storage service does not invent hierarchy semantics, but it does use caller-supplied dimensions to build physical layout.

### Fixed Core Dimensions

- `hierarchy`
- `level`
- `plane`
- `payload_kind`
- `format`

### Extensible Layout Dimensions

- ordered partition tags used for path construction

### Non-Layout Metadata

- opaque annotations stored immutably but not used in path construction

Examples of storage-used dimensions include:

- hierarchy tags such as `A`, `B`, `C`
- levels `L0`..`L6`
- planes such as `as-is`, `chunks`, `embeddings`
- payload kinds such as `metric`, `history`, `chat-note`, `artifact`, `thumbnail`
- partition tags such as `64x64`, locale, shard, tenant, or date bucket

## Path Construction

Canonical path construction is 100% storage-owned.

Backend drivers do not construct canonical storage paths and do not provide suffix hints.

### Canonical Path Rule

All internally stored path references are relative paths.

- the storage base/root is deployment-local configuration
- layout version is not encoded into the path name

Representative path shape:

```text
/<hierarchy>/<level>/<plane>/<payload_kind>/<partition...>/<content_hash>.<ext>
```

Examples:

- `/B/L4/as-is/thumbnail/64x64/<hash>.png`
- `/A/L0/as-is/chat-note/<mission_id>/<hash>.jsonl`
- `/B/L3/chunks/transcript/<date_bucket>/<hash>.parquet`

## Append Contract

The storage library exposes two append command families:

- `append_data`
- `append_control`

### `append_data`

Carries:

- layout dimensions
- identity metadata
- lineage metadata
- correlation metadata
- payload descriptor
- opaque immutable metadata

Minimum fields:

- `layout`: hierarchy, level, plane, payload kind, format, partition tags
- `identity`: idempotency key, content hash if known, writer identity
- `lineage`: parent refs, derived-from refs
- `correlation`: `mission_id`, `run_id`, `agent_id`, `artifact_id`, `trace_id`
- `payload`: inline bytes or external reference
- `metadata`: opaque immutable annotations

### `append_control`

Carries the same identity/correlation envelope plus:

- control intent kind
- target refs to prior immutable truth
- optional rationale, policy, or operator metadata

Examples:

- exclusion
- supersession
- reprocess
- invalidation
- backfill
- override request

## Acknowledgement Semantics

Public write semantics are intentionally minimal.

### `FileAccepted`

The request was validated and durably handed to Redpanda.

### `FileSaved`

The canonical file or logical file batch is durably present in immutable storage.

Callers never block for save readiness.

- synchronous result: accepted or rejected at admission time
- asynchronous confirmation: `FileSaved`

If a caller needs confirmation, it listens for `FileSaved` or polls the durable event stream/queue.

## QoS and Idempotency

The system supports multiple write scenarios:

- fire-and-forget writes
- slow durable artifact writes
- low-latency real-time ingestion writes

The API exposes composable QoS policy objects plus a small set of predefined templates.

However:

- callers never block on save state
- idempotency is always required
- caller-facing backpressure knobs are not exposed

Backpressure remains an internal storage-service control-plane concern.

## Public vs Internal Lifecycle

Internal queueing, worker pickup, retries, and similar mechanics are not first-class public API concepts.

Not part of the public API:

- `Queued`
- `WriteStarted`
- `WriteProgress`
- internal retry counts
- worker handoff details

Those remain internal diagnostics for operators and debugging.

## Atomic Commit Model

There are two commit paths.

### `InlineCommitPath`

- for small inline payloads
- bytes come from caller memory or adapter payload
- storage hashes, durably logs, and writes canonical immutable storage objects

### `ExternalRefCommitPath`

- for large payloads provided by reference
- storage validates and claims the reference, then promotes it into managed immutable storage
- default behavior is canonical copy/promote into storage managed by the storage service

`FileSaved` always means the canonical object/file exists at its immutable canonical storage path.

It never means merely queued, claimed, staged, or temporarily visible.

## Multi-File Atomicity

True atomic commit is straightforward for a single file/object.

For a logical write that materializes as multiple files:

- atomicity is not claimed at raw object-put level
- atomic visibility is provided through a final immutable commit manifest/marker

This means:

- member files may exist before the logical batch is visible
- partially landed member files without a manifest are not visible as committed logical truth
- `FileSaved` for a multi-file logical write means the manifest/marker exists and references a complete immutable member set

## Derivation Signaling

Derivation workers consume post-commit trigger events rather than relying primarily on blind storage scans.

Rules:

- derivation triggers are emitted only after committed logical truth exists
- multi-file writes trigger from the final commit manifest/marker, not partial member files
- no trigger is emitted for invisible partial files

Start with a generic trigger schema and route/filter by metadata.

## Downstream Handoff

Downstream exposure and indexing services consume `FileSaved` events.

Examples of downstream responsibilities:

- Iceberg/ClickHouse publisher for structured Hierarchy A data
- LakeFS or file-catalog service for Hierarchy B versioning/tracking
- chunking workers over `as-is` artifacts
- embedding workers over chunk outputs
- Milvus sync/index workers over embeddings
- shell/view services that create SQL views or search projections

The storage service does not own those downstream policies.

## `FileSaved` Event Contract

`FileSaved` is the handoff boundary from storage truth to downstream indexing, catalog, versioning, chunking, embedding, and query exposure.

It must carry at least:

- logical write id
- relative committed path or commit-manifest path
- hierarchy, level, plane, payload kind, format
- correlation ids
- lineage refs
- selected layout tags needed for downstream routing

`FileSaved` events must be replay-safe.

- downstream consumers may be offline or lagging
- they resume from durable offsets and replay safely
- downstream publication/indexing must be idempotent
- reconciliation scans may compare storage truth vs downstream projections to heal gaps

## DeerFlow Backend Driver

DeerFlow is modeled as one backend-driver facade over multiple specialized emitters.

Recommended emitter families:

- transcript/message emitter
- artifact emitter
- metric/structured event emitter
- control-intent emitter

Driver responsibilities:

- map DeerFlow-native outputs into storage dimensions and metadata
- choose `append_data` vs `append_control`
- choose inline vs external-ref payload mode
- choose QoS template/policy
- supply correlation and lineage metadata

Driver non-responsibilities:

- canonical path construction
- commit mechanics
- Redpanda publishing internals
- downstream indexing/catalog/view/search logic

## Deferred Optimization

CAS/hash-collapse dedupe is deferred.

The current design guarantees idempotent handling of the same logical write intent.

It does not guarantee elimination of identical physical content across distinct logical writes.

## Overload / Admission Policy

The storage service accepts writes while durable-log capacity, memory, and internal safety budgets remain within configured thresholds.

Once admission safety thresholds are exceeded:

- new writes are rejected explicitly
- accepted writes are never silently dropped

Overload handling is internal control-plane policy, not caller-tunable QoS.

## Stress-Test Validation Snapshot

This design aligns with `docs/architecture/stess-tests.md` as follows.

1. Deep scientific research lifecycle
   - aligned on append-only durable intake, Redpanda buffering, async save, and downstream fanout separation
   - explicit gap: CAS/hash-collapse dedupe is deferred
2. YouTube translate + dub + lip-sync
   - aligned on large artifact writes via external-reference path and async save semantics
3. Right to be forgotten
   - aligned through `append_control` and immutable exclusion-style overlays
4. Massive video proxying
   - aligned by boundary: storage service does not proxy large-file reads
5. Real-time audio/text streaming chat
   - aligned through split ingress model: LiveKit media bypass plus ordinary storage-service path for text/data
6. Object-storage slow down
   - aligned through durable-log decoupling and explicit admission thresholds
7. State-server replica crash during active WebRTC session
   - aligned by LiveKit parallel path and replay-safe downstream catch-up
8. Concurrent conflicting intents
   - aligned through immutable append-only control writes and deterministic durable log ordering

## Non-Goals

- making storage own query/view/index systems
- making storage proxy large-file reads
- exposing internal queue/worker states as public API
- providing CAS/hash-collapse dedupe in the initial version
