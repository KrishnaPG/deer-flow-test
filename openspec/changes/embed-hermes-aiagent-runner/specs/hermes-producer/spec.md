## ADDED Requirements

### Requirement: Hermes producer SHALL use ACP over JSON-RPC stdio
The Hermes producer SHALL integrate with Hermes through ACP JSON-RPC over stdio by managing Hermes as a subprocess.

#### Scenario: Launch ACP subprocess
- **WHEN** the Hermes producer starts a new Hermes worker
- **THEN** it launches Hermes ACP as a child process
- **AND** it communicates over JSON-RPC using stdin/stdout
- **AND** stdout is treated as protocol output rather than free-form logs

### Requirement: Hermes producer SHALL NOT modify 3rdParty Hermes source
The Hermes integration SHALL be implemented outside `3rdParty/hermes-agent` and SHALL NOT require source changes under `3rdParty/`.

#### Scenario: Integrate Hermes without source edits
- **WHEN** the producer is implemented
- **THEN** Hermes is treated as a read-only dependency
- **AND** the producer captures traffic only at the ACP subprocess boundary

### Requirement: Hermes raw traffic SHALL be published to Redpanda before ingestion
Hermes producer flows SHALL publish raw Hermes ACP traffic to Redpanda topic `hermes.l0_drop` before any ingestion, Berg10 mapping, or domain interpretation occurs.

#### Scenario: Publish raw ACP traffic
- **WHEN** the producer observes an ACP frame or emits a producer-side command envelope
- **THEN** it publishes that event to `hermes.l0_drop`
- **AND** the payload remains raw and unmapped

### Requirement: Durable buffer semantics SHALL remain Redpanda-first and low-latency
The Hermes producer SHALL treat Redpanda as the primary durable L0 landing zone for low-latency raw capture rather than as an ETL layer.

#### Scenario: Producer writes first durable copy
- **WHEN** the producer captures Hermes boundary traffic
- **THEN** the first durable write is to Redpanda
- **AND** the producer does not block on Berg10 ingestion or semantic transformation

### Requirement: Every observable boundary event SHALL be preserved
The Hermes producer SHALL preserve every boundary event it can observe so a future consumer can reconstruct the whole session.

#### Scenario: Preserve command and protocol traffic
- **WHEN** the producer issues session or run commands such as create, load, resume, prompt, or cancel
- **THEN** those producer-side command envelopes are durably published
- **AND** observed ACP requests, responses, and notifications are also durably published

#### Scenario: Preserve subprocess lifecycle events
- **WHEN** Hermes ACP starts, exits, crashes, times out, or restarts
- **THEN** the producer emits lifecycle envelopes that are durably published

### Requirement: Raw events SHALL preserve full session reconstruction metadata
Each published raw event SHALL contain enough metadata to reconstruct the full Hermes session and run history from the producer boundary.

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

### Requirement: Sequence assignment SHALL be owned by the producer
The Hermes producer SHALL assign monotonic `seq` values within a session instead of relying on Hermes internals for replay ordering.

#### Scenario: Producer controls ordering
- **WHEN** the producer publishes envelopes for one session
- **THEN** each envelope gets a session-scoped monotonic `seq`
- **AND** replay consumers can reconstruct a deterministic order from Redpanda alone

### Requirement: Multiple sessions per subprocess SHALL be supported by design
The Hermes producer design SHALL allow one ACP subprocess to serve multiple sessions, while the default runtime policy may still use one subprocess per session.

#### Scenario: Default isolation policy
- **WHEN** the producer runs with default settings
- **THEN** it may launch one subprocess per session
- **AND** its registries and sequencing logic do not assume this is the only possible topology

### Requirement: Payload compression MAY be applied
Hermes raw payloads MAY be compressed before publish to reduce Redpanda storage usage.

#### Scenario: Compressed raw payload
- **WHEN** compression is enabled
- **THEN** the message headers identify the content encoding
- **AND** the raw message body remains otherwise opaque until ingestion

### Requirement: Producer delivery SHALL be reliable
The Hermes producer SHALL use reliable Redpanda/Kafka producer settings so raw traffic is not dropped silently.

#### Scenario: Producer waits for durable acknowledgement
- **WHEN** Hermes publishes a raw event
- **THEN** the producer uses `acks=all`
- **AND** idempotent producer mode is enabled
- **AND** transient failures trigger retries with backoff
- **AND** the producer config preserves ordering during retries

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
The Hermes producer SHALL NOT assign Berg10 semantic fields or perform domain interpretation before publish.

#### Scenario: Producer captures raw envelope only
- **WHEN** the producer publishes a Hermes envelope
- **THEN** it does not classify the record into Berg10 `data_hierarchy`, `data_level`, or `storage_plane`
- **AND** those mappings are deferred to future ingestion work

### Requirement: Replay SHALL be reconstructible from Redpanda
The Hermes producer SHALL allow a future consumer to reconstruct a session from Redpanda without depending on producer-local memory.

#### Scenario: Replay after producer restart
- **WHEN** the producer restarts after publishing part of a session
- **THEN** a replay consumer can still reconstruct the session from Redpanda using `session_id` and `seq`

### Requirement: Optional identifiers SHALL remain optional until a source exists
The producer SHALL preserve optional fields such as `thread_id` when available, but SHALL NOT invent a required source-backed value when no source exists.

#### Scenario: Future manual branching
- **WHEN** a higher-level UI or orchestration flow later introduces manual session branching
- **THEN** `thread_id` may be attached to raw envelopes for those branches
- **AND** current producer flows remain valid when `thread_id` is absent
