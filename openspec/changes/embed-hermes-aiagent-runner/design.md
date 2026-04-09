# Design: Embed Hermes AIAgent Runner

## Architecture

The system introduces a backend-hosted Hermes runner service. The initial implementation target is Rust so it aligns with the existing crate ecosystem, testing infrastructure, and long-lived service requirements. It owns Hermes `AIAgent` execution and exposes a transport-neutral API that desktop and web/WASM clients can both consume.

### Components

1.  **UI Clients (`apps/deer_gui`, future web/WASM UI, `apps/deer_chat_lab`)**
    - Use a common backend API instead of spawning Python directly.
    - Send commands such as `create_session`, `send_message`, and `subscribe_run_events`.

2.  **Hermes Runner Service**
    - Hosts the in-process Hermes `AIAgent`.
    - Accepts session/thread/run commands through an HTTP plus WebSocket backend API.
    - Registers callbacks on the `AIAgent` (`stream_delta_callback`, `tool_progress_callback`).
    - Publishes UI stream events and Redpanda raw events from the same callback source.
    - Uses `rust-rdkafka` for Redpanda producer integration.
    - Maintains a small in-memory replay window per run/session and falls back to Redpanda-backed replay for older events.
    
3.  **Sink A: UI Stream API (Real-time)**
    - Maps Hermes text deltas and tool states into a transport-neutral event model.
    - Delivers those events over WebSocket as the primary streaming transport.
    - Uses the same event envelope shape as the Redpanda payload so UI clients do not depend on a second event schema.

4.  **Sink B: Redpanda L0 Buffer (Primary Durable Staging)**
    - Captures the *raw, unmodified* event payloads directly from the Hermes callbacks.
    - Publishes them to Redpanda topic `hermes.l0_drop`.
    - Uses reliable producer semantics: `acks=all`, idempotence enabled, retry/backoff.
    - Keeps the payload opaque; schema-on-read happens later during ingestion.

5.  **Dev Fallback: File-Based L0 Drop (Low Priority)**
    - For lower-priority dev environments without Redpanda, the same raw events may be appended to `<base_dir>/runners/hermes/l0_drop/YYYY-MM-DD/run_<uuid>.jsonl`.
    - This fallback is documented but not the current implementation priority.

## Client/Service Boundary

- The Hermes runner must not depend on local desktop process spawning.
- The backend API is the stable contract; desktop and web/WASM clients are just consumers.
- Initial API shape should cover:
  - `POST /runs` to start a run and return `run_id` and `session_id`
  - `POST /runs/{run_id}/cancel` to cancel an active run
  - `WS /runs/{run_id}/events` to subscribe to live events and request replay from a prior sequence
  - Optional `GET /runs/{run_id}/replay?from_seq=<n>` for non-WebSocket replay and debugging
- WebSocket is the default streaming transport because it supports low-latency fan-out, reconnect handling, and future bidirectional control without introducing a second desktop-only path.

## Implementation Choices

- **Language:** Rust is the recommended and selected implementation language for the initial runner service.
- **Kafka Client:** Use `rust-rdkafka` so the runner inherits mature idempotent producer behavior from `librdkafka`.
- **Live Fan-out:** Use an async broadcast channel per run/session so multiple clients can subscribe without affecting Hermes callback execution.
- **Replay Strategy:** Keep a bounded in-memory event window for fast reconnects, then read Redpanda for replay when the requested `seq` is no longer cached locally.
- **Event Parity:** WebSocket frames and Redpanda values use the same logical envelope fields to minimize adapter complexity in `deer_gui` and future web clients.

## Data Model (Raw Event Shape)

The primary Redpanda payloads contain a pure dump of what the Hermes engine produces. We do not impose canonical schemas at publish time, but we do standardize the envelope required for replay, correlation, and storage.

Example raw event payload:
```json
{
  "engine": "hermes",
  "schema_version": "v1",
  "session_id": "session-1234",
  "thread_id": "thread-1234",
  "timestamp": "2026-04-09T14:22:10Z",
  "run_id": "run-1234",
  "message_id": "msg-42",
  "tool_call_id": null,
  "seq": 18,
  "event_kind": "tool.started",
  "source_callback": "tool_progress",
  "raw_payload": {
    "name": "read_file",
    "status": "started",
    "args": {"path": "/etc/hosts"}
  }
}
```

## Redpanda Topic & Delivery Contract

- Topic name format: `<engine>.l0_drop`.
- Hermes uses topic `hermes.l0_drop`.
- The producer may gzip-compress the message value before publish to reduce broker storage usage.
- Compression metadata must be declared in headers so a future ingestor knows whether to decompress before parsing.
- The producer uses `rust-rdkafka` with `acks=all`, `enable.idempotence=true`, a large retry budget, exponential backoff, and ordering-safe in-flight configuration.
- Messages are retained until successful ingestion has been recorded; short retention is explicitly disallowed.
- Operational target: keep data available for up to one month, while ensuring the ingestor runs at least daily in the worst case.

### Recommended message headers

- `engine=hermes`
- `schema_version=v1`
- `content_encoding=identity|gzip`
- `session_id=<stable session id>`
- `thread_id=<stable thread id when available>`
- `run_id=<single turn/run id>`
- `message_id=<message id when available>`
- `tool_call_id=<tool call id when available>`
- `event_kind=<message.delta|message.finish|tool.started|tool.completed|reasoning.available|session.marker>`
- `producer_id=<bridge instance id>`

