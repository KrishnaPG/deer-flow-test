# Shared Storage Service Design WIP

Status: work in progress
Scope: design scratchpad for the shared storage service and DeerFlow backend-driver stress test

## Working Rules

- This file is a shadow WIP document, not an approved spec.
- Decisions here are tentative until explicitly approved by the user.
- Once the design is complete and approved, promote the result into `docs/superpowers/specs/`.
- After spec approval, write an implementation plan in `docs/superpowers/plans/`.

## Focus Order

1. scope and non-goals
2. storage ingress contract
3. WAL / dual-dispatch architecture
4. atomic commit model and object layout
5. signaling contract for derived-level workers
6. downstream handoff contract for exposure/indexing services
7. backend-driver contract
8. DeerFlow stress test
9. validation against `docs/architecture/stess-tests.md`

## Current Sources Reviewed

- `docs/architecture/state-server.md`
- `docs/architecture/stess-tests.md`
- `docs/architecture/troubleshoot.md`
- `docs/architecture/dual-data-heirarchies.md`
- `docs/superpowers/specs/modular-toolkit/42-new-generator-onboarding-contract.md`
- `docs/superpowers/specs/modular-toolkit/02-workspace-and-layering.md`

## Decision Log

### D1. Shared Storage Service Scope

Status: approved with carry-forward constraints

Owns:
- primary in-proc library API for generators and workers
- optional transport wrappers such as HTTP, WebSocket, and gRPC over the same core library contract
- hashing, immutable object identity, and sequence metadata capture
- WAL / queue handoff and event emission
- atomic persistence into storage organized by level `L0`..`L6`
- write-completed signaling for downstream exposure/index/catalog services

Does not own:
- A/B/C hierarchy semantics
- shell-specific domain interpretation beyond installing supplied SQL views
- UI, state-server, or app logic
- world projection or rendering concerns
- business interpretation of derived levels
- direct large-object proxying to end users
- Iceberg publication, LakeFS versioning/catalog work, ClickHouse exposure, Milvus indexing, SQL view creation, content search, or vector search
- the direct LiveKit media-ingress path that writes raw media to storage landing zones outside the normal storage-service ingress path

Open question:
- should the storage service only emit derivation signals, or also execute derivation pipelines itself?

Current recommendation:
- the storage service emits signals only; separate workers execute derivations.

Rationale:
- keeps the storage boundary narrow and high-performance
- preserves clear ownership between persistence and compute
- matches the stress-test model where workers react to storage events asynchronously

Hard invariants attached to this scope:
- storage is immutable once written
- stored objects/rows/files are never updated in place
- stored objects/rows/files are never deleted as part of ordinary operation
- repair means append new corrective facts or overlays, never mutate prior truth
- exclusions, invalidations, supersessions, and reprocess requests must be modeled as new immutable writes
- the primary performance target is ultra-low-latency, tightly coupled, in-proc usage
- the core library should prefer zero-copy or minimal-copy paths wherever possible

Explicit exception path:
- real-time media ingress may bypass the storage service entirely through LiveKit egress or similar direct media transports
- in that path, LiveKit writes raw media directly to object-storage landing zones
- later hash/promote steps can bring that landed media into canonical L0 truth and downstream storage-native processing
- this exception exists to preserve ultra-low-latency WebRTC/media behavior and should be treated as a parallel ingress lane, not as a contradiction of the main storage-service contract

### D2. Replace NATS JetStream With Redpanda Tiered Storage

Status: approved

Decision statement:
- the architecture should not use NATS JetStream as the write-ahead log / durable ingress queue
- use Redpanda with Tiered Storage as the durable log instead

Immediate architectural surfaces affected:
- WAL / durable-ingress design
- dual-dispatch pattern definition
- sequence assignment and ordering guarantees
- producer contract for storage writers
- consumer contract for derivation workers
- replay / rehydration model for state-server-like consumers
- retention, compaction, and cold-history policy
- dead-letter / retry / backpressure handling
- progress and notification event topology
- operational troubleshooting and lag inspection model

Likely non-affected or minimally affected surfaces:
- A/B/C domain semantics remain outside storage-service ownership
- level semantics `L0`..`L6` remain the same
- startup SQL view registration still exists
- backend drivers still annotate writes with hierarchy+level metadata, even if storage operationally cares only about levels

Follow-on decision:
- use Redpanda as the single durable event backbone
- split traffic into distinct topic classes with different delivery, retention, and consumer obligations

