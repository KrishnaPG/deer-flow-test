# State Server Stakeholders And Sources Of Truth

**Core Paradigm:** Real-Time, Low-Latency First with Guaranteed Reliability and Absolute Immutability.

## 1. Architectural Mandate: Storage-Native, Mediated-Read

To prevent the State Server from becoming a write bottleneck, to ensure ultra-low latency, and to guarantee absolute data provenance, the system strictly enforces the following "Storage-Native" data flow:

1. **The Absolute Single Source of Truth:** Object Storage (e.g., S3, lakeFS, Iceberg) is the *only* immutable source of truth. **No component ever writes raw data or artifacts directly to a database.**
2. **Absolute Immutability (Append-Only):** Storage is strictly append-only. `UPDATE` and `DELETE` operations are architecturally forbidden. History is never rewritten.
3. **Tri-Plane Storage Hierarchy (1:N:M):** All data exists physically on Storage across levels L0 to L4 in a relational hierarchy. To avoid data duplication and support multiple AI models, data is split into three planes linked by immutable content hashes:
    *   **As-Is Plane (1):** The actual canonical data (raw video, chat JSON, OCR text, synthesized Markdown). Identified by `as_is_hash = SHA256(payload)`.
    *   **Chunks Plane (N):** The segmented data derived from the As-Is plane. Stores the chunk payload and is identified by `chunk_hash = SHA256(as_is_hash + chunking_strategy + chunk_index)`.
    *   **Embeddings Plane (M):** The mathematical vector representations of the Chunks. **Strict Rule:** This plane *only* stores the vector array and the pointer (`chunk_hash`) back to the Chunks plane. It never duplicates the payload text.
15. **Internal Write Path (The Storage Service & Dual-Dispatch):** AI agents and workers are "thin clients". They do not manage S3 connections or retries. Instead, they send write requests to a centralized **Storage Service**. The Storage Service implements a **Dual-Dispatch pattern**: it accepts the file into a reliable persistent queue (powered by NATS JetStream acting as the local fast queue) and *in parallel* emits an event to the Event Bus.
16. **The Event Bus (WAL / Claim Check):** Parallel to the persistent queue, lightweight notification events are published to NATS JetStream. Small payloads (chat tokens) are embedded directly in the event; large payloads emit stateful progress URIs (`WriteStarted`, `WriteProgress`, `ArtifactCreated`).
17. **Databases as Views (Read/Compute Plane):** Databases (ClickHouse/Milvus) are purely projections, views, and search indexes built on top of the Storage truth. ClickHouse mounts the Storage data lake directly, while Milvus is populated asynchronously via internal Sync Workers.
18. **External Read Path (Mediated):** Any external entity (UI, Control Center, Observers) **must** route reads through the State Server acting as the strict **ABAC** gatekeeper.
19. **External Write Path (Intents):** External human commands/intents go through the State Server for immediate ABAC validation, then handed to the Storage Service to document the intent into Storage and publish to the Event Bus.

---

## 2. Stakeholders & Access Matrix

| Stakeholder / Component | Role | Access Type | Data Handled / Transferred |
| :--- | :--- | :--- | :--- |
| **Data Generators (Agents / Workers)** | Thin Clients | API POST / Stream | Generates data, delegates all storage/retry complexity to the Storage Service. |
| **Storage Service** | Reliable Ingress / Dual-Dispatch | Write (S3) / Pub (NATS) | Handles cryptographic hashing, persistent queueing, S3 commits, stream micro-batching, and parallel Event Bus emits. |
| **LiveKit Egress** | Raw Media Transport | Direct S3 Write | Offloads L0 Raw Media (WebRTC audio/video) ingestion from agents. Writes to temp S3 landing zones. |
| **State Server** | Hot Cache & ABAC Gatekeeper | Subscribes / Read Proxy | Listens to Event Bus for the latest activity ("Hot Window"). Mediates all external reads, querying DBs for historical data ("Warm Cache"). Enforces ABAC. |
| **Control Center (Operator)** | Active Human Operator | External Write (Intents) / Read | Submits `intents` via State Server. Reads real-time updates from State Server. |
| **Observer UI** | Read-Only Viewer | Read via State Server | Reads best-effort, ABAC-filtered data strictly through the State Server. |
| **Event Bus (NATS JetStream)** | Write-Ahead Log / Claim Check | Pub/Sub | High-volume fanout for newly written Storage pointers (URIs), live stream tokens, and pipeline progress updates. |
| **ClickHouse** | Relational/OLAP View | Storage Mount (Read) | Mounts and queries the "As-Is" and "Chunks" Object Storage. Acts as the join engine. |
| **Milvus & Sync Workers** | Vector Search View | Async Ingest (Read) | Sync Workers listen to NATS, read "Embeddings" from Storage, and index them into Milvus. |
| **lakeFS / S3** | The Single Source of Truth | Storage (Truth) | Stores all raw files, intents, exclusions, chunks, and vector parquet files physically organized by the Tri-Plane hierarchy. |

