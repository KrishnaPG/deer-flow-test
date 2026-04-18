# Design: Embed Hermes ACP Producer

## Architecture

The system introduces a Rust producer that manages Hermes ACP as a child process over JSON-RPC stdio. Its job is to capture every observable boundary event with very low latency, attach minimal replay metadata, and durably publish the raw traffic to Redpanda topic `hermes.l0_drop`.

This producer is intentionally not an ETL or interpretation layer. It does not assign Berg10 `data_hierarchy`, `data_level`, or `storage_plane` values. It does not normalize ACP payloads into domain records. Those concerns are deferred to future ingestion.

## Design Goals

- Capture raw Hermes traffic at the ACP boundary without modifying Hermes source.
- Preserve enough raw information to reconstruct a complete session and support future real-time agentic chat UI.
- Keep the first durable write path low-latency and highly durable.
- Own replay sequencing outside Hermes.
- Allow future scaling from one subprocess per session to multiple sessions per subprocess without redesigning the core model.

## Components

1. **Hermes ACP Subprocess Manager**
   - Launches Hermes ACP as a child process.
   - Uses stdin/stdout for JSON-RPC protocol and stderr for diagnostics.
   - Monitors liveness, startup readiness, crash, exit, and timeout conditions.
   - Default runtime policy is one subprocess per session.
   - The abstraction supports multiple sessions per subprocess so later pooling is possible without changing the producer contract.

2. **ACP JSON-RPC Client**
   - Encodes producer-issued commands into JSON-RPC frames.
   - Decodes JSON-RPC requests, responses, and notifications from Hermes ACP.
   - Associates request/response correlation and emits envelopes for both sides of the boundary.

3. **Session and Run Registry**
   - Tracks long-lived `session_id` state.
   - Treats each prompt execution as a distinct `run_id` within a session.
   - Keeps optional fields such as `thread_id` when available from the source or later user-driven branching.
   - Allows future manual branching semantics to attach `thread_id` without changing the raw contract.

4. **Sequence Allocator**
   - Assigns monotonic `seq` per `session_id`.
   - Owns ordering semantics independently of Hermes internals.
   - Ensures replay consumers can reconstruct session order deterministically from Redpanda.

5. **Redpanda Publisher**
   - Publishes every raw envelope to `hermes.l0_drop`.
   - Uses reliable producer semantics with `rust-rdkafka`.
   - Treats Redpanda as the first durable source of truth.

6. **Replay Layer**
   - Redpanda is the durable replay source.
   - A bounded in-memory tail may be kept as a fast-path optimization.
   - The producer does not require downstream ingestion to provide replay.

## Process Model

### Default Policy

- One ACP subprocess per session.
- Rationale: stronger fault isolation, simpler cancellation semantics, easier crash attribution, and easier reasoning about session ownership.

### Supported Future Policy

- One ACP subprocess may handle multiple sessions.
- The producer runtime and registries must not assume a strict 1:1 process-to-session mapping.
- Session lookup, run lookup, and sequencing must be scoped by `session_id`, not by subprocess identity.

## Subprocess Lifecycle

1. **Spawn**
   - Start `hermes acp` (or equivalent ACP entrypoint) as a child process.
   - Record a lifecycle envelope describing spawn intent and process metadata available at launch.

2. **Handshake / Ready**
   - Establish JSON-RPC stdio framing.
   - Perform initialization/authentication/session setup flows required by ACP.
   - Publish the observed request and response frames to Redpanda.

3. **Session Create / Load / Resume**
   - The producer can create a new ACP session or attach to an existing one.
   - These producer-issued commands and the corresponding ACP frames are part of the durable raw log.

4. **Run Execution**
   - Each prompt submission becomes a `run_id` within a `session_id`.
   - The submitted prompt envelope is published before or alongside the observed ACP traffic.
   - All subsequent ACP notifications/results are published in sequence order.

5. **Cancel**
   - Producer-issued cancel intent is published.
   - ACP cancel request/response/notifications are also published when observed.

6. **Exit / Crash / Restart**
   - Normal exit, abnormal exit, read loop termination, protocol decode failure, timeout, and supervised restart each produce lifecycle envelopes.
   - These lifecycle events are part of the durable session reconstruction story.

