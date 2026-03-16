# State Server Stakeholders And Sources Of Truth

## Stakeholders And Who Touches The State Server
- Control Center (operator UI): writes `intent` commands; reads lossless ordered `state_diffs` + periodic `snapshots`; needs canonical `mission_state` (scene graph, widgets, objectives, approvals, run progress), plus references to artifacts/memory (IDs, cursors, URLs), not the blobs themselves.
- Observer UIs (read-only): reads best-effort `state_diffs`/`snapshots`; needs sanitized views (RBAC-filtered) + high-level progress + selected artifact previews.
- Human operators (people): indirectly write via UI intents; need auditability (who changed what, when, why) and reproducible what-was-true-at-time-T.
- Agents (DeerFlow/Hermes/Rowboat workers): never directly mutate canonical UI state; emit `agent_events`/tool results to orchestration lanes, which the State Server consumes and converts into canonical state transitions.
- Orchestrator (Temporal workflows): owns durable step execution, retries, timers, HITL gates; it does not own UI truth; emits workflow events (started/step/progress/completed/failed) that are reflected into State Server canonical state.
- Runtime plumbing (Dapr sidecars + mTLS): provides invocation/pubsub/bindings so every service can publish/consume normalized events without bespoke networking; State Server is a Dapr participant and policy gate.
- Event bus (NATS JetStream): high-volume fanout/replay/backpressure for non-UI lanes (telemetry, lineage, memory promotions); State Server consumes/produces but should not be the sole relay for sustained telemetry.
- Storage + artifact graph (object store + Iceberg + lakeFS + Postgres hot tables): is the system-of-record for artifacts and lineage; State Server stores pointers + what the operator sees right now.
- Memory layers (Redis hot, Postgres warm/pgvector, Milvus cold macro): are retrieval substrates; State Server only needs current working set pointers (selected memory items, retrieval plan, citations list, cache keys).
- Policy/audit (Casbin ABAC + audit log): State Server is the enforcement point for any canonical state append; must record every intent/event with `mission_id`, `run_id`, `agent_id`, `trace_id`.
- Observability (OTel/Langfuse/Prometheus): reads traces/metrics; State Server must propagate correlation IDs and emit spans, but is not a source of business truth.

## What Data Lives Where (So We Actually Have A Single Source Of Truth)
- Single source of truth for what happened: append-only `state_events` (and `lineage_events`) in the immutable store (object store + Iceberg). Nothing mutates history.
- Single source of truth for what exists (files, transcripts, embeddings, derived outputs): the hierarchical artifact graph:
  - L0 raw blobs in object storage + `artifacts_raw` rows
  - L1 derived primitives in `artifacts_features`
  - L2 semantic summaries in Postgres (hot) + Iceberg (cold)
  - L3 operational metadata/lineage in Postgres (hot) + Iceberg (cold)
- Single source of truth for what the UI should show now: State Server derived materialized state (rebuilt from `state_events`), delivered as `snapshots` + `diffs`.
- Not a source of truth: Redis/Postgres snapshots/caches, vector indexes. They are derived and rebuildable from artifacts + events.

## Concrete Access Matrix (Who Needs What From State Server)
- Operator UI (write + read): `intents` (start mission, upload/select artifacts, run agent, approve step, fork branch, pin citation, publish report), `snapshots/diffs`, cursor resume tokens.
- Observer UI (read): filtered `snapshots/diffs`, redacted artifact refs, progress timeline, published outputs.
- Temporal (read + emit): reads minimal mission config (or is passed it at start); emits workflow step events; waits for HITL signals sourced from State Server approvals.
- Agent workers/services (emit only): tool outputs + artifact creation events + progress; never write canonical UI state directly.
- Memory policy service (emit + read pointers): reads artifact/summary changes; emits promotions/demotions; State Server consumes only what changes operator-visible state (e.g., new summary ready, citations updated).
- Ingest/connectors (Drive/YouTube/etc.) (emit): emit artifact version changed events + write new blobs; State Server reflects source changed and re-deriving outputs.

## Stress-Test Scenarios (End-to-End Truth + Update Propagation)

### 1) Q&A Over Private Google Drive Files; Realtime Answer Updates When Files Change
- Sources of truth:
  - Raw file versions: L0 blob + `artifacts_raw` (include `drive_file_id`, `drive_revision_id`, checksum, RBAC labels).
  - Extracted text/OCR: L1 in `artifacts_features` keyed by `(source_artifact_id, extractor_version, revision_id)`.
  - Answers: L2 summary objects with explicit provenance (which revisions, which chunks, which model/version).