Initial topic classes to design next:
- write-intent / commit topics
- progress / lifecycle topics
- derivation-trigger topics
- control-intent topics

Constraint carried forward from approval:
- each topic class must be able to meet its own obligations, especially fast delivery vs stronger durability / replay guarantees

Approved topic-class direction:
- use Redpanda as the single durable event backbone
- start with one topic per class
- partition within each topic by stable workload key such as `mission_id` or equivalent routing key
- allow later split into multiple topics within a class only if scale, hot partitions, or retention pressure require it

Topic-class semantics:

1. `write-intent`
   - purpose: durable ingest requests and commit intents accepted by the storage service
   - obligations: strongest durability, replayability, deterministic ordering per routing key
   - typical consumers: storage commit workers, audit/recovery tooling

2. `progress-lifecycle`
   - purpose: internal lifecycle/diagnostic signals such as write-start, retry, failure, and completion notifications
   - obligations: fast delivery, short/medium replay window, unambiguous lifecycle meaning
   - typical consumers: UI/state-server bridge, monitoring, operator tooling

3. `derivation-trigger`
   - purpose: signal downstream workers that new immutable data is available for derived-level processing
   - obligations: durable, replayable, idempotent consumption
   - typical consumers: chunkers, embedders, aggregators, L2->L3/L4/L5/L6 workers

4. `control-intent`
   - purpose: exclusions, invalidations, reprocess requests, overrides, backfills, repair requests
   - obligations: durable, auditable, immutable
   - typical consumers: storage control plane, derivation workers, policy overlays

Hard invariant attached to all topic classes:
- because storage is immutable, no topic may imply in-place update or deletion of already committed storage truth
- any correction, repair, exclusion, supersession, or reprocessing action must be expressed as a new immutable event and new downstream interpretation

### D3. Ingress Is Library-First, With Transport APIs As Optional Adapters

Status: approved

Decision statement:
- the storage service is primarily a tightly coupled in-proc library/package
- HTTP, WebSocket, and gRPC APIs may exist, but they are adapter layers over the core library rather than the primary architecture

Approved ingress transport shape:
- small payloads may be submitted inline
- large payloads are submitted by reference
- these rules must be available first through the in-proc library API and only secondarily through network adapters

Design consequences:
- the core ingest contract should be Rust-native and allocation-aware
- transport adapters must translate into the same append-only core commands
- zero-copy or minimal-copy handling is a first-class design goal for inline payloads
- by-reference writes must preserve immutable provenance and commit semantics just as strongly as inline writes

Open design questions created by this decision:
- what is the exact core library API shape for inline vs by-reference writes?
- how does the library acknowledge acceptance vs durable save?
- how much caller-supplied metadata is mandatory vs derived inside the storage service?

### D4. Storage Uses Supplied Dimensions To Build Physical Layout

Status: approved

Decision statement:
- the storage service does not own the business meaning of hierarchy tags such as `A`, `B`, or `C`
- however, it does operationally use supplied dimensions to build physical layout in storage

Dimensions the storage service may use for physical organization:
- hierarchy tag, such as `A`, `B`, `C`, or future tags
- level `L0`..`L6`
- plane, such as `as-is`, `chunks`, `embeddings`, or future planes
- payload kind within a hierarchy, such as `metric`, `history`, `chat-note`, `artifact`, `thumbnail`
- representation or format hints, such as `png`, `jsonl`, `parquet`, `wav`
- user-supplied partition tags, such as `64x64`, locale, model family, mission shard, tenant, date bucket, or similar routing dimensions

Example carried forward:
- `/<base>/B/L4/thumbnail/64x64/<filehash>.png`

Clarification:
- storage is not deriving hierarchy semantics on its own
- it is using caller-supplied metadata as deterministic layout dimensions for filenames, folders, partitions, and table/view exposure
- this means the ingest contract must make those dimensions explicit and stable

Constraints introduced by this decision:
- path construction must be deterministic from immutable write metadata
- path construction rules must be versioned so layout changes do not silently break readers
- user-supplied tags used in physical layout must be normalized and validated
- storage should distinguish tags used for path construction from opaque tags kept only as metadata

### D5. Path-Building Dimensions Use A Fixed Core Schema Plus Extensible Partition Tags

Status: approved

Options considered:
- fully free-form tags
- fixed required core schema plus extensible extra partition tags
- fully driver-defined layout schema per backend

Recommended option:
- fixed required core schema plus extensible extra partition tags

