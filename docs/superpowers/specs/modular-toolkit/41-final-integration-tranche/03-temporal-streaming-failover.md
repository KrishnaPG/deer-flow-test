## Temporal, Streaming, And Failover Semantics

### `replay_cursor`

Purpose:

- identify the singular active temporal inspection point shared across replay-
  capable surfaces

Required basis:

- `sequence_id`, `event_time`, or `checkpoint_id`
- source stream or canonical record family
- broker epoch
- origin panel and emission sequence
- resolution mode: `live_tail`, `checkpoint`, or `historical_event`

### `temporal_range`

Purpose:

- express a bounded time window for brushing, filtering, compare alignment, or
  replay scope

Required basis:

- lower and upper temporal anchors
- anchor kind
- inclusivity semantics
- granularity
- open-endedness where relevant

### Stream Lifecycle

Every live linked stream must expose one of:

- `connecting`
- `live`
- `catching_up`
- `degraded`
- `stalled`
- `recovered`
- `closed`

Rules:

- the shell must show whether a surface is at live tail or viewing historical
  state
- state transitions into degraded or recovered conditions must be operator-
  visible
- linked interaction must not imply freshness while replaying backlog

### Freshness And Staleness

Required metadata:

- freshness timestamp or last applied `sequence_id`
- staleness budget by shell mode
- supersession semantics for pins, compare members, and focus targets
- explicit stale reason

Rule:

- stale linked state may remain visible only when marked explicitly

### Operational Degradation

The shell must define operator-visible degradation behavior for:

- ingest lag
- backpressure
- partial stream loss
- unavailable enrichments
- storage slowdown with event-bus continuity

### Broker Recovery

Every shell mode with linked temporal behavior must define:

- broker owner by interaction type
- broker epoch semantics
- origin tracking and cycle suppression
- handoff rules for replica recovery or leadership change

Rule:

- recovered brokers must resume only under a new epoch

### Shell-State Durability And Rehydration

Shell-local state must be classified as:

- `discardable`
- `rehydratable`
- `recoverable_draft`
- `non_rehydratable`

Required recovery basis:

- shell-instance ID
- recovery fence or session epoch
- last acknowledged temporal anchor per mission, session, or run
- invalidation rules for stale or excluded references

Rule:

- rehydration must resolve through canonical references, never mutable cached
  payloads

### Late-Event Behavior

Rules:

- late events must be inserted by canonical temporal anchor, not append order
- late events may not silently move the active `replay_cursor` unless the
  operator is in `live_tail` mode
- if late events invalidate current focus, selection, compare, or pin context,
  the shell must surface explicit supersession or tombstone state

