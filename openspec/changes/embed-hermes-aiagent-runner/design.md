# Design: Embed Hermes In Our ACP Client

## Architecture

The system introduces a Rust-managed integration layer, referred to as `our ACP Client`, that manages Hermes ACP as a child process over JSON-RPC stdio. Its job is to capture every observable boundary event with very low latency, attach minimal replay metadata, expose a live response stream for consumers such as terminal UI, and durably publish the raw protocol traffic to Redb topic `hermes.l0_drop`.

`our ACP Client` is intentionally not an ETL or interpretation layer. It does not assign Berg10 `data_hierarchy`, `data_level`, or `storage_plane` values. It does not normalize ACP payloads into domain records. Those concerns are deferred to future ingestion.

ACP remains the primary control and session boundary for Hermes now and for future non-Hermes agents later. Hermes-specific richer streaming may be added later as a hybrid observability enhancement, but not as a replacement for ACP control semantics.

## Design Goals

- Capture raw Hermes traffic at the ACP boundary without modifying Hermes source.
- Preserve enough raw information to reconstruct a complete session and support future real-time agentic chat UI.
- Keep the first durable write path low-latency and highly durable.
- Own replay sequencing outside Hermes.
- Expose a live stream now, even when current Hermes ACP only provides coarse/final assistant text.
- Allow future scaling from one subprocess per session to multiple sessions per subprocess without redesigning the core model.
- Keep the ACP control/session model reusable for future non-Hermes agents.

## Protocol and Type Strategy

### ACP wire typing

Rust should use the `agent-client-protocol` crate where appropriate for:

- ACP request and response types
- session update parsing
- typed ACP ids such as protocol request ids and tool-call ids
- typed content blocks and MCP server configuration
- typed client-side connection handling over byte streams

Rust should still own:

- subprocess creation and supervision
- stdio wiring into the ACP connection
- Redb publication
- replay sequencing
- runtime registries
- live stream fanout
- application-specific metadata and projections

### Typed naming

Avoid generic names such as `RawObservedEvent` or `UiStreamEvent`. Use context-sensitive, strongly typed runtime names, for example:

- `ChatSessionId`
- `ChatRunId`
- `ChatThreadId`
- `AcpSubprocessId`
- `AcpJsonRpcRequestId`
- `AcpSessionSequenceNumber`
- `AcpCapturedProtocolEvent`
- `AcpResponseStreamEvent`
- `AcpProtocolFrameKind`
- `AcpResponseStreamEventKind`
- `AcpSubprocessLifecycleKind`

The ACP crate's protocol ids and messages should not replace our application/runtime types. Protocol ids are transport-scoped. Runtime ids are session/run/replay-scoped.

## Components

1. **Hermes ACP Subprocess Manager**
   - Launches Hermes ACP as a child process.
   - Uses stdin/stdout for JSON-RPC protocol and stderr for diagnostics.
   - Monitors liveness, startup readiness, crash, exit, and timeout conditions.
   - Default runtime policy is one subprocess per session.
   - The abstraction supports multiple sessions per subprocess so later pooling is possible without changing the client contract.

2. **Our ACP Client Connection Layer**
   - Uses `agent-client-protocol` crate types and connection machinery where appropriate.
   - Encodes client-issued ACP commands.
   - Decodes ACP requests, responses, and notifications from Hermes ACP.
   - Associates request/response correlation and emits transport-observation envelopes for both sides of the boundary.

3. **Chat Session and Run Registry**
   - Tracks long-lived `ChatSessionId` state.
   - Treats each prompt execution as a distinct `ChatRunId` within a chat session.
   - Keeps optional fields such as `ChatThreadId` when available from the source or later user-driven branching.
   - Allows future manual branching semantics to attach `ChatThreadId` without changing the raw contract.

4. **Sequence Allocator**
   - Assigns monotonic `AcpSessionSequenceNumber` per `ChatSessionId`.
   - Owns ordering semantics independently of Hermes internals.
   - Ensures replay consumers can reconstruct session order deterministically from Redb.