---

## 3. Data Architecture & Sources of Truth

| Data Category | Tier | Absolute Source of Truth (Storage) | Database View (Search/Compute) | State Server View (Cache) |
| :--- | :--- | :--- | :--- | :--- |
| **Live Activity (Intents)** | Transients | `s3://.../as_is/L0/intents` | ClickHouse (Mounted) | **Hot Cache** (Memory via NATS) |
| **Historical State** | Canonical | `s3://.../as_is/L0/events` | ClickHouse (Mounted) | **Warm Cache** (Memory/Redis) |
| **Artifacts (Raw)** | **L0: Crude Data** | `s3://.../as_is/L0` (lakeFS) | - | Authorized Pointers (Pre-Signed URLs) |
| **Artifacts (Chunks)** | **L1-L4: Segments** | `s3://.../chunks/L1-L4` | ClickHouse (Mounted Text/Payload) | Warm/Hot Cache Proxy |
| **Artifacts (Vectors)** | **L1-L4: Embeddings**| `s3://.../embeddings/L1-L4` | Milvus (Vectors + `chunk_hash`) | Warm/Hot Cache Proxy |
| **Compliance Ledger** | **Exclusions** | `s3://.../exclusions/` | ClickHouse / Milvus (Anti-Join Filter) | Enforced via DB Views |

---

## 4. Ingestion Patterns & Interaction Modes (HITL vs. HOTL)

The Storage Service dictates how data moves depending on the required latency and interaction type.

### The Storage Service & Dual-Dispatch (Stateless Ingress)
The Storage Service acts as a highly optimized, stateless router (e.g., written in Rust). It delegates the "local fast queue" responsibility to NATS JetStream, allowing it to instantly acknowledge ingest requests without managing an embedded disk WAL.
*   **Small Payloads (<1MB):** Written directly to a NATS JetStream ingress queue. Full payload emitted to the NATS Event Bus in parallel. Background workers pull from the ingress queue, micro-batch the data into highly-optimized Parquet/JSONL files, and push to S3.
*   **Large Payloads (>1MB):** Written to temp S3. Queue receives task pointer. NATS emits progress events (`WriteStarted`, `WriteProgress: 45%`). Background workers commit via S3 idempotent rename (`CopyObject`/`DeleteObject`) and emit final `ArtifactCreated` (100% Ready) URI.

### HITL: Human-In-The-Loop (Interactive / Synchronous)
*Examples: Live voice chat with an AI, real-time meeting translation.*
*   **Requirement:** Mandatory ultra-low latency (< 50ms). User is the primary driver.
*   **Media Transport:** User and AI connect directly via **LiveKit WebRTC**. Audio bypasses the State Server entirely. LiveKit Egress acts as a reliable component, recording raw L0 audio to an S3 "landing zone" to be hashed and promoted later.
*   **Data Transport (The "Stream-Tee"):** The AI Agent streams text tokens to the Storage Service `STREAM /ingest` endpoint. The Storage Service acts as a Tee:
    *   *Fast Path:* Instantly wraps tokens in NATS events (`StreamDelta`). The UI sees text appear character-by-character with zero latency.
    *   *Slow Path:* Micro-batches tokens in an in-memory buffer until a logical delimiter is hit (e.g., end of sentence), hashes the chunk, writes it to the persistent queue, and commits to S3.

