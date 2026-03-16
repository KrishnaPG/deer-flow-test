# State Server Architecture Notes

## Core Responsibilities
- Canonical UI/agent state authority (scene graph, widgets, objectives, agent lifecycle).
- RBAC + policy enforcement for operator vs observers; audit logging for every intent/event.
- Adapters to orchestrators (DeerFlow, Hermes, Rowboat, Temporal workflows) with normalized event schema.
- Emit UI state diffs over realtime transports (WS/SSE/Centrifugo). LiveKit optional for multimodal audio/video only.

## Orchestration & Runtime Plumbing (Permissive Licenses)
- **Temporal** (MIT) – durable workflows, retries, timers, HITL pauses/resume. Ideal for agent runs + approval gates.
- **Dapr** (Apache-2.0) sidecars are required (Rust + TypeScript/Bun + Python). We lean on its service invocation, pub/sub, bindings, and state abstractions so every polyglot microservice can emit/consume consistent events over mTLS/zero-trust lanes.
- **NATS JetStream** (Apache-2.0) – event bus for replay/backpressure when broadcasting state or ingesting orchestrator events. Temporal task queues handle workflow steps; NATS connects ancillary services (memory ranking, summarizers) through Dapr components.
- **Centrifugo** (Apache-2.0) – scalable WebSocket/SSE hub if we outgrow direct WS from State Server.

## Transport Strategy
- Default realtime channel: WS/SSE (or Centrifugo) for Control Center state. Single operator simplifies concurrency; observers subscribe to read-only feed.
- **LiveKit** (Apache-2.0) only when multimodal (voice/video/physical agent) needed; otherwise omit to reduce ops surface.

## Storage & Data Lake (Immutable-first)
- Object storage (S3-compatible or filesystem) as primary log store; append-only Parquet + optional Markdown/PNG/GLB for human review at the "raw artifact" tier.
- **OpenDAL** (Apache-2.0, Rust) / **fsspec** (BSD-3, Python) / **go-cloud/blob** (Apache-2.0, Go) abstract S3 vs local FS; **SeaweedFS** (Apache-2.0) if we need self-hosted S3 API.
- **Apache Iceberg** (Apache-2.0) is the default table format w/ the native catalog (REST or Nessie-like) for machine access. Iceberg stores metadata as object-store files, so lakeFS can branch/merge that metadata alongside the Parquet data, giving per-mission review branches and reproducible releases. **lakeFS** (Apache-2.0) rides on top for human-facing versioned branches/releases of datasets + audit diffs.
- DuckLake: viable alternative when the analytics stack is DuckDB-first; if adopted, run separate DuckLake catalog DBs per lakeFS branch/environment to preserve isolation.
- Analytical access: ClickHouse, DuckDB, Trino reading Parquet/Iceberg directly via the above abstractions.

### Hierarchical Artifact Graph
- **Level 0 – Raw**: entire uploaded files (images, CAD meshes, notebooks) persisted as immutable blobs with coarse metadata (owner, upload time, RBAC labels). Stored in object store, indexed in Iceberg `artifacts_raw`.
- **Level 1 – Derived Primitives**: computer vision tags, OCR text, ASR transcripts, embedding vectors. Generated via Temporal activities; stored alongside provenance columns (`source_artifact_id`, `extractor_version`) in Iceberg `artifacts_features`.
- **Level 2 – Semantic Summaries**: model-authored narratives, structured JSON, checklists. Persisted in Postgres for quick lookup + Iceberg for long-term analytics. Each summary references both raw artifacts and derived primitives.
- **Level 3 – Operational Metadata**: relations across runs ("image used in mission #42"), lineage edges into workflows, retention + compliance flags. Sits in Postgres (hot) and Iceberg (cold) for time-travelled audits.
- Dapr components expose bindings to each tier so specialized services (e.g., object detector) can operate without knowing physical storage details.

## Memory & Retrieval Stack
- **Cognitive tiers**
  - Hot lane (Redis): working set per mission (scene graph deltas, recent human prompts, short token windows). Bounded TTL; summarizer writes compressed notes into warm lane.
  - Warm lane (Postgres + pgvector): mission/session metadata, RBAC, structured tool outputs, embedding shards for the most recent cycle. Acts as lookup cache before vector store.
  - Cold lane (Iceberg on object store): durable append-only event log, full artifacts, multi-hop lineage. Compression-friendly, query via DuckDB/Trino.
