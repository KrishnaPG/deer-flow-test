# Decisions: Embed Hermes In Our ACP Client

## Confirmed decisions

### 1. Call the Rust integration layer `our ACP Client`
The Rust side is referred to as `our ACP Client`, not a generic `producer`.

- Use `producer` only when specifically discussing Redb/Kafka publication semantics.
- Use `our ACP Client` for the application/runtime integration boundary.

### 2. ACP remains the primary control and session boundary
ACP is the canonical control/session protocol for Hermes now and future non-Hermes agents later.

- Session lifecycle, prompt execution, cancellation, and replay sequencing are all anchored at the ACP boundary.
- Future Hermes-specific richer streaming must augment ACP, not replace it.

### 3. Redb is the first durable landing zone
`hermes.l0_drop` on Redb is the first durable raw write path.

- This path is low-latency and durable.
- It is not an ETL layer.
- Berg10 interpretation and ingestion happen later.

### 4. Preserve everything observable
Our ACP Client preserves every observable boundary event it can capture.

- client-issued commands
- ACP requests
- ACP responses
- ACP notifications
- subprocess lifecycle and error events

This is necessary for replay, audit, and future UI reconstruction.

### 5. Use context-sensitive strong types
Avoid generic runtime names.

Preferred examples:

- `ChatSessionId`
- `ChatRunId`
- `ChatThreadId`
- `AcpSubprocessId`
- `AcpSessionSequenceNumber`
- `AcpCapturedProtocolEvent`
- `AcpResponseStreamEvent`

### 6. Use Rust `agent-client-protocol` crate where appropriate
The Rust ACP crate should be used where it meaningfully reduces wire/protocol parsing risk.

Recommended reuse:

- ACP request/response/session-update typing
- ACP connection and message handling over byte streams

Keep local ownership of:

- subprocess management
- runtime ids and registries
- sequencing
- Redb envelopes
- live stream projection

### 7. Expose a live stream now
Our ACP Client should expose a live stream immediately.

Current live stream fidelity from Hermes ACP:

- live tool activity
- live coarse thought/status events
- final/coarse assistant text

### 8. Do not fabricate assistant deltas
Current Hermes ACP does not expose true incremental assistant text deltas.

- Do not synthesize token-by-token or chunk-by-chunk assistant text from final text.
- The live stream must reflect actual source fidelity.

### 9. Hybrid Hermes-specific streaming is deferred
True incremental assistant text is deferred to a future Hermes-specific hybrid stream adapter.

Candidate example:

- TUI gateway or similar Hermes-specific observability surface

When added later:

- ACP still remains the control/session boundary
- the hybrid stream only enriches live observability

### 10. `thread_id` remains optional
`thread_id` stays optional until a real source exists.

- This preserves compatibility with future manual branching initiated from UI or orchestration.
