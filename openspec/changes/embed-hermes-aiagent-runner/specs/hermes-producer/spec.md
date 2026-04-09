## ADDED Requirements

### Requirement: Hermes raw traffic SHALL be published to Redpanda before ingestion
Hermes producer flows SHALL publish raw, opaque Hermes session traffic to a Redpanda topic named `hermes.l0_drop` before any schema-on-read ingestion occurs.

#### Scenario: Publish raw Hermes callback event
- **WHEN** Hermes emits a callback event during a run
- **THEN** the bridge publishes that event to topic `hermes.l0_drop`
- **AND** the published payload remains raw and unmapped

### Requirement: Topic naming SHALL follow `<engine>.l0_drop`
Every runner engine SHALL publish raw L0 buffer traffic to a topic named `<engine>.l0_drop`.

#### Scenario: Hermes topic naming
- **WHEN** the Hermes runner is active
- **THEN** it publishes to `hermes.l0_drop`

### Requirement: Producer delivery SHALL be reliable
The Hermes producer SHALL use reliable Redpanda/Kafka producer settings so raw traffic is not dropped silently.

#### Scenario: Producer waits for durable acknowledgement
- **WHEN** Hermes publishes a raw event
- **THEN** the producer uses `acks=all`
- **AND** idempotent producer mode is enabled
- **AND** transient failures trigger retries with backoff
- **AND** the producer config preserves ordering during retries

### Requirement: Raw events SHALL preserve full session reconstruction metadata
Each published raw event SHALL contain enough metadata to reconstruct the full Hermes session/thread conversation.

#### Scenario: Session replay data is preserved
- **WHEN** a user prompt leads to assistant replies, tool calls, and tool results
- **THEN** the raw event stream includes stable `session_id`, `thread_id` when available, `run_id`, `message_id`, `tool_call_id`, `seq`, `event_kind`, and `timestamp`
- **AND** final assistant completion events include `finish_reason`

### Requirement: Session ordering SHALL be preserved by partition key
Hermes raw events SHALL use `session_id` as the Redpanda message key so a full session can be replayed in order.

#### Scenario: Multiple turns share a session
- **WHEN** several Hermes runs belong to the same session
- **THEN** they are published with the same `session_id` key
- **AND** consumers can reconstruct the session in order

### Requirement: Payload compression MAY be applied
Hermes raw payloads MAY be gzip-compressed before publish to reduce Redpanda storage usage.

#### Scenario: Compressed raw payload
- **WHEN** gzip compression is enabled
- **THEN** the message headers identify `content_encoding=gzip`
- **AND** the raw message body remains otherwise opaque until ingestion

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

### Requirement: File fallback SHALL remain documented as secondary
The system SHALL document a file-based fallback under `<base_dir>/runners/hermes/l0_drop/`, but it SHALL be treated as lower priority than Redpanda.

#### Scenario: Development without Redpanda
- **WHEN** Redpanda is unavailable in a low-priority dev environment
- **THEN** raw events may be written to `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.jsonl`
- **AND** this path is considered a secondary fallback, not the primary architecture

### Requirement: UI integration SHALL use a backend-compatible transport
Hermes runner integration SHALL be exposed through a backend API rather than a desktop-only stdin/stdout subprocess contract.

#### Scenario: Desktop and web clients share the same runner API
- **WHEN** a desktop client or a web/WASM client needs Hermes interaction
- **THEN** both use the same backend command and stream API
- **AND** Hermes execution is not coupled to a UI-owned Python subprocess

### Requirement: The runner API SHALL expose stable run control endpoints
Hermes runner integration SHALL expose a stable HTTP plus WebSocket API for run lifecycle and event streaming.

#### Scenario: Start and cancel a run
- **WHEN** a client starts a Hermes interaction
- **THEN** it can create a run via `POST /runs`
- **AND** it can cancel an active run via `POST /runs/{run_id}/cancel`

#### Scenario: Subscribe to live events
- **WHEN** a client needs streaming Hermes updates
- **THEN** it subscribes via `WS /runs/{run_id}/events`
- **AND** the stream delivers live event envelopes for deltas, tool activity, reasoning, and finish events

### Requirement: UI stream envelopes SHALL match the durable event contract
The runner SHALL deliver live UI events using the same logical envelope fields that are published to Redpanda.

#### Scenario: UI and Redpanda event parity
- **WHEN** a Hermes event is delivered to a WebSocket client and published to Redpanda
- **THEN** both representations include the same `engine`, `schema_version`, `session_id`, `run_id`, `seq`, `event_kind`, and `timestamp` values
- **AND** clients do not need a separate UI-only event schema to track ordering or replay state

### Requirement: Reconnecting clients SHALL be able to request replay by sequence
The Hermes runner SHALL support reconnect and replay without requiring UI clients to read Redpanda directly.

#### Scenario: Replay from in-memory cache
- **WHEN** a client reconnects and requests `replay_from_seq` for a recently emitted event
- **THEN** the runner replays the missing envelopes from its local replay buffer
- **AND** resumes live streaming afterward

#### Scenario: Replay from Redpanda
- **WHEN** a client reconnects and requests `replay_from_seq` older than the runner's local replay buffer
- **THEN** the runner replays the missing envelopes by reading Redpanda for the matching `session_id`
- **AND** the client does not require direct broker access
