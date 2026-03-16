# State Server Stakeholders And Sources Of Truth

## 1. Stakeholders & Access Matrix

| Stakeholder / Component | Role | Access Type | Data Handled / Transferred |
| :--- | :--- | :--- | :--- |
| **Control Center (Operator)** | Active Human Operator | Write / Read | Writes `intents` (commands). Reads lossless `state_diffs`, `snapshots`, and pointers (URLs/IDs) to artifacts and memory. |
| **Observer UI** | Read-Only Viewer | Read | Reads best-effort, RBAC-filtered `state_diffs` and `snapshots`. Views sanitized progress and artifacts. |
| **Real-Time Ingestion Pipeline** | High-Throughput Data Plane | Write Only | Ingests, processes, and writes L0-L4 artifacts directly to ClickHouse/Milvus/Iceberg. Bypasses State Server. Emits metadata pointers via NATS. |
| **LiveKit Server** | Real-Time Media Plane | Stream | Handles raw audio/video streaming (e.g., live voice, WebRTC). Bypasses Temporal. Feeds directly into the Ingestion Pipeline. |
| **Agents (DeerFlow/Hermes)** | Task Execution Workers | Emit Only | Emit `agent_events` and tool outputs to the Ingestion Pipeline. *Never directly mutate canonical UI state.* |
| **Temporal** | Control Plane (Orchestrator) | Read / Emit | Reads mission config. Orchestrates macro-jobs (retries, timers). Waits for HITL approvals from State Server. *Does not handle the data plane.* |
| **Dapr Sidecars** | Runtime Plumbing | Pub/Sub / Invoke | Routes normalized events between polyglot microservices via mTLS. |
| **NATS JetStream** | Event Bus | Pub/Sub | High-volume fanout for telemetry, lineage, and pipeline metadata notifications. |
| **State Server** | UI / Mission Gatekeeper | Read / Write | Sole writer of UI intent and mission state. Subscribes to NATS for artifact pointers to update the UI. |
| **Memory Policy Service** | Memory Manager | Read / Emit | Reads artifact changes. Emits promotion/demotion events for memory tiers. |
| **Observability Stack** | Tracing & Metrics (OTel) | Read | Consumes traces/metrics. Requires correlation IDs propagated by State Server and Pipeline. |

---

## 2. Data Architecture & Sources of Truth

| Data Category | Tier | Immutable Source of Truth (Cold/Log) | Fast Lookup / Cache (Warm/Hot) | Description & Examples |
| :--- | :--- | :--- | :--- | :--- |
| **UI State** | Canonical | `state_events` (Iceberg + S3) | State Server Memory / Redis | The materialized view of the UI (scene graph, widgets) rebuilt from events. |
| **Artifacts** | **L0: Crude Data** | Object Store + `artifacts_raw` | - | **Raw, untouched sources:** Original PDFs, raw MP4s, unparsed log files, raw CAD meshes, raw TSV/CSV dumps, raw web scrapes. |
| **Artifacts** | **L1: Refined Data** | `artifacts_refined` (Iceberg) | ClickHouse (OLAP) + Milvus (Vector) | **Cleaned & Standardized:** Filled NAs, TSV converted to JSON, denoised audio, valid Markdown. |
| **Artifacts** | **L2: Information** | `artifacts_features` (Iceberg) | ClickHouse (OLAP) + Milvus (Vector) | **Extractions & Views:** OCR text, ASR transcripts, visemes, embedding vectors, object bounding boxes, parsed ASTs, extracted claims. |
| **Artifacts** | **L3: Knowledge** | `artifacts_knowledge` (Iceberg)| ClickHouse (OLAP) + Milvus (Vector) | **Aggregations & Semantics:** Translated text, semantic summaries, mathematical proofs, entity-relationship graphs, synthesized answers. |
| **Artifacts** | **L4: Intelligence**| `artifacts_intel` (Iceberg) | ClickHouse (OLAP) + Milvus (Vector) | **Insights & Predictions:** Generated hypotheses, what-if simulations, future research directions, predictive corollaries, fully synthesized papers. |
| **Lineage** | Cross-Cutting | `lineage_events` (Iceberg) | ClickHouse (Graph/Relational) | The directed acyclic graph (DAG) tracking provenance across all L0-L4 artifacts (e.g., "L3-Answer derived from L2-Chunk_A and L1-Doc_B"). |

*Note: Redis, ClickHouse, and Milvus indexes are rebuildable caches/views. They are NOT the absolute system of record.*

---

## 3. Stress-Test Scenarios

### Scenario 1: Q&A Over Private Google Drive Files
*Realtime answer updates when source files change.*