Why this is the current recommendation:
- gives the storage service a stable, deterministic physical layout contract
- preserves cross-backend operational consistency for tooling and troubleshooting
- still allows backend-specific partitioning such as `64x64`, locale, model family, tenant, shard, or date bucket

Proposed fixed core layout dimensions:
- hierarchy
- level
- plane
- payload kind
- representation / format

Proposed extensible layout dimensions:
- ordered partition tags used for path construction

Proposed non-layout metadata bucket:
- opaque annotations retained as immutable metadata but not used in path construction

### D6. Append Contract Uses Separate Data And Control APIs

Status: approved

Decision statement:
- the core storage library exposes two append command families:
  - `append_data`
  - `append_control`
- ordinary immutable data writes and immutable control overlays are intentionally separated at the API boundary

Approved `append_data` envelope:
- `layout`: hierarchy, level, plane, payload kind, format, partition tags
- `identity`: idempotency key, content hash if known, writer identity
- `lineage`: parent refs, derived-from refs, other immutable ancestry metadata
- `correlation`: `mission_id`, `run_id`, `agent_id`, `artifact_id`, `trace_id`
- `payload`: inline bytes or external reference
- `metadata`: opaque immutable annotations not used in physical path construction

Approved `append_control` envelope:
- same identity and correlation envelope as `append_data`
- control intent kind, such as exclusion, supersession, reprocess, invalidation, backfill, or override request
- target refs pointing at prior immutable truth
- optional rationale, policy, or operator metadata

Approved acknowledgement lifecycle:
- `FileAccepted`: request validated and durably handed to Redpanda
- `FileSaved`: immutable object/file landed in storage

Approved caller behavior:
- synchronous calls return `FileAccepted` by default
- `FileSaved` is delivered asynchronously via lifecycle topics

Rationale:
- keeps the API append-only and semantically honest
- prevents ordinary data writes from being confused with control overlays
- supports ultra-low-latency in-proc calls without conflating acceptance with full downstream readiness

### D7. WAL And Lifecycle Must Support Multiple Write Classes

Status: approved

Trigger for this decision:
- the architecture must support different latency and durability expectations without forcing every caller onto the same slowest path

Write scenarios explicitly in scope:
- fire-and-forget writes
- slow research artifact / durable writes
- low-latency real-time ingestion writes, such as live meeting transcription recorded as artifact

Constraint introduced by these scenarios:
- the system must not trade off latency for durability, or durability for latency, when a caller does not require the tradeoff
- this means the WAL, lifecycle events, and commit path likely need per-write-class behavior over a shared core contract

Open design question:
- should the storage library expose explicit write classes / qos profiles so each append declares the expected acceptance, commit, and progress behavior?

Approved direction:
- the storage library exposes composable QoS policy objects on append operations
- it also provides a small set of predefined profile enums/templates for convenience

Rationale:
- keeps common cases simple
- preserves flexibility for workloads that need a non-default mix of latency, durability, batching, progress visibility, and acknowledgement behavior

Implication for API design:
- callers may choose a built-in profile template and optionally override policy fields
- the storage service core must treat QoS as declarative policy, not as ad hoc branches scattered across the write path

### D8. QoS Is Fully Async, Idempotency Is Mandatory, Backpressure Is Internal

Status: approved

Decision statement:
- no caller ever blocks on the storage service for commit or ready state
- all caller-facing acknowledgements are asynchronous beyond initial acceptance
- if a caller needs confirmation, it must consume `FileSaved` events or poll the durable event stream / queue

Approved constraints:
- `commit_wait` is removed from caller-facing QoS
- `ready_wait` is removed from caller-facing QoS
- idempotency is always required; it is not optional
- caller-facing backpressure policy is removed from QoS

Rationale:
- the storage service is a low-level immutable append system and should not expose blocking semantics as part of ordinary write behavior
- immutable storage makes duplicate prevention and replay safety fundamental, so idempotency must be mandatory
- backpressure is still a real internal concern, but it should be handled by storage admission control, queueing, prioritization, and overload protection rather than as an application-tunable per-write option

Caller confirmation model:
- synchronous result: accepted or rejected at admission time
- asynchronous lifecycle: `FileSaved`, failure, retry, and other internal state transitions via lifecycle topics

Internal overload model:
- backpressure exists inside the storage service and worker system because resources are finite
- under severe overload the service may reject new admissions or downgrade non-critical progress emissions, but that is an internal control-plane policy, not a caller QoS choice

### D9. Public API Lifecycle Is Minimal; Internal Pipeline States Stay Internal

