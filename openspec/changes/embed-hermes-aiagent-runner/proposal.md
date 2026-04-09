# Embed Hermes AIAgent Runner

## Summary
Provide a backend-hosted Hermes runner service, implemented in Rust, that wraps an in-process Hermes `AIAgent`. The runner exposes a transport-neutral HTTP plus WebSocket API that both desktop and web/WASM clients can use, while simultaneously publishing raw, unmapped Hermes session traffic into Redpanda on the `hermes.l0_drop` topic.

## Why
- Avoids coupling Hermes execution to a desktop-only stdin/stdout subprocess model that cannot work for web/WASM clients.
- Enables one backend integration path for `deer_gui`, future web/WASM UIs, and thin labs.
- Keeps storage canonical and durable: raw Hermes session/request/response traffic is durably buffered before being ingested into the Berg10 content-addressable VFS as L0 records.
- Preserves enough raw fidelity to reconstruct the full Hermes conversation/session, including user requests, assistant responses, tool calls, tool results, reasoning, and finish markers.
- Provides a clear boundary between raw data capture (Hermes producer) and structured ingestion/normalization (future ingestor).

## Scope
In-scope:
- A backend-hosted Hermes runner component that instantiates `AIAgent`.
- A transport-neutral API for starting runs, canceling runs, and subscribing to stream events.
- Storage path resolution respects `<base_dir>/runners/<engine_name>/` hierarchy:
  - Runner-owned files: `<base_dir>/runners/hermes/...`
  - Supporting files: `<base_dir>/runners/hermes/skills/`, `<base_dir>/runners/hermes/config/`, etc.
- Dual-sink implementation: 
  - Emit stream events over a backend API suitable for desktop and web/WASM clients.
  - Publish raw, unmapped Hermes callback payloads to Redpanda topic `hermes.l0_drop`.
- Reliable Redpanda producer semantics: idempotent producer, `acks=all`, retry/backoff, and durable buffering until a future ingestor marks items as successfully ingested.
- Session reconstruction contract: published messages carry stable `session_id`, `thread_id` (when available), `run_id`, `message_id`, `tool_call_id`, `seq`, and finish markers.
- Replay support for reconnecting clients, using a small in-memory event window in the runner and Redpanda-backed replay when the requested sequence is older than the in-memory cache.
- Desktop and lab clients use the same backend API instead of directly spawning Python.
- Environment variable `BERG10_BASE_DIR` is used by the backend-hosted Hermes service for runner path resolution.

Out-of-scope (initial iteration):
- Full ingestion pipeline implementation on the Rust side (producer-side capture is the current priority; ingestor comes later).
- Full UI features (we are just providing the streaming data to the existing GUI).
- File-based `l0_drop` fallback under `<base_dir>/runners/hermes/l0_drop/...` beyond documenting the design for lower-priority dev usage.

## Acceptance Criteria
- A backend Hermes runner service can start a Hermes session/run on request.
- Sending a prompt from a client streams live text and event updates back through a WebSocket event stream.
- The runner exposes `POST /runs`, `POST /runs/{run_id}/cancel`, and `WS /runs/{run_id}/events` as the initial stable API surface.
- Concurrently, raw Hermes callback events are published to Redpanda topic `hermes.l0_drop`.
- Published messages preserve enough identifiers and ordering metadata to reconstruct the full session/thread transcript.
- Producer uses reliable delivery settings (`acks=all`, idempotence enabled, retries with backoff, strict ordering-safe in-flight configuration) and surfaces publish failures cleanly to the UI.
- Raw payload may be gzip-compressed before publish to reduce Redpanda storage usage, while remaining opaque to the producer path.
- Data remains in Redpanda until a future ingestor marks it as successfully ingested; short retention is not allowed.
- Default retention budget is long-lived (target: one month), with operational expectation that ingestors run at least daily in the worst case.
- `deer_gui` and any future WASM/web UI can use the same API contract without subprocess-only assumptions.
- Reconnecting clients can request replay from a prior `seq`, and the runner replays events without requiring the client to reconstruct missing state on its own.

## Risks & Mitigations
- **Broker Outage or Backpressure:** Redpanda unavailability could block or fail session capture.
  - *Mitigation:* Use idempotent producer settings, bounded retries with backoff, explicit UI error surfacing, and keep file-based fallback documented for low-priority dev use.
- **Message Growth:** Full-fidelity raw session capture can consume significant broker storage.
  - *Mitigation:* Allow gzip-compressed payloads, use long retention with operational monitoring, and only delete data after successful ingestion has been recorded.
- **Ordering Drift:** Reconstructing sessions from streamed callbacks requires stable ordering metadata.
  - *Mitigation:* Partition by `session_id`, include monotonic `seq` values within the session, and emit explicit finish events for each completed assistant message.
- **Reconnect Gaps:** UI clients can disconnect mid-run and miss deltas or tool progress events.
  - *Mitigation:* Use WebSocket delivery for live fan-out, retain a short in-memory replay window per run, and fall back to Redpanda replay when reconnect gaps exceed local cache.
