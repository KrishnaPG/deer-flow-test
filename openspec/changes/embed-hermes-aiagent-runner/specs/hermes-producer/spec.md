## ADDED Requirements

### Requirement: Our ACP Client SHALL use ACP over JSON-RPC stdio
Our ACP Client SHALL integrate with Hermes through ACP JSON-RPC over stdio by managing Hermes as a subprocess.

#### Scenario: Launch ACP subprocess
- **WHEN** our ACP Client starts a new Hermes worker
- **THEN** it launches Hermes ACP as a child process
- **AND** it communicates over JSON-RPC using stdin/stdout
- **AND** stdout is treated as protocol output rather than free-form logs

### Requirement: Our ACP Client SHALL use Rust ACP protocol typing where appropriate
Our ACP Client SHALL use the Rust `agent-client-protocol` crate where appropriate for ACP wire/protocol typing, while retaining client-owned runtime types for replay, sequencing, and live stream projection.

#### Scenario: ACP wire types and local runtime types coexist
- **WHEN** our ACP Client implements ACP transport and session handling
- **THEN** ACP requests, responses, and updates use crate-provided protocol typing where appropriate
- **AND** client-owned runtime types remain separate for concepts such as `ChatSessionId`, `ChatRunId`, `AcpCapturedProtocolEvent`, and `AcpResponseStreamEvent`

### Requirement: Our ACP Client SHALL NOT modify 3rdParty Hermes source
The Hermes integration SHALL be implemented outside `3rdParty/hermes-agent` and SHALL NOT require source changes under `3rdParty/`.

#### Scenario: Integrate Hermes without source edits
- **WHEN** our ACP Client is implemented
- **THEN** Hermes is treated as a read-only dependency
- **AND** our ACP Client captures traffic only at the ACP subprocess boundary

### Requirement: Our ACP Client SHALL use context-sensitive strong runtime types
Our ACP Client SHALL use context-sensitive strong runtime types for its internal ids and events rather than generic names.

#### Scenario: Runtime ids and events are strongly typed
- **WHEN** runtime ids and event structs are introduced in Rust
- **THEN** they use context-sensitive names such as `ChatSessionId`, `ChatRunId`, `ChatThreadId`, `AcpCapturedProtocolEvent`, and `AcpResponseStreamEvent`
- **AND** generic names such as `RawObservedEvent`, `UiStreamEvent`, or plain `SessionId` are not used for client-owned runtime concepts

### Requirement: Hermes raw traffic SHALL be published to Redpanda before ingestion
Our ACP Client flows SHALL publish raw Hermes ACP traffic to Redpanda topic `hermes.l0_drop` before any ingestion, Berg10 mapping, or domain interpretation occurs.

#### Scenario: Publish raw ACP traffic
- **WHEN** our ACP Client observes an ACP frame or emits a client-side command envelope
- **THEN** it publishes that event to `hermes.l0_drop`
- **AND** the payload remains raw and unmapped

### Requirement: Durable buffer semantics SHALL remain Redpanda-first and low-latency
Our ACP Client SHALL treat Redpanda as the primary durable L0 landing zone for low-latency raw capture rather than as an ETL layer.

#### Scenario: Client writes first durable copy
- **WHEN** our ACP Client captures Hermes boundary traffic
- **THEN** the first durable write is to Redpanda
- **AND** the client does not block on Berg10 ingestion or semantic transformation

### Requirement: Every observable boundary event SHALL be preserved
Our ACP Client SHALL preserve every boundary event it can observe so a future consumer can reconstruct the whole session.

#### Scenario: Preserve command and protocol traffic
- **WHEN** our ACP Client issues session or run commands such as create, load, resume, prompt, or cancel
- **THEN** those client-side command envelopes are durably published
- **AND** observed ACP requests, responses, and notifications are also durably published

#### Scenario: Preserve subprocess lifecycle events
- **WHEN** Hermes ACP starts, exits, crashes, times out, or restarts
- **THEN** our ACP Client emits lifecycle envelopes that are durably published

### Requirement: Raw events SHALL preserve full session reconstruction metadata
Each published raw event SHALL contain enough metadata to reconstruct the full Hermes session and run history from the client boundary.

#### Scenario: Session replay data is preserved
- **WHEN** a session includes prompts, assistant updates, tool activity, cancellations, or failures
- **THEN** the raw event stream includes stable `session_id`, `run_id`, `seq`, and `timestamp`
- **AND** optional identifiers such as `thread_id`, `message_id`, and `tool_call_id` are included when observable

### Requirement: Session ordering SHALL be preserved by partition key
Hermes raw events SHALL use `session_id` as the Redpanda message key so a full session can be replayed in order.

#### Scenario: Multiple runs share a session
- **WHEN** several prompt executions belong to the same session
- **THEN** they are published with the same `session_id` key
- **AND** consumers can reconstruct the session in order using `seq`

### Requirement: Sequence assignment SHALL be owned by our ACP Client
Our ACP Client SHALL assign monotonic `seq` values within a session instead of relying on Hermes internals for replay ordering.

#### Scenario: Client controls ordering
- **WHEN** our ACP Client publishes envelopes for one session
- **THEN** each envelope gets a session-scoped monotonic `seq`
- **AND** replay consumers can reconstruct a deterministic order from Redpanda alone