Status: approved

Decision statement:
- internal queueing, worker pickup, retries, and similar mechanics are not first-class parts of the public storage API
- callers submit immutable writes and later observe storage facts, not internal pipeline choreography

Public-facing lifecycle model:
- synchronous admission result at append time: accepted or rejected
- asynchronous storage facts for observers who care:
  - `FileSaved`
  - terminal failure signal only if the write cannot be completed without new caller action

Explicitly not part of the public API contract:
- `Queued`
- `WriteStarted`
- `WriteProgress`
- internal retry counts
- worker handoff details

Internal observability model:
- richer internal trace/debug events are still required for storage operators and debugging writers/queues
- these events exist for diagnostics, metrics, replay, and troubleshooting, not as business-facing API concepts
- they should be routed and retained separately from immutable user data truth

Retention / cleanup rule:
- Redpanda WAL segments, fanout delivery logs, and internal diagnostic lifecycle events are infrastructure artifacts, not canonical user data
- they may be compacted, tiered, retained briefly, or cleaned up according to operational policy
- immutable storage truth in object storage is not affected by such cleanup

Design consequence:
- separate the public storage-fact contract from the internal transport/diagnostic contract
- avoid naming internal pipeline states in the main append API unless the user explicitly opts into an operator/debug stream

### D10. Atomic Commit Uses Separate Inline And External-Reference Paths

Status: approved

Decision statement:
- atomic commit has two execution paths:
  - `InlineCommitPath`
  - `ExternalRefCommitPath`

Approved meanings:

1. `InlineCommitPath`
   - used for small inline payloads
   - immutable bytes come from the in-proc caller buffer or adapter payload
   - storage hashes, durably logs, and writes them into canonical immutable storage
   - low-copy and micro-batch optimizations are allowed as long as canonical commit semantics remain intact

2. `ExternalRefCommitPath`
   - used for large payloads provided by reference
   - storage validates and claims the reference, then promotes it into managed immutable storage
   - the default behavior is canonical copy/promote into storage managed by the storage service
   - future policy may allow a trusted immutable claim/freeze path without copy, but that is not the default

Hard rule:
- `FileSaved` means the object/file is durably present at its canonical immutable storage path
- it does not mean merely queued, claimed, staged, or visible in a temporary location

Rationale:
- keeps canonical truth inside managed immutable storage
- preserves a single meaning of `FileSaved` across small and large payloads
- still leaves room for later optimization where trusted immutable external stores can be claimed without full copy

### D11. Canonical Paths Are Relative; Multi-File Atomicity Uses Commit Manifests

Status: approved

Decision statement:
- all internally stored path references are relative paths
- the storage base/root is deployment-local configuration and must never be baked into stored path references
- layout/schema version is not encoded into the path name

Approved canonical path rule:
- canonical committed object paths are deterministic relative paths derived from:
  - core layout dimensions
  - ordered partition tags
  - canonical content identity

Representative relative path shape:
- `/<hierarchy>/<level>/<plane>/<payload_kind>/<partition...>/<content_hash>.<ext>`

Examples:
- `/B/L4/as-is/thumbnail/64x64/<hash>.png`
- `/A/L0/as-is/chat-note/<mission_id>/<hash>.jsonl`
- `/B/L3/chunks/transcript/<date_bucket>/<hash>.parquet`

Atomicity rule:
- true atomic commit is straightforward for a single canonical file/object
- for a logical write that materializes as multiple files, atomicity is not claimed at the raw object-put level
- instead, atomic visibility is provided through a final immutable commit manifest / commit marker written only after all member files are durably present

Implications of the manifest model:
- member files may exist before the logical batch is visible
- downstream consumers and visibility logic must treat the commit manifest / marker as the source of truth for visibility
- `FileSaved` for a multi-file logical write means the manifest/marker was durably written and references a complete immutable member set
- partially landed member files without a manifest are not visible as committed logical truth

Rationale:
- avoids claiming impossible cross-object atomic semantics from object storage
- preserves clean migration because relative paths survive base-directory relocation
- keeps visibility semantics explicit and auditable

### D12. Derivation Signaling Uses Generic Post-Commit Triggers

Status: approved

Decision statement:
- derivation workers consume post-commit trigger events rather than relying primarily on blind storage scans
- trigger events are emitted only after a logical write is committed
- for multi-file logical writes, triggers reference the final commit manifest / marker, not partial member files