| Dimension | Details |
| :--- | :--- |
| **Sources of Truth** | **L0**: Drive Blob.<br>**L1**: Cleaned Markdown.<br>**L2**: Text Embeddings/Chunks.<br>**L3**: Synthesized Answer.<br>**L4**: Strategic insight/recommendation based on answer. |
| **State Server Role** | Reflects extraction progress via NATS metadata. Updates pointer to newest L3 answer (based on auto-refresh policy). |
| **Stress Points** | **Partial Updates**: Must show deterministic provenance (e.g., "Answer based on Rev 1, Rev 2 is 60% processed").<br>**RBAC Drift**: If user loses Drive access mid-session, State Server must revoke read access instantly.<br>**Consistency**: Avoid silent answer mutation; either version answers or explicitly mark updated. |

### Scenario 2: YouTube Translate + Dub + Lip-Sync
*ASR -> Translate -> TTS -> Align -> Render pipeline.*

| Dimension | Details |
| :--- | :--- |
| **Sources of Truth** | **L0**: Source Video.<br>**L1**: Denoised Audio Track.<br>**L2**: ASR Timestamps, Visemes.<br>**L3**: Translated Glossary, Generated Audio.<br>**L4**: Final lip-synced composite video tailored for target culture. |
| **State Server Role** | Exposes job DAG state, completion ETAs, stage previews, and accepts operator overrides (e.g., glossary edits). |
| **Stress Points** | **Glossary Edits**: Operator edits create a new workflow branch. State Server must track branch lineage.<br>**Retries**: Never overwrite; append new render artifacts and update "published output" pointer. |

### Scenario 3: Deep Scientific Research Lifecycle
*Survey -> Hypothesis -> Experiments -> On-the-fly Code -> Paper/Slides.*

| Dimension | Details |
| :--- | :--- |
| **Sources of Truth** | **LakeFS**: Research corpus snapshots (reproducibility).<br>**L1**: Normalized Datasets.<br>**L2**: Extracted metrics from on-the-fly code.<br>**L3**: Proofs, Entity graphs, Paper sections.<br>**L4**: Novel hypotheses, Future research directions, Final published Intelligence. |
| **State Server Role** | Tracks decision points, experiment queues, explicit approvals, and publication assembly views. |
| **Stress Points** | **Parallelism**: Ingestion Pipeline handles 10,000+ parallel agent outputs. State Server orchestrates the single canonical plan without bottlenecking on artifact writes.<br>**Code Execution**: Code/Outputs treated as artifacts; Temporal runs the code; State Server tracks metrics.<br>**Reproducibility**: Every figure/table references artifact IDs (dataset version + code hash + run id). |

---

## 4. Additional Edge-Case Scenarios

| Scenario | Challenge | State Server Responsibility |
| :--- | :--- | :--- |
| **Multi-Tenant Enterprise** | Strict isolation and compliance. | Apply Casbin ABAC at read-time. Ensure immutable audit export. |
| **Right to be Forgotten** | Purging data without breaking UI. | Render historical state gracefully when underlying L0/L1 blobs are tombstoned. |
| **Massive Broadcast** | 1 Operator, 10,000 Observers. | Maintain lossless ack/backpressure for operator; downsample/throttle diffs for observers. |
| **Long-Running Mission** | Multi-day runs with failovers. | Fast resume via snapshot + cursor. Prevent duplicate transitions using idempotency tokens. |
| **Model Upgrades** | Upgraded L2/L3 artifact generation. | Allow operator to visually compare old vs. new pipeline outputs before publishing. |
| **Adversarial Inputs** | Prompt injection in docs/videos. | Record tool allow/deny decisions and show "blocked by policy" events. |
| **Multimodal Voice/Video** | Agent LiveKit sessions. | Retain truth for session state, transcript pointers, and agent control intents (LiveKit is media transport only). |
| **Fork/Merge Branches** | Operator explores a fork. | Reflect branch pointers and the final "published branch" selection. |

---

## 5. Architectural Mandate

**Dual-Plane Truth Contract:**
To prevent write bottlenecks and workflow bloat, the system strictly separates the Control Plane from the Data Plane:

1. **Control Plane (UI Truth):** The State Server is the SOLE writer of `state_events` (Mission progress, HITL approvals, UI intents, agent lifecycles). It must never gatekeep or serialize high-volume artifact data.
2. **Data Plane (Artifact Truth):** The Real-Time Ingestion Pipeline is the high-throughput writer for L0-L4 artifacts into Iceberg/ClickHouse/Milvus. Agents, extractors, and LiveKit emit raw data/features directly to this pipeline. The State Server merely subscribes to lightweight *metadata pointers* from the pipeline via NATS to update the UI.