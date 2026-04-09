# Embed Hermes AIAgent Runner

## Summary
Provide an in-process Hermes AIAgent adapter (`hermes_bridge.py`) that can be launched as a backend generator. This adapter will act as a dual-sink: it streams live conversation events to the Rust Desktop GUI via `stdout` while simultaneously publishing raw, unmapped Hermes session traffic into Redpanda on the `hermes.l0_drop` topic.

## Why
- Enables live integration with the existing `deer_gui` frontend with zero changes to the Rust UI codebase.
- Keeps storage canonical and durable: raw Hermes session/request/response traffic is durably buffered before being ingested into the Berg10 content-addressable VFS as L0 records.
- Preserves enough raw fidelity to reconstruct the full Hermes conversation/session, including user requests, assistant responses, tool calls, tool results, reasoning, and finish markers.
- Provides a clear boundary between raw data capture (Hermes producer) and structured ingestion/normalization (future ingestor).

## Scope
In-scope:
- A new standalone Python bridge (`apps/deer_gui/python/hermes_bridge.py`) that instantiates `AIAgent`.
- Storage path resolution respects `<base_dir>/runners/<engine_name>/` hierarchy:
  - Runner-owned files: `<base_dir>/runners/hermes/...`
  - Supporting files: `<base_dir>/runners/hermes/skills/`, `<base_dir>/runners/hermes/config/`, etc.
- Dual-sink implementation: 
  - Emit UI-compatible JSON over `stdout`.
  - Publish raw, unmapped Hermes callback payloads to Redpanda topic `hermes.l0_drop`.
- Reliable Redpanda producer semantics: idempotent producer, `acks=all`, retry/backoff, and durable buffering until a future ingestor marks items as successfully ingested.
- Session reconstruction contract: published messages carry stable `session_id`, `thread_id` (when available), `run_id`, `message_id`, `tool_call_id`, `seq`, and finish markers.
- Rust GUI minor wiring to allow selecting the Hermes engine (launching `hermes_bridge.py` instead of `bridge.py`).
- Environment variable `BERG10_BASE_DIR` is passed from Rust to Python for path resolution.

Out-of-scope (initial iteration):
- Full ingestion pipeline implementation on the Rust side (producer-side capture is the current priority; ingestor comes later).
- Full UI features (we are just providing the streaming data to the existing GUI).
- File-based `l0_drop` fallback under `<base_dir>/runners/hermes/l0_drop/...` beyond documenting the design for lower-priority dev usage.

## Acceptance Criteria
- Running the `deer_gui` with the Hermes engine selected launches `hermes_bridge.py`.
- Sending a prompt streams live text back to the `deer_gui` UI.
- Concurrently, raw Hermes callback events are published to Redpanda topic `hermes.l0_drop`.
- Published messages preserve enough identifiers and ordering metadata to reconstruct the full session/thread transcript.
- Producer uses reliable delivery settings (`acks=all`, idempotence enabled, retries with backoff) and surfaces publish failures cleanly to the UI.
- Raw payload may be gzip-compressed before publish to reduce Redpanda storage usage, while remaining opaque to the producer path.
- Data remains in Redpanda until a future ingestor marks it as successfully ingested; short retention is not allowed.
- Default retention budget is long-lived (target: one month), with operational expectation that ingestors run at least daily in the worst case.

## Risks & Mitigations
- **Broker Outage or Backpressure:** Redpanda unavailability could block or fail session capture.
  - *Mitigation:* Use idempotent producer settings, bounded retries with backoff, explicit UI error surfacing, and keep file-based fallback documented for low-priority dev use.
- **Message Growth:** Full-fidelity raw session capture can consume significant broker storage.
  - *Mitigation:* Allow gzip-compressed payloads, use long retention with operational monitoring, and only delete data after successful ingestion has been recorded.
- **Ordering Drift:** Reconstructing sessions from streamed callbacks requires stable ordering metadata.
  - *Mitigation:* Partition by `session_id`, include monotonic `seq` values within the session, and emit explicit finish events for each completed assistant message.