### Requirement: Multiple sessions per subprocess SHALL be supported by design
Our ACP Client design SHALL allow one ACP subprocess to serve multiple sessions, while the default runtime policy may still use one subprocess per session.

#### Scenario: Default isolation policy
- **WHEN** our ACP Client runs with default settings
- **THEN** it may launch one subprocess per session
- **AND** its registries and sequencing logic do not assume this is the only possible topology

### Requirement: Payload compression MAY be applied
Hermes raw payloads MAY be compressed before publish to reduce Redpanda storage usage.

#### Scenario: Compressed raw payload
- **WHEN** compression is enabled
- **THEN** the message headers identify the content encoding
- **AND** the raw message body remains otherwise opaque until ingestion

### Requirement: ACP Client delivery SHALL be reliable
Our ACP Client SHALL use reliable Redpanda/Kafka producer settings so raw traffic is not dropped silently.

#### Scenario: Client waits for durable acknowledgement
- **WHEN** our ACP Client publishes a raw event
- **THEN** it uses `acks=all`
- **AND** idempotent producer mode is enabled
- **AND** transient failures trigger retries with backoff
- **AND** the client config preserves ordering during retries

### Requirement: Our ACP Client SHALL expose a live response stream now
Our ACP Client SHALL expose a live response stream derived from ACP-observed activity even before a richer Hermes-specific hybrid stream adapter exists.

#### Scenario: Live tool and status stream
- **WHEN** a run is active
- **THEN** live subprocess, run, thought/status, and tool events are surfaced as they are observed

### Requirement: Our ACP Client SHALL preserve current Hermes ACP text fidelity
Our ACP Client SHALL preserve the assistant-text fidelity Hermes ACP currently exposes and SHALL NOT invent finer-grained assistant text deltas.

#### Scenario: Hermes ACP emits only coarse or final assistant text
- **WHEN** Hermes ACP exposes assistant text only as coarse or final updates
- **THEN** our ACP Client emits assistant text only at that same coarse or final granularity
- **AND** it does not fabricate token or delta streaming from final text

### Requirement: True incremental assistant text SHALL be deferred to a future hybrid Hermes-specific stream adapter
True incremental assistant text is outside the current Hermes ACP fidelity and SHALL be deferred to a future hybrid Hermes-specific stream adapter.

#### Scenario: Future richer Hermes stream source
- **WHEN** a future Hermes-specific source such as a TUI gateway is integrated
- **THEN** it may enrich the live stream with true incremental assistant text deltas
- **AND** ACP remains the primary control and session boundary

### Requirement: ACP SHALL remain the primary control and session boundary
ACP SHALL remain the primary control and session boundary for Hermes now and for future non-Hermes agents later.

#### Scenario: Future non-Hermes agent support
- **WHEN** a future agent other than Hermes is integrated
- **THEN** the same ACP control/session model remains reusable
- **AND** richer engine-specific stream adapters remain optional enhancements rather than replacements for ACP control semantics

### Requirement: Redpanda retention SHALL outlive delayed ingestion
Raw Hermes data SHALL remain in Redpanda long enough to tolerate delayed ingestion runs.

#### Scenario: Ingestor delayed for days
- **WHEN** the ingestor has not yet processed a message
- **THEN** the message remains in Redpanda and is not deleted due to short retention

#### Scenario: Operational retention target
- **WHEN** retention is configured for `hermes.l0_drop`
- **THEN** the target retention window is approximately one month
- **AND** operators run the ingestor at least daily in the worst case

### Requirement: Successful ingestion SHALL control deletion timing
Hermes raw data SHALL NOT be treated as disposable before ingestion success is known.

#### Scenario: Message ingested successfully
- **WHEN** a future ingestor has durably completed ingestion for a raw Hermes message
- **THEN** the system may allow Redpanda cleanup according to the configured post-ingestion retention policy
- **BUT** deletion semantics are controlled by ingestion success, not by a short default retention window

### Requirement: Berg10 mapping SHALL be deferred to ingestion
Our ACP Client SHALL NOT assign Berg10 semantic fields or perform domain interpretation before publish.

#### Scenario: Client captures raw envelope only
- **WHEN** our ACP Client publishes a Hermes envelope
- **THEN** it does not classify the record into Berg10 `data_hierarchy`, `data_level`, or `storage_plane`
- **AND** those mappings are deferred to future ingestion work

### Requirement: Replay SHALL be reconstructible from Redpanda
Our ACP Client SHALL allow a future consumer to reconstruct a session from Redpanda without depending on client-local memory.

#### Scenario: Replay after client restart
- **WHEN** our ACP Client restarts after publishing part of a session
- **THEN** a replay consumer can still reconstruct the session from Redpanda using `session_id` and `seq`

### Requirement: Optional identifiers SHALL remain optional until a source exists
Our ACP Client SHALL preserve optional fields such as `thread_id` when available, but SHALL NOT invent a required source-backed value when no source exists.

#### Scenario: Future manual branching
- **WHEN** a higher-level UI or orchestration flow later introduces manual session branching
- **THEN** `thread_id` may be attached to raw envelopes for those branches
- **AND** current client flows remain valid when `thread_id` is absent
