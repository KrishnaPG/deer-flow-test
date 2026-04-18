# Embed Hermes ACP Producer

## Summary
Provide a Rust-managed Hermes producer that launches Hermes ACP as a subprocess over JSON-RPC stdio, captures every observable ACP frame and producer-issued command envelope, and durably publishes that raw traffic to Redpanda topic `hermes.l0_drop` before any ingestion or Berg10 interpretation occurs.

## Why
- Keeps the first write path extremely fast and durable: Redpanda is the primary L0 landing zone, not an ETL layer.
- Preserves enough raw fidelity to reconstruct full Hermes sessions for replay, audit, and future real-time agentic chat UI.
- Avoids modifying `3rdParty/hermes-agent`; integration happens entirely through the ACP boundary.
- Makes sequencing and durability our responsibility instead of relying on internal Hermes storage semantics.
- Cleanly separates producer concerns (capture, order, durability, replay metadata) from ingestion concerns (classification, Berg10 mapping, normalization, projection).

## Scope
In-scope:
- A Rust producer that launches and manages Hermes ACP over JSON-RPC stdio.
- Redpanda-first raw capture to topic `hermes.l0_drop`.
- Preservation of every observable boundary event the producer can capture, including:
  - producer-issued commands such as prompt, cancel, session create, session load, and resume
  - ACP requests, responses, and notifications observed on the subprocess boundary
  - subprocess lifecycle events that materially affect reconstruction (spawned, exited, crashed, restarted, timeout)
- Stable replay metadata on each published envelope:
  - `session_id`
  - `run_id`
  - `seq`
  - `timestamp`
  - optional `thread_id`, `message_id`, `tool_call_id`, and similar fields when observable
- A sequencing model owned by the Rust producer, scoped monotonically within `session_id`.
- A process model that supports multiple sessions per ACP subprocess, while defaulting to one subprocess per session for initial isolation.
- Redpanda-backed replay as the durable recovery source, with optional in-memory replay tail as an optimization.
- Use of `BERG10_BASE_DIR` for runner-owned paths under `<base_dir>/runners/hermes/` when local state or diagnostics are needed.

Out-of-scope (this change):
- Berg10 field mapping, `data_hierarchy` classification, normalization, and any other domain interpretation.
- Ingestion from `hermes.l0_drop` into `crates/berg10/storage-vfs/` and `crates/berg10/storage-catalog/`.
- Stable public HTTP/WebSocket client APIs.
- File fallback as a primary path; this change is explicitly Redpanda-first.
- Any source changes under `3rdParty/`.

## Acceptance Criteria
- The producer launches Hermes ACP as a subprocess and communicates using JSON-RPC over stdio.
- The producer can create or resume Hermes sessions and execute prompts as runs within those sessions.
- Every observable boundary event the producer can capture is published to Redpanda topic `hermes.l0_drop`.
- Published envelopes preserve raw ACP payloads and producer command payloads without Berg10 mapping or semantic reinterpretation.
- Each published envelope includes stable replay metadata with `session_id`, `run_id`, `seq`, and `timestamp`, plus optional identifiers when available.
- `session_id` is used as the Redpanda message key.
- `seq` is assigned by the Rust producer and is monotonic within a session.
- The implementation supports multiple sessions per subprocess at the abstraction level, while the default runtime policy uses one subprocess per session.
- Redpanda producer settings provide high durability and ordering safety (`acks=all`, idempotence enabled, retries with backoff, ordering-safe in-flight configuration).
- Redpanda remains the durable source of raw truth until downstream ingestion succeeds; producer-side storage interpretation is not performed.
- The design keeps `thread_id` optional so future manual session branching can attach it when a real source exists.

## Risks & Mitigations
- **Subprocess Instability:** Hermes ACP can hang, crash, or emit malformed frames.
  - *Mitigation:* Treat stdout as protocol-only, stderr as diagnostics, add heartbeat/timeouts, emit lifecycle envelopes, and allow supervised restart.
- **Reconstruction Gaps:** Missing prompt/cancel or protocol frames would weaken replay fidelity.
  - *Mitigation:* Persist every observable producer-side command and every observable ACP frame, not only assistant-visible events.
- **Ordering Drift:** Multiple runs inside one session can interleave unexpectedly.
  - *Mitigation:* Use `session_id` as the partition key and assign session-scoped monotonic `seq` in the producer.
- **Storage Growth:** Full raw capture increases broker retention cost.
  - *Mitigation:* Allow compression, keep retention operationally long-lived, and defer deletion to successful downstream ingestion policy.