## Redpanda-First Raw Capture Contract

The producer writes to Redpanda first. This is a quick, durable dump with low latency. It is not a place to perform ETL, classification, or Berg10 mapping.

- Topic name: `hermes.l0_drop`
- Message key: `session_id`
- Ordering authority: producer-assigned `seq`
- Durable replay source: Redpanda
- Domain interpretation: deferred to ingestion

## Raw Envelope Shape

The producer envelope standardizes replay/correlation metadata while preserving raw payloads exactly as observed.

Example envelope:

```json
{
  "engine": "hermes",
  "schema_version": "v1",
  "session_id": "session-1234",
  "run_id": "run-42",
  "seq": 18,
  "timestamp": "2026-04-18T14:22:10Z",
  "thread_id": null,
  "message_id": null,
  "tool_call_id": null,
  "event_kind": "acp.notification",
  "source_kind": "acp",
  "source_direction": "child_to_producer",
  "raw_payload": {
    "jsonrpc": "2.0",
    "method": "session/update",
    "params": {
      "sessionId": "session-1234",
      "update": {"kind": "message.delta", "text": "hello"}
    }
  }
}
```

### Envelope Principles

- `raw_payload` preserves the source payload without Berg10 interpretation.
- Producer metadata is additive and minimal.
- Optional identifiers remain optional when the source does not provide them.
- The producer persists every frame it can observe:
  - JSON-RPC requests
  - JSON-RPC responses
  - JSON-RPC notifications
  - producer-side command envelopes
  - subprocess lifecycle envelopes

## Event Kinds

The producer may use coarse event kinds to help replay and diagnostics, without performing domain interpretation. Example kinds:

- `producer.command`
- `producer.lifecycle.spawned`
- `producer.lifecycle.exited`
- `producer.lifecycle.crashed`
- `producer.lifecycle.restarted`
- `acp.request`
- `acp.response`
- `acp.notification`
- `producer.error`

These are transport-observation kinds only. They are not Berg10 `payload_kind` or semantic classification.

## Replay Model

- Replay is keyed by `session_id` and ordered by `seq`.
- Redpanda is the durable replay source.
- A bounded in-memory replay tail may be used to accelerate recent reconnects, but it is an optimization only.
- Session reconstruction should not depend on local memory surviving process restarts.

## Session, Run, and Thread Semantics

- `session_id`
  - Primary durable conversational boundary.
  - Redpanda partition key.
  - Sequence scope.

- `run_id`
  - Represents one prompt execution inside a session.
  - Assigned/managed by the producer runtime.
  - Multiple runs may exist over the lifetime of one session.

- `thread_id`
  - Optional.
  - Preserved when a real source exists.
  - Remains available for future manual branching initiated by UI or higher-level orchestration.

## Reliable Producer Configuration

### Required configuration

- `acks=all`
- `enable.idempotence=true`
- ordering-safe in-flight request configuration
- retries with backoff
- surfaced terminal failure status

### Optional configuration

- payload compression when operationally useful
- bounded local replay tail
- subprocess pooling policy for multi-session processes

## Storage and Directory Strategy

- `BERG10_BASE_DIR` remains the base path for producer-owned runtime files.
- Hermes-owned runtime data, diagnostics, and any future local state live under `<base_dir>/runners/hermes/`.
- This change does not define Berg10 ingestion paths because producer output lands in Redpanda, not directly in `crates/berg10/storage-vfs/`.

## Deferred Concerns

The following are explicitly deferred to future changes:

- Berg10 `data_hierarchy`, `data_level`, and `storage_plane` mapping
- semantic classification of ACP/raw envelopes
- normalization and projection into catalog/query models
- public UI transport/API contracts
- deletion handshake after successful ingestion
- file fallback as a primary or recommended path

## Verification Strategy

- Run integration tests against an ephemeral Redpanda instance.
- Launch Hermes ACP as a subprocess in test harnesses where possible.
- Exercise:
  - session create
  - session load/resume
  - prompt/run execution
  - cancellation
  - subprocess crash/restart paths
- Verify every observable boundary frame is published to `hermes.l0_drop`.
- Verify `session_id` partitioning and monotonic `seq` ordering.
- Verify replay from Redpanda reconstructs a complete session log without requiring producer-local memory.