### HOTL: Human-On-The-Loop (Observational / Asynchronous)
*Examples: Batch video dubbing, massive document ingestion, agent swarms running research.*
*   **Requirement:** Progress visibility. Slight delays (1-5 seconds) are completely acceptable. User is watching, not driving.
*   **Transport (Fire-And-Forget):** The AI Worker finishes a 50MB synthesized audio track and fires a `POST /write` to the Storage Service, then immediately moves to the next task.
*   **UI Updates:** The Storage Service's persistent queue emits NATS progress updates (`WriteStarted`). The State Server updates the UI progress bars in real-time while the heavy file transfer happens in the background. The UI only enables the "Play" button when the `ArtifactCreated` final event arrives.

---

## 5. State Server Caching Strategy

The State Server is explicitly **not** the Source of Truth. It is highly optimized for fast external delivery:

*   **Hot Cache (The Window):** Maintains a rolling window of the absolute latest activity (e.g., the last 5 minutes of chat, the currently active agent progress bars). It is populated purely by asynchronously listening to the Event Bus pointers. UI clients receive sub-millisecond responses from this cache.
*   **Warm Cache (On-Demand):** When an external client (UI) requests data outside the Hot Window, the State Server intercepts the request, validates ABAC, queries the Database Views (ClickHouse/Milvus), and temporarily stores the result in a Warm Cache before returning it to the client.

---

## 6. Stress-Test Scenarios

### Scenario 1: Deep Scientific Research Lifecycle (10,000+ Parallel Agents)
*A massive burst of worker processes generating hypotheses, code, and datasets.*

*   **Action:** 10,000 agents simultaneously chunk L3 data.
*   **Data Path:** Agents fire-and-forget chunks to the Storage Service. The Storage Service uses aggressive in-memory Bloom filters (Hash Collapsing) to deduplicate exact matches (e.g., 1,000 identical "Error: timeout" logs) into a **single S3 write**, while parallel-emitting 1,000 NATS events. Background workers micro-batch the rest into Parquet files.
*   **Stress Point Survived:** Storage costs and DB query times are annihilated by deduplication and micro-batching. State Server ignores the write burst.

### Scenario 2: YouTube Translate + Dub + Lip-Sync (HOTL)
*ASR -> Translate -> TTS -> Align -> Render pipeline.*

*   **Action:** A complex, multi-stage pipeline transcribes an original L0 video, translates it, synthesizes new audio, and renders a lip-synced composite video tailored for a target culture.
*   **Data Path (Storage):** Workers fire-and-forget large L1-L4 artifacts (video/audio blobs) to the Storage Service.
*   **State Server Role:** Listens to NATS `WriteProgress` events to update the UI's progress bars in real-time. The operator can visually observe the pipeline's exact state without blocking worker execution (HOTL pattern). Accepts operator overrides (intents) to edit the glossary.
*   **Stress Point Survived:** The UI remains perfectly responsive and informative while massive background I/O occurs asynchronously. Glossary edits spawn new branches via immutability, preserving older versions.

### Scenario 3: Right to be Forgotten (Compliance & Immutability)
*A user requests the deletion of a massive, heavily processed L0 document.*

*   **Action:** User clicks "Delete Document" in the UI.
*   **Data Path:** The UI sends an intent to the State Server. The State Server validates ABAC and hands the intent to the Storage Service.
*   **The Immutable Ledger:** The Storage Service writes a new, immutable record to the `s3://.../exclusions/` prefix: `{ target_hash: "as_is_hash_xyz", reason: "user_request" }`.
*   **Database View (Overlay):** ClickHouse and Milvus dynamically construct their "Search Views" using an anti-join or filter against the Exclusions ledger.
*   **Stress Point Survived:** Immediate UI disappearance satisfying compliance without violating append-only S3 guarantees.

### Scenario 4: Massive Video Proxying (LakeFS / Streaming)
*A user requests to view an original 4GB L0 raw MP4 file.*