### Recommended producer configuration

- `acks=all`
- `enable.idempotence=true`
- `max.in.flight.requests.per.connection=1`
- `compression.type=gzip` when payload compression is enabled
- Delivery retries with exponential backoff and surfaced terminal failure status

### Partitioning

- Use `session_id` as the Kafka/Redpanda message key.
- Rationale: Hermes sessions span multiple runs/turns; partitioning by `session_id` preserves long-lived conversational ordering and makes full-thread reconstruction easier than partitioning by `run_id`.
- `thread_id` should be carried as metadata because it may not always be present or may map differently across runner integrations, while `session_id` is the most stable ingestion/replay boundary for Hermes.
- `seq` is session-scoped and monotonic so replay consumers can deterministically reconstruct missing slices after reconnect or delayed ingestion.

## Storage Directory Strategy

- The system uses a centralized `<base_dir>` (resolved via environment variable `BERG10_BASE_DIR`).
- Agent runners have dedicated directories under `<base_dir>/runners/<engine_name>/` to isolate their subsystems.
- For Hermes:
  - Low-priority file fallback: `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.jsonl`
  - Engine-specific files: `<base_dir>/runners/hermes/skills/`, `<base_dir>/runners/hermes/config/`, etc.
- For DeerFlow:
  - `<base_dir>/runners/deerflow/` (future runner subsystems).
- For the VFS:
  - `<base_dir>/vfs/checkouts/`, `<base_dir>/vfs/catalog/`, `<base_dir>/vfs/content/` (managed by `berg10-storage`).
- **Schema Routing**: The future ingestor determines how to parse events based on Redpanda topic name (`hermes.l0_drop`, `deerflow.l0_drop`, etc.) and engine/schema headers. File fallback preserves the same engine namespace via `runners/hermes/`.
- Directory structure:
  ```text
  <base_dir>/
  ├── vfs/
  │   ├── checkouts/
  │   ├── catalog/
  │   └── content/
  └── runners/
      ├── hermes/
      │   ├── l0_drop/
      │   │   ├── 2026-04-08/
      │   │   │   └── run_abc123.jsonl     (Dev fallback only)
      │   │   └── 2026-04-09/
      │   │       └── run_def456.jsonl     (Dev fallback only)
      │   ├── skills/
      │   └── config/
      └── deerflow/
          └── (future DeerFlow runner subsystems)
  ```

## Producer Lifecycle (Redpanda Primary)

1.  **Initialize:** A client requests a Hermes run. The runner service resolves or creates `session_id`, `thread_id`, and generates `run_id`.
2.  **Publish Stream:** Each Hermes callback is wrapped in an envelope with stable identifiers and monotonic `seq`.
3.  **Compress (optional):** The raw payload may be gzip-compressed before publish. Compression metadata is included in headers.
4.  **Acknowledge:** Producer waits for Redpanda acknowledgement using reliable delivery settings.
5.  **Fan Out UI Events:** The runner service forwards the same logical envelope to subscribed WebSocket clients via an async broadcast channel.
6.  **Cache for Replay:** The runner stores a bounded recent event window so reconnecting clients can catch up without broker reads when possible.
7.  **Finalize:** The service emits explicit finish events so a future ingestor can close out assistant messages and full runs.
8.  **Failure:** On transient publish failure, producer retries with exponential backoff. On terminal failure, subscribed clients are notified and the run is marked failed.

## Replay and Reconnect Behavior

- Clients reconnecting to `WS /runs/{run_id}/events` provide `replay_from_seq=<n>`.
- If the requested sequence is still available in the runner's in-memory replay buffer, the runner replays from memory first and then resumes live fan-out.
- If the requested sequence is older than the in-memory window, the runner reads the relevant `session_id` partition from Redpanda and replays matching envelopes before resuming live fan-out.
- Replay is runner-mediated so UI clients do not need direct Redpanda access.

## Session Reconstruction Requirements

- Every emitted raw event must carry enough metadata to reconstruct full Hermes request/response history.
- Minimum reconstruction fields:
  - `session_id`
  - `thread_id` when available
  - `run_id`
  - `message_id`
  - `tool_call_id` when applicable
  - `seq` (monotonic within `session_id`)
  - `event_kind`
  - `timestamp`
- Final assistant responses must emit an explicit completion event with:
  - `finish_reason`
  - full assistant text if available
  - final tool linkage state
- Tool progress messages must preserve invocation/result linkage so a future ingestor can rebuild tool call chains without reinterpreting UI events.
- WebSocket-delivered events must preserve the same identifiers and sequencing semantics as Redpanda-published events so reconnect logic can compare and merge them safely.

## Retention and Deletion Semantics

- Redpanda is the primary durable L0 buffer.
- Data must not be deleted before a future ingestor has marked successful ingestion.
- Operationally, retention should be configured long enough to tolerate delayed ingestion runs (target: one month).
- Ingestor is expected to run at least daily in the worst case.
- Exact deletion handshake is deferred to the future ingestor change, but this producer change assumes Redpanda remains the system of record until ingest success is known.

## Verification Strategy

- Run integration tests against an ephemeral Redpanda instance.
- Drive the runner with fixture sessions that exercise plain text streaming, tool calls, tool results, reasoning, and explicit finishes.
- Verify the same event ordering and identifiers across WebSocket delivery and Redpanda consumption.
- Inject broker interruptions to validate retry/backoff behavior and duplicate suppression under idempotent producer settings.