- **Vector hierarchy**
  - Micro-embeddings (token-level, prompt snippets) stored in Redis Streams for in-flight relevance feedback.
  - Mesoscale embeddings (document sections, tool traces) tracked in pgvector for transactional augmentations.
  - Macro embeddings (artifact-level, mission summaries) live in Milvus (Apache-2.0) with collection-per-level schema. Metadata fields reference artifact graph levels for filtered retrieval.
- Memory policy service orchestrates demotion/promotion between tiers, ensures derived primitives (Level 1) and semantic summaries (Level 2) stay in sync with embeddings, and emits events to Temporal when large artifacts require asynchronous processing.

## Observability & Telemetry
- **Trace backbone**: OpenTelemetry spans from UI prompt → State Server intent → Temporal workflow steps → downstream Dapr services. Export to OTLP/ClickHouse via SigNoz or Tempo; correlate with mission IDs.
- **LLM metrics**: Langfuse (MIT) captures latency, cost, token usage per model call. 
- **Agent instrumentation**: DeerFlow exposes run metadata via its event stream but lacks first-class OTEL exporters. Wrap its hooks with custom interceptors to emit spans/logs. Hermes-Agent currently surfaces internal events via logging/webhooks; add lightweight shims (Python OTEL SDK) to translate to our trace schema.
- **Artifact lineage observability**: every artifact tier mutation emits to NATS (through Dapr pub/sub) and lands in Iceberg `lineage_events`. UI dashboards show prompt→agent→artifact→memory chain, with ability to fork into child tracker.
- **User/system metrics**: Prometheus scraping Dapr/Temporal/Redis plus Redpanda/NATS connectors. Grafana dashboards blend infra metrics with OTEL traces for full-loop visibility.

## Operational Guardrails (Industrial-grade)
- **Canonical state model**: State Server is event-sourced. Commands never mutate prior facts; they append new events to the log (object store + Iceberg `state_events`). Periodic snapshots (Redis + Postgres) provide fast resume and are purely derived from immutable history. Snapshots are checkpointed per mission ID, letting reconnecting clients replay from the last acknowledged cursor.
- **Availability**: deploy active/active State Server replicas behind sticky load balancing. Only a single replica per mission holds the leadership lease (stored in Redis w/ fencing tokens). On failover, followers load the latest snapshot + replay diff log before resuming diff emission.
- **Transport policies**: UI diff stream guarantees ordered delivery per mission (sequence numbers, idempotency tokens). Observers are best-effort; operators receive lossless delivery with backpressure (client must ack within N seconds or gets downgraded to snapshot mode). Binary protobuf or FlatBuffers payloads keep latency bounded.
- **Workflow/event boundaries**: Temporal handles agent orchestration, approvals, timers. NATS JetStream handles high-volume telemetry + memory promotion events. Dapr sidecars enforce these lanes; never push sustained telemetry through Temporal task queues.
- **Backpressure & load shedding**: prioritize operator session → observability → artifact extraction. For overload conditions: (1) pause Level-1/2 derivations, (2) downsample observer diffs, (3) park non-critical Temporal workflows via rate-limit signals.
- **Security & compliance**: mutual TLS everywhere (Dapr mTLS, Envoy/Linkerd between services). JWT-based auth for humans/automations (minted via centralized IdP); SPIFFE/SPIRE IDs for services. Encrypt object store and DBs; wrap keys via KMS/HashiCorp Vault. Use Casbin ABAC policies (backed by Postgres) to evaluate RBAC + attribute-based constraints before appending state. Retention policies codified per artifact level (e.g., raw artifacts 180 days, summaries 365 days), with automated delete workflows that tombstone Iceberg rows + purge object-store blobs via lakeFS hooks.
- **Correlation contract**: every event/span/log carries `mission_id`, `run_id`, `agent_id`, `artifact_id`, and `trace_id`. Temporal workflow IDs map 1:1 with mission cycles so traces stitched end-to-end.

## Collaboration Model
- Single operator with observers ➝ no need for CRDTs like Yjs/Automerge/Loro. Server remains authoritative and simply streams diffs.
- If future requirement adds multi-operator editing, we can revisit CRDT-backed scene graphs.

## Outstanding Questions
- Decide if DuckLake needs first-class support alongside Iceberg (for DuckDB-heavy operators) or remains optional.
- Flesh out exact snapshot/restore schema for `state_events` vs in-memory cache to prove failover RTO.
- Define audit schema + retention/RBAC policies for the state log.