Approved routing direction:
- start with a generic derivation-trigger schema and route/filter by metadata and partition key
- introduce specialized trigger topics later only if scale or routing efficiency requires it

Trigger payload should carry at least:
- logical write identity
- relative committed object path or manifest path
- hierarchy, level, plane, payload kind, format
- correlation ids
- lineage refs
- selected layout tags needed for routing decisions

Hard invariant:
- no derivation trigger is emitted for invisible partial files
- derivation is triggered from committed logical truth only

### D13. Storage Boundary Ends At Commit; Exposure And Indexing Are Downstream Services

Status: approved

Decision statement:
- the shared storage service ends at immutable storage commit and post-commit signaling
- Iceberg, LakeFS, ClickHouse, Milvus, SQL view creation, content search, and vector search belong to downstream services outside the storage-service boundary

Why:
- object storage remains the absolute immutable source of truth
- Hierarchy A is structured enough to benefit from Iceberg-backed ClickHouse exposure
- Hierarchy B is primarily unstructured/as-is artifact storage, with downstream chunking and embedding paths feeding Milvus
- versioning/file-tracking concerns such as LakeFS are separate from the core write path

Implications:
- storage emits `FileSaved` for each single-file or logical batch commit
- downstream services subscribe to committed-write events and build their own exposure/index/catalog layers
- Hierarchy A downstream can publish into Iceberg and then expose through ClickHouse
- Hierarchy B downstream can maintain LakeFS/catalog/versioning and derive chunks/embeddings for search systems
- the same committed-write event stream is the handoff boundary from storage to all downstream consumers

Design consequence:
- remove table/view/index ownership from the storage service design
- replace the pending exposure work item with a downstream handoff/event-contract design item

### D14. Downstream Exposure Services Consume `FileSaved` Events

Status: approved

Decision statement:
- downstream services consume committed-write events emitted by the storage service
- those downstream services may batch, debounce, index, publish, or catalog committed writes according to their own responsibilities
- the storage service does not own those policies

Examples of downstream responsibilities:
- Iceberg/ClickHouse publisher for structured Hierarchy A data
- LakeFS or file-catalog service for Hierarchy B versioning/tracking
- chunking workers over `as-is` artifacts
- embedding workers over chunk outputs
- Milvus sync/index workers over embeddings
- shell/view services that create SQL views or search projections

Storage-service responsibility stays limited to:
- durable immutable commit
- committed-write signaling
- enough metadata in the event for downstream services to route and act

### D15. `FileSaved` Events Must Be Replay-Safe For Downstream Consumers

Status: approved

Problem this addresses:
- downstream services may be offline or lagging while committed writes continue
- committed-write handoff must be replayable and idempotent

Current recommended direction:
- the storage service publishes `FileSaved` events only
- downstream services own any additional readiness/publication semantics they need
- downstream consumers must be able to resume from durable offsets and replay safely

`FileSaved` should carry at least:
- logical write id
- relative committed path or commit-manifest path
- hierarchy, level, plane, payload kind, format
- correlation ids
- lineage refs
- selected layout tags needed for downstream routing

Replay / recovery rule:
- post-commit events remain durably retained in Redpanda until consumed according to retention policy
- if a downstream consumer is offline, it resumes from its last committed offset and processes the backlog
- downstream publication/indexing steps must be idempotent so replay after a crash does not duplicate visibility or indexing
- later reconciliation scans may compare storage truth vs downstream projections to heal gaps

Terminology rule:
- storage first returns `FileAccepted` when responsibility for eventual durable commit is taken
- storage later emits `FileSaved` when the canonical file or logical file batch is durably committed
- downstream services must treat `FileSaved` as the handoff boundary from storage truth to indexing, catalog, versioning, chunking, embedding, and query exposure

### D16. Canonical Path Construction Is 100% Storage-Owned

Status: approved

Decision statement:
- backend drivers such as DeerFlow do not construct canonical storage paths and do not supply path suffix hints
- drivers only supply immutable layout dimensions and metadata
- the storage service alone constructs canonical relative paths

Rationale:
- centralizes layout policy
- prevents backend-specific path drift
- keeps the backend-driver contract generic and stress-testable

### D17. DeerFlow Uses One Driver Facade With Multiple Specialized Emitters

Status: approved

Decision statement:
- DeerFlow is modeled as one backend-driver facade over multiple specialized emitters

Recommended specialized emitter families:
- transcript / message emitter
- artifact emitter
- metric / structured event emitter
- control-intent emitter
- additional emitter families as DeerFlow-native shapes require

