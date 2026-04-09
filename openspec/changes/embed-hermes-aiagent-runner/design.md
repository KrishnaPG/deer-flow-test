# Design: Embed Hermes AIAgent Runner

## Architecture

The system introduces a new Python process (`hermes_bridge.py`) spawned by the Rust `deer_gui` frontend. It operates asynchronously, managing the `AIAgent` and pushing data to two disparate sinks.

### Components

1.  **`deer_gui` Rust App (`apps/deer_gui/src/bridge/adapter.rs`)**
    - Spawns `hermes_bridge.py` instead of the legacy `bridge.py` based on an environment variable or config toggle.
    - Communicates over `stdin`/`stdout` using JSON lines.

2.  **`hermes_bridge.py` (Python Adapter)**
    - Reads commands from `stdin` (`send_message`, `list_threads`, etc.).
    - Translates thread/message concepts into Hermes-compatible execution runs.
    - Registers callbacks on the `AIAgent` (`stream_delta_callback`, `tool_progress_callback`).
    
3.  **Sink A: GUI Output (Real-time)**
    - Maps Hermes text deltas and tool states into the `MessageRecord` format used by `deer_gui`.
    - Emits `{"kind": "event", "event": "message", ...}` payloads to `stdout`.

4.  **Sink B: Redpanda L0 Buffer (Primary Durable Staging)**
    - Captures the *raw, unmodified* event payloads directly from the Hermes callbacks.
    - Publishes them to Redpanda topic `hermes.l0_drop`.
    - Uses reliable producer semantics: `acks=all`, idempotence enabled, retry/backoff.
    - Keeps the payload opaque; schema-on-read happens later during ingestion.

5.  **Dev Fallback: File-Based L0 Drop (Low Priority)**
    - For lower-priority dev environments without Redpanda, the same raw events may be appended to `<base_dir>/runners/hermes/l0_drop/YYYY-MM-DD/run_<uuid>.jsonl`.
    - This fallback is documented but not the current implementation priority.

## Data Model (Raw Event Shape)

The primary Redpanda payloads contain a pure dump of what the Hermes engine produces. We do not impose canonical schemas at publish time.

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
  "seq": 18,
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

### Partitioning

- Use `session_id` as the Kafka/Redpanda message key.
- Rationale: Hermes sessions span multiple runs/turns; partitioning by `session_id` preserves long-lived conversational ordering and makes full-thread reconstruction easier than partitioning by `run_id`.
- `thread_id` should be carried as metadata because it may not always be present or may map differently across runner integrations, while `session_id` is the most stable ingestion/replay boundary for Hermes.

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

1.  **Initialize:** User sends a prompt. Bridge resolves or creates `session_id`, `thread_id`, and generates `run_id`.
2.  **Publish Stream:** Each Hermes callback is wrapped in an envelope with stable identifiers and monotonic `seq`.
3.  **Compress (optional):** The raw payload may be gzip-compressed before publish. Compression metadata is included in headers.
4.  **Acknowledge:** Producer waits for Redpanda acknowledgement using reliable delivery settings.
5.  **Finalize:** Bridge emits explicit finish events so a future ingestor can close out assistant messages and full runs.
6.  **Failure:** On transient publish failure, producer retries with exponential backoff. On terminal failure, the UI is notified and the run is marked failed.

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

## Retention and Deletion Semantics

- Redpanda is the primary durable L0 buffer.
- Data must not be deleted before a future ingestor has marked successful ingestion.
- Operationally, retention should be configured long enough to tolerate delayed ingestion runs (target: one month).
- Ingestor is expected to run at least daily in the worst case.
- Exact deletion handshake is deferred to the future ingestor change, but this producer change assumes Redpanda remains the system of record until ingest success is known.
