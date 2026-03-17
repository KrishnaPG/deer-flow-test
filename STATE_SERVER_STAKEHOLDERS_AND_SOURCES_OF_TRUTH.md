# State Server Stakeholders And Sources Of Truth

**Core Paradigm:** Real-Time, Low-Latency First with Guaranteed Reliability and Absolute Immutability.

## 1. Architectural Mandate: Storage-Native, Mediated-Read

To prevent the State Server from becoming a write bottleneck, to ensure ultra-low latency, and to guarantee absolute data provenance, the system strictly enforces the following "Storage-Native" data flow:

1. **The Absolute Single Source of Truth:** Object Storage (e.g., S3, lakeFS, Iceberg) is the *only* immutable source of truth. **No component ever writes raw data or artifacts directly to a database.**
2. **Absolute Immutability (Append-Only):** Storage is strictly append-only. `UPDATE` and `DELETE` operations are architecturally forbidden. History is never rewritten.
3. **Tri-Plane Storage Hierarchy (1:N:M):** All data exists physically on Storage across levels L0 to L4 in a relational hierarchy. To avoid data duplication and support multiple AI models, data is split into three planes linked by immutable content hashes:
    *   **As-Is Plane (1):** The actual canonical data (raw video, chat JSON, OCR text, synthesized Markdown). Identified by `as_is_hash = SHA256(payload)`.
    *   **Chunks Plane (N):** The segmented data derived from the As-Is plane. A single As-Is document can be chunked in multiple ways (e.g., semantic vs. sliding window). Stores the chunk payload and is identified by `chunk_hash = SHA256(as_is_hash + chunking_strategy + chunk_index)`.
    *   **Embeddings Plane (M):** The mathematical vector representations of the Chunks. A single chunk can be embedded by multiple models (e.g., OpenAI vs. BGE). **Strict Rule:** This plane *only* stores the vector array and the pointer (`chunk_hash`) back to the Chunks plane. It never duplicates the payload text.
4. **Internal Write Path (The Reliable Component Contract):** AI agents, workers, and extractors (acting as "Reliable Components") **write directly** and reliably to the Object Storage planes. 
5. **The Event Bus (WAL / Claim Check):** Immediately after a successful write to Storage, the Reliable Component publishes a lightweight notification event (containing a pointer/URI to the storage object) to the Event Bus (e.g., NATS JetStream).
6. **Databases as Views (Read/Compute Plane):** Databases (ClickHouse/Milvus) are purely projections, views, and search indexes built on top of the Storage truth. 
    *   *ClickHouse* mounts the Storage data lake directly (e.g., via S3 Table Engines or Iceberg integration).
    *   *Milvus* is populated asynchronously via internal "Index Sync Workers" that listen to the Event Bus and ingest the newly written vector parquet files from Storage.
7. **External Read Path (Mediated):** Any external entity (UI, Control Center, Observers) **must** route reads through the State Server. The State Server acts as the strict **ABAC (Attribute-Based Access Control) gatekeeper**.
8. **External Write Path (Intents):** External human commands/intents go through the State Server for immediate ABAC validation. Once validated, they are handed to a Reliable Component, which documents the intent into Storage and publishes to the Event Bus.

---

## 2. Stakeholders & Access Matrix

| Stakeholder / Component | Role | Access Type | Data Handled / Transferred |
| :--- | :--- | :--- | :--- |
| **Data Generators (Agents / Workers)** | Reliable Components | Direct Write (Storage) / Direct Read (DBs) | Writes L0-L4 "As-Is", "Chunks", and "Embeddings" (Parquet/JSON/Blobs) directly to Object Storage. Queries DBs directly for context. Emits pointers to Event Bus. |
| **State Server** | Hot Cache & ABAC Gatekeeper | Subscribes / Read Proxy | Listens to Event Bus for the latest activity ("Hot Window"). Mediates all external reads, querying DBs for historical data ("Warm Cache"). Enforces ABAC. |
| **Control Center (Operator)** | Active Human Operator | External Write (Intents) / Read | Submits `intents` via State Server. Reads real-time updates from State Server's hot cache or historical data from its warm cache. |
| **Observer UI** | Read-Only Viewer | Read via State Server | Reads best-effort, ABAC-filtered data strictly through the State Server. Never accesses DBs or Storage directly. |
| **Event Bus (NATS JetStream)** | Write-Ahead Log / Claim Check | Pub/Sub | High-volume fanout for newly written Storage pointers (URIs), telemetry, and pipeline metadata notifications. |
| **ClickHouse** | Relational/OLAP View | Storage Mount (Read) | Mounts and queries the "As-Is" and "Chunks" Object Storage. Acts as the join engine linking Vectors to Text via `chunk_hash`. |
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

## 4. State Server Caching Strategy

The State Server is explicitly **not** the Source of Truth. It is highly optimized for fast external delivery:

*   **Hot Cache (The Window):** Maintains a rolling window of the absolute latest activity (e.g., the last 5 minutes of chat, the currently active agent progress bars). It is populated purely by asynchronously listening to the Event Bus pointers. UI clients receive sub-millisecond responses from this cache.
*   **Warm Cache (On-Demand):** When an external client (UI) requests data outside the Hot Window, the State Server intercepts the request, validates ABAC, queries the Database Views (ClickHouse/Milvus), and temporarily stores the result in a Warm Cache before returning it to the client.