*   **Action:** User clicks "View Original Source" in the UI.
*   **Data Path:** UI requests the file from the State Server.
*   **State Server Role:** Validates ABAC permissions, generates a Pre-Signed URL pointing directly to the specific version commit in lakeFS/S3, and returns the URL.
*   **Stress Point Survived:** Prevents State Server OOM crashes and bandwidth saturation by never proxying large blob traffic.

### Scenario 5: Real-Time Audio/Text Streaming Chat (HITL)
*User speaking to an AI agent with millisecond latency requirements.*

*   **Action:** User initiates a live voice conversation with an AI Agent.
*   **Data Path (Media):** Direct WebRTC connection via LiveKit (sub-50ms latency). LiveKit Egress independently records the raw session to an S3 "landing zone" for later Hash & Promote to L0 truth by the Storage Service.
*   **Data Path (Text/Data):** AI Agent streams text to the Storage Service. The "Stream-Tee" instantly pushes tokens to NATS for real-time UI display, while micro-batching sentences to S3 in the background.
*   **Stress Point Survived:** Absolute low-latency HITL interaction while preserving strict "Storage-Native" truth, completely offloading the transport layer, and keeping AI Agents extremely "thin".
---

## 7. Orchestration, Observability, and Operations

While the Storage Service handles truth and ingestion, the orchestration of agents, policy enforcement, and system observability rely on a specific stack of permissive open-source tools.

### Orchestration & Runtime Plumbing
*   **Temporal (MIT):** Handles durable workflows, retries, timers, and approval gates. Used exclusively for orchestrating the *logic* of the agents and pipelines, not for transporting large payloads or telemetry.
*   **Dapr (Apache-2.0):** Sidecars deployed alongside every polyglot microservice (Rust, Python, Node). Provides mTLS, service invocation, and pub/sub bindings, ensuring agents can emit/consume consistent events over zero-trust lanes without embedding custom network SDKs.

### Memory & Retrieval Stack (Cognitive Tiers)
To support the State Server's caching and the AI Agents' context windows, the system utilizes a layered memory approach:
*   **Hot Lane (Redis):** Working set per mission (scene graph deltas, recent human prompts, short token windows). Holds the leadership lease (fencing tokens) for active State Server replicas.
*   **Warm Lane (Postgres + pgvector):** Mission/session metadata, RBAC definitions, structured tool outputs, and embedding shards for the most recent cycle. Acts as the fast lookup cache before querying the cold vector store.
*   **Cold Lane (Iceberg / Milvus):** The durable, append-only Tri-Plane data residing on S3, queryable via ClickHouse and Milvus.

### Operational Guardrails & Security
*   **Canonical State Model (Event Sourced):** The State Server is strictly event-sourced. Commands append new events to the log. Snapshots are checkpointed per mission ID in Postgres/Redis, allowing reconnecting clients to replay from the last acknowledged cursor.
*   **High Availability:** Active/active State Server replicas deploy behind sticky load balancing. Only a single replica per mission holds the Redis leadership lease. On failover, followers load the latest snapshot and replay the diff log before resuming emission.
*   **Security & ABAC:** Mutual TLS everywhere via Dapr and Envoy. JWT-based auth for humans; SPIFFE/SPIRE IDs for services. ABAC policies are evaluated using **Casbin** (backed by Postgres) before any state append is permitted.
*   **Backpressure & Load Shedding:** Under overload, the system prioritizes operator sessions over observability. It dynamically parks non-critical Temporal workflows and downsamples observer UI diffs to maintain operator stability.

### Observability & Telemetry
*   **The Correlation Contract:** Every event, span, and log MUST carry: `mission_id`, `run_id`, `agent_id`, `artifact_id`, and `trace_id`.
*   **Trace Backbone:** OpenTelemetry (OTEL) spans from UI prompt → State Server intent → Temporal workflow steps → downstream Dapr services. Exported to ClickHouse via SigNoz/Tempo.
*   **LLM Metrics:** **Langfuse (MIT)** captures latency, cost, and token usage per model call.
*   **Artifact Lineage:** Every artifact tier mutation emits to NATS and lands in Iceberg `lineage_events`, allowing UI dashboards to show the complete `prompt -> agent -> artifact -> memory` chain.