5. **Durable Event Publisher**
   - Publishes every `AcpCapturedProtocolEvent` to a broker-agnostic durable event sink.
   - The initial implementation targets Redb stream `hermes.l0_drop`.
   - Uses reliable producer semantics in the concrete broker adapter.
   - Treats the configured broker as the first durable source of truth.

6. **Live Stream Fanout**
   - Projects ACP-observed activity into `AcpResponseStreamEvent` for live consumers.
   - Exposes live status/tool events immediately.
   - Exposes assistant text only at the granularity Hermes ACP currently provides.
   - Emits after durable publication acknowledgement so live consumers see events that are already accepted into the raw log.

7. **Replay Layer**
   - Redb is the durable replay source.
   - A bounded in-memory tail may be kept as a fast-path optimization.
   - `our ACP Client` does not require downstream ingestion to provide replay.

## Process Model

### Default Policy

- One ACP subprocess per session.
- Rationale: stronger fault isolation, simpler cancellation semantics, easier crash attribution, and easier reasoning about session ownership.

### Supported Future Policy

- One ACP subprocess may handle multiple sessions.
- The runtime and registries must not assume a strict 1:1 process-to-session mapping.
- Session lookup, run lookup, and sequencing must be scoped by `ChatSessionId`, not by subprocess identity.

## Subprocess Lifecycle

1. **Spawn**
   - Start `hermes acp` (or equivalent ACP entrypoint) as a child process.
   - Record an `AcpCapturedProtocolEvent` describing spawn intent and process metadata available at launch.

2. **Handshake / Ready**
   - Establish JSON-RPC stdio framing.
   - Perform initialization/authentication/session setup flows required by ACP.
   - Publish the observed request and response frames to Redb.

3. **Session Create / Load / Resume**
   - `our ACP Client` can create a new ACP session or attach to an existing one.
   - These client-issued commands and the corresponding ACP frames are part of the durable raw log.

4. **Run Execution**
   - Each prompt submission becomes a `ChatRunId` within a `ChatSessionId`.
   - The submitted prompt envelope is published before or alongside the observed ACP traffic.
   - All subsequent ACP notifications/results are published in sequence order.
   - Live consumers receive projected `AcpResponseStreamEvent` updates after durable acknowledgement.

5. **Cancel**
   - Client-issued cancel intent is published.
   - ACP cancel request/response/notifications are also published when observed.

6. **Exit / Crash / Restart**
   - Normal exit, abnormal exit, read loop termination, protocol decode failure, timeout, and supervised restart each produce lifecycle envelopes.
   - These lifecycle events are part of the durable session reconstruction story.

## Broker-Agnostic Durable Capture Contract

`our ACP Client` writes to a durable broker first. The initial implementation uses Redb. This is a quick, durable dump with low latency. It is not a place to perform ETL, classification, or Berg10 mapping.

- Initial stream/topic name: `hermes.l0_drop`
- Partition key: `ChatSessionId`
- Ordering authority: client-assigned `AcpSessionSequenceNumber`
- Initial durable replay source: Redb
- Domain interpretation: deferred to ingestion
- Broker binding is implementation-specific and must remain replaceable without changing client-owned runtime event types

## Durable Raw Event Model

The durable raw envelope should be represented in Rust as a strongly typed `AcpCapturedProtocolEvent` that standardizes replay/correlation metadata while preserving raw payloads exactly as observed.

Illustrative fields:

- `chat_session_id: ChatSessionId`
- `chat_run_id: ChatRunId`
- `chat_thread_id: Option<ChatThreadId>`
- `acp_subprocess_id: AcpSubprocessId`
- `acp_session_sequence_number: AcpSessionSequenceNumber`
- `timestamp`
- `acp_protocol_frame_kind: AcpProtocolFrameKind`
- optional protocol correlation ids such as `AcpJsonRpcRequestId`
- optional source ids such as message id and tool-call id when observable
- `raw_json_rpc_payload`

### Envelope principles

- `raw_json_rpc_payload` preserves the source payload without Berg10 interpretation.
- Client-owned metadata is additive and minimal.
- Optional identifiers remain optional when the source does not provide them.
- `our ACP Client` persists every frame it can observe:
  - JSON-RPC requests
  - JSON-RPC responses
  - JSON-RPC notifications
  - client-side command envelopes
  - subprocess lifecycle envelopes