Driver responsibilities:
- map DeerFlow-native outputs into storage dimensions and metadata
- choose `append_data` vs `append_control`
- choose inline vs external-ref payload mode
- choose QoS template/policy
- supply correlation and lineage metadata

Non-responsibilities:
- canonical path construction
- commit mechanics
- Redpanda publishing internals
- downstream indexing/catalog/view/search logic

Rationale:
- keeps backend mapping modular without fragmenting the public backend-driver surface
- allows different DeerFlow output shapes to evolve independently while still stress-testing one generic storage contract

### D18. Hash-Collapse / CAS Dedupe Is Deferred

Status: approved

Decision statement:
- content-addressed hash-collapse / cross-intent physical dedupe is out of scope for the current storage-service design
- mandatory idempotency remains required, but it is not treated as a substitute for full CAS-based physical dedupe

Rationale:
- effective physical dedupe across distinct logical writes requires stronger CAS/content-addressed design choices
- that optimization adds complexity and is not required for the initial storage-service boundary
- storage cost is acceptable enough that correctness and architectural clarity take priority for now

Implication:
- the current design guarantees idempotent handling of the same logical write intent
- it does not guarantee elimination of identical physical content across distinct logical writes

### D19. Admission Under Overload Uses Explicit Safety Thresholds

Status: approved

Problem this addresses:
- Scenario 6 requires the system to survive object-storage slow-down and downstream lag without silently dropping data
- the design still needs an explicit admission policy for when internal durable queues or commit workers approach unsafe saturation

Recommended direction:
- accept writes while durable-log capacity, memory, and internal safety budgets remain within configured thresholds
- once admission safety thresholds are exceeded, reject new writes explicitly rather than silently dropping them
- never silently drop accepted writes
- internal diagnostics should reflect degraded mode, but the public write contract stays simple: `FileAccepted` or rejected, followed later by `FileSaved` if accepted work completes

Operational consequences:
- overload handling is a storage-service control-plane concern, not caller-tunable QoS
- different write classes may consume different internal budgets or priority lanes, but admission still resolves to accept or reject
- downstream lag may lengthen time-to-save, but accepted writes remain durable responsibilities of the system

### D20. Stress-Test Validation Snapshot

Status: approved checkpoint

Scenario alignment against `docs/architecture/stess-tests.md`:

1. Deep scientific research lifecycle (10,000+ parallel agents)
- aligned on append-only durable intake, Redpanda buffering, async save, and downstream fanout separation
- explicit gap: CAS/hash-collapse dedupe is deferred for now

2. YouTube translate + dub + lip-sync (HOTL)
- aligned on large artifact writes via external-reference path and async save semantics
- downstream progress/index/catalog behavior stays outside storage-service ownership

3. Right to be forgotten (compliance & immutability)
- aligned through `append_control` and immutable exclusion-style overlays
- downstream search/filter enforcement remains outside the storage service

4. Massive video proxying (lakeFS / streaming)
- aligned by boundary: storage service does not proxy large-file reads; mediated read systems handle that elsewhere

5. Real-time audio/text streaming chat (HITL)
- aligned through split ingress model:
  - media may bypass the storage service via LiveKit egress direct-to-storage landing zones
  - text/data may still use the storage-service append path

6. S3 / object storage rate limiting (503 slow down)
- aligned through durable-log decoupling and explicit admission thresholds
- accepted writes are never silently dropped

7. State-server replica crash during active WebRTC session
- aligned by keeping real-time media on the LiveKit path and replay-safe event streams for downstream catch-up

8. Concurrent conflicting intents
- aligned through immutable append-only control writes and deterministic durable log ordering

Current design confidence:
- shared storage-service boundary is coherent and internally consistent
- main deferred optimization is CAS/hash-collapse dedupe
- next promotion candidate is turning this approved WIP into a formal spec document

## Open Questions Queue

1. Should the storage service only emit derivation signals, or also execute derivation pipelines itself?
2. How does replacing JetStream with Redpanda change the WAL, notification, and replay architecture?
3. What is the ingress protocol for writers: HTTP, gRPC, both, or queue-first?
4. What metadata is mandatory on every write request?
5. What is the atomic commit model for small vs large payloads under immutable storage?
6. What exact metadata must `FileSaved` carry for downstream exposure/indexing services?
7. How are backend-provided SQL views registered, versioned, and validated at startup?
8. What is the exact backend-driver surface for DeerFlow?
9. Which stress-test scenarios are first-class acceptance gates for the initial version?