---

## 5. Stress-Test Scenarios

### Scenario 1: Deep Scientific Research Lifecycle (10,000+ Parallel Agents)
*A massive burst of worker processes generating hypotheses, code, and datasets.*

*   **Action:** 10,000 agents simultaneously chunk L3 data using two different strategies and embed them using three different models.
*   **Data Path:** Agents write their `Chunks` JSON and `Embeddings` Parquet files directly into Object Storage (S3). They do **not** talk to the State Server or the Databases. They publish URI pointer events to NATS.
*   **Database View:** ClickHouse instantly sees the new text chunks via its S3 mount. Milvus Sync Workers pull the NATS events and index the Parquet vectors, storing only the float arrays and the `chunk_hash`.
*   **Stress Point Survived:** The State Server and Databases are protected from massive write contention because Storage absorbs the parallel burst effortlessly, and data duplication is avoided via the 1:N:M Tri-Plane architecture.

### Scenario 2: YouTube Translate + Dub + Lip-Sync
*ASR -> Translate -> TTS -> Align -> Render pipeline.*

*   **Action:** A complex, multi-stage pipeline transcribes an original L0 video, translates it, synthesizes new audio, and renders a lip-synced composite video tailored for a target culture.
*   **Data Path (Storage):** Every intermediate artifact is written directly to the Tri-Plane Storage as an immutable record:
    *   **L0:** Original MP4 video.
    *   **L1:** Denoised Audio Track (`As-Is`).
    *   **L2:** ASR Timestamps, Visemes (`Chunks`), Text Embeddings (`Embeddings`).
    *   **L3:** Translated Glossary, Synthesized Audio Track (`As-Is`).
    *   **L4:** Final lip-synced composite video MP4 (`As-Is`).
*   **Data Path (Event Bus):** After writing each L1-L4 artifact, workers publish a URI pointer to NATS. 
*   **State Server Role:** Listens to NATS to update the UI's progress bars and stage previews (e.g., showing the translated glossary to the operator). Accepts operator overrides (intents) to edit the glossary.
*   **Stress Point Survived:** **Glossary Edits & Immutability:** If an operator edits the L3 glossary, the system does *not* overwrite the existing L3 artifact. It writes a *new* L3 glossary with a new `as_is_hash` and generates a new branch of the DAG (L4 composite video). The UI seamlessly updates its "published output" pointer to the newest branch, while the old branch remains intact and fully reproducible on Storage.

### Scenario 3: Right to be Forgotten (Compliance & Immutability)
*A user requests the deletion of a massive, heavily processed L0 document.*

*   **Action:** User clicks "Delete Document" in the UI.
*   **Data Path:** The UI sends an intent to the State Server. The State Server validates ABAC and hands the intent to a Reliable Component.
*   **The Immutable Ledger:** The Reliable Component does **not** delete the file from S3 or issue `DELETE` statements to the DBs. Instead, it writes a new, immutable record to the `s3://.../exclusions/` prefix: `{ target_hash: "as_is_hash_xyz", reason: "user_request" }`.
*   **Database View (Overlay):** ClickHouse and Milvus dynamically construct their "Search Views" using an anti-join or filter against the Exclusions ledger (`WHERE hash NOT IN (SELECT target_hash FROM exclusions)`).
*   **Stress Point Survived:** The document and all its derivative chunks/embeddings instantly vanish from UI search results and agent context windows, satisfying compliance without violating the append-only, immutable guarantee of the Storage truth.

### Scenario 4: Massive Video Proxying (LakeFS / Streaming)
*A user requests to view an original 4GB L0 raw MP4 file.*

*   **Action:** User clicks "View Original Source" in the UI.
*   **Data Path:** UI requests the file from the State Server.
*   **State Server Role:** Validates ABAC permissions, generates a Pre-Signed URL pointing directly to the specific version commit in lakeFS/S3, and returns the URL.
*   **Stress Point Survived:** The State Server avoids OOM (Out Of Memory) crashes and bandwidth saturation by never proxying large blob traffic.

### Scenario 5: Real-Time Audio/Text Streaming Chat
*User speaking to an AI agent with millisecond latency requirements.*

*   **Action:** User initiates a live voice conversation with an AI Agent.
*   **Data Path (Transport):** State Server validates ABAC and issues a token. The UI establishes a direct WebRTC (LiveKit) or WebSocket connection with the AI Agent. Audio/Text streams directly between UI and Agent, bypassing the State Server and DBs entirely to achieve <50ms latency.
*   **Data Path (Record):** The AI Agent acts as the Reliable Component. As it processes the live audio, it asynchronously chunks the transcript and synthesized audio, writes them to the Tri-Plane Storage, and fires NATS pointer events.
*   **Stress Point Survived:** Achieves ultra-low latency for the user experience while guaranteeing that all interactions are durably recorded to the single source of truth without bottlenecking the real-time transport layer.