- Who touches State Server and for what:
  - Operator UI: selects folder/files, asks question, pins allowed doc set, sets auto-refresh on change.
  - Drive connector: emits revision-updated event + stores new L0; triggers Temporal extraction workflow.
  - Temporal: runs extraction -> embedding -> summary refresh; emits progress events.
  - State Server: reflects progress + updates current answer pointer to newest L2 answer only if policy allows (e.g., operator opted into auto-refresh); otherwise shows new revision available.
- Real-world failure modes to validate:
  - Partial updates: show answer is based on rev X; rev Y processing 60% with deterministic provenance.
  - RBAC drift: if a user loses access to a file, State Server must stop serving it and invalidate derived outputs for that user view (policy enforced at read time, not just write time).
  - Consistency: avoid silent answer mutation; either version answers (recommended) or explicitly mark updated.

### 2) YouTube Translate + Dub + Lip-Sync Pipeline (ASR -> Translate -> TTS -> Align -> Render)
- Sources of truth:
  - Original video: L0.
  - ASR transcript + timestamps: L1 (time-aligned segments).
  - Translation segments + glossary constraints: L1/L2 depending on structure.
  - Generated audio + visemes/phoneme alignment: L1 artifacts.
  - Rendered translated video: L2 deliverable.
- State Server needs to expose:
  - Job graph state (stages, percent, ETA), per-stage artifacts pointers, preview links, and operator overrides (voice selection, glossary edits, re-render).
- Update semantics:
  - Operator edits glossary: emits intent -> new workflow branch; State Server shows branch lineage (render v3 uses glossary v2).
  - Retry/rollback: never overwrite; append new render artifacts and update the current published output pointer.

### 3) Deep Scientific Research Lifecycle (Survey -> Direction Choice -> Hypothesis -> Experiments/Code -> Results -> Paper + Slides + Images)
- Sources of truth:
  - Research corpus snapshots: artifact set with lakeFS branch/release tags (reproducibility).
  - Claims, citations, experiment configs, code, datasets, run logs, metrics: artifacts + lineage events.
  - Paper/slides/images: L2 deliverables with provenance to datasets/runs/citations.
- State Server canonical state should include:
  - Decision points (picked direction, hypothesis), task DAG status, experiment queue, approvals (e.g., run code on dataset X), and a publication assembly view (sections, figures, citations list).
- Critical stress points:
  - Parallelism: many agents produce artifacts concurrently; State Server remains the single canonical what-is-the-current-plan + what-is-approved + what-is-published.
  - Reproducibility: every figure/table references artifact IDs (dataset version + code hash + run id).
  - On-the-fly code: treat code and outputs as artifacts; execution is a Temporal activity; State Server shows status + links + extracted metrics.

## More Scenarios To Stress-Test (Recommended)
1. Multi-tenant enterprise: per-tenant RBAC + audit export + legal hold; ensure State Server read filtering and immutable event retention works under strict compliance.
2. Right to be forgotten / retention enforcement: tombstone Iceberg rows + purge blobs; State Server must render historical states with redactions while preserving audit integrity.
3. Live demo with 1 operator + 10k observers: observers get downsampled diffs; operator stays lossless with ack/backpressure and snapshot fallback.
4. Long-running mission (days): State Server failover + resume from snapshot+cursor; verify no duplicate UI transitions (idempotency tokens).
5. Model upgrades: new extractor/model versions generate new L1/L2 artifacts; State Server lets operator compare outputs and choose which becomes published.
6. Adversarial inputs (prompt injection in docs/videos): policy gate prevents tools from exfiltrating; State Server records tool allow/deny decisions and shows blocked-by-policy events.
7. Multimodal voice/video agent (LiveKit): LiveKit is transport for media only; State Server stays truth for session state, transcript pointers, and agent control intents.
8. Fork/merge research branches (lakeFS): operator forks a mission branch, explores, then merges metadata; State Server reflects branch pointers and published-branch selection.

## Open Question (Determines The Cleanest Single Truth Contract)
Should the State Server be the only writer of `state_events` (recommended: yes), with all other services publishing to NATS/Dapr and the State Server translating those into canonical state transitions?

If no, we need a stricter shared command/event schema + conflict policy because multiple writers to canonical state complicate audit and replay semantics.