## Live Stream Model

`our ACP Client` also exposes a live response stream using a separate typed projection, `AcpResponseStreamEvent`.

This projection is for immediate consumers such as terminal UI. It is not the durable raw truth.

Illustrative event kinds:

- `RunStarted`
- `AssistantThoughtStatusChunk`
- `ToolCallStarted`
- `ToolCallCompleted`
- `AssistantTextFinal`
- `RunCompleted`
- `RunCancelled`
- `StreamError`

### Current Hermes ACP fidelity

Current Hermes ACP behavior supports:

- live tool activity
- live coarse thought/status updates
- final/coarse assistant text

Current Hermes ACP does not support true incremental assistant-text deltas.

Therefore phase 1 live stream behavior is intentionally honest:

- do expose live stream now
- do not fabricate token or delta streaming from a final text blob
- surface assistant text only at the granularity ACP actually emits

## Deferred Incremental Assistant Text

True incremental assistant-text streaming is deferred to a future hybrid Hermes-specific stream adapter.

Candidate sources include Hermes-specific observability surfaces such as a TUI gateway. That future hybrid adapter may enrich `AcpResponseStreamEvent` with variants such as:

- `AssistantTextDeltaChunk`
- `AssistantReasoningDeltaChunk`

This does not replace ACP. ACP remains the primary control and session boundary.

## Event Kinds

The durable raw layer may use coarse transport-observation kinds to help replay and diagnostics, without performing domain interpretation. Example kinds:

- `client.control_intent`
- `subprocess.lifecycle.spawned`
- `subprocess.lifecycle.exited`
- `subprocess.lifecycle.crashed`
- `subprocess.lifecycle.restarted`
- `acp.client_request`
- `acp.server_response`
- `acp.server_notification`
- `client.error`

These are transport-observation kinds only. They are not Berg10 `payload_kind` or semantic classification.

## Replay Model

- Replay is keyed by `ChatSessionId` and ordered by `AcpSessionSequenceNumber`.
- Redb is the durable replay source.
- A bounded in-memory replay tail may be used to accelerate recent reconnects, but it is an optimization only.
- Session reconstruction should not depend on local memory surviving process restarts.

## Session, Run, and Thread Semantics

- `ChatSessionId`
  - Primary durable conversational boundary.
  - Redb partition key.
  - Sequence scope.

- `ChatRunId`
  - Represents one prompt execution inside a chat session.
  - Assigned and managed by `our ACP Client`.
  - Multiple runs may exist over the lifetime of one session.

- `ChatThreadId`
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

- `BERG10_BASE_DIR` remains the base path for client-owned runtime files.
- Hermes-owned runtime data, diagnostics, and any future local state live under `<base_dir>/runners/hermes/`.
- This change does not define Berg10 ingestion paths because client output lands in Redb, not directly in `crates/berg10/storage-vfs/`.

## Deferred Concerns

The following are explicitly deferred to future changes:

- Berg10 `data_hierarchy`, `data_level`, and `storage_plane` mapping
- semantic classification of ACP/raw envelopes
- normalization and projection into catalog/query models
- Hermes-specific hybrid stream adapters for richer text deltas
- public HTTP/WebSocket APIs
- deletion handshake after successful ingestion
- file fallback as a primary or recommended path

## Verification Strategy

- Run integration tests against an ephemeral Redb instance.
- Launch Hermes ACP as a subprocess in test harnesses where possible.
- Exercise:
  - session create
  - session load/resume
  - prompt/run execution
  - cancellation
  - subprocess crash/restart paths
- Verify every observable boundary frame is published to `hermes.l0_drop`.
- Verify `ChatSessionId` partitioning and monotonic `AcpSessionSequenceNumber` ordering.
- Verify replay from Redb reconstructs a complete session log without requiring client-local memory.
- Verify live stream fanout surfaces tool/status events immediately and assistant text only at current Hermes ACP granularity.
- Verify the ACP control/session model remains reusable for future non-Hermes agents.
