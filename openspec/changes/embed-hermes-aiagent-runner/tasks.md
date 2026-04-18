# Tasks: Embed Hermes In Our ACP Client

## Phase 1: ACP Client Core
- [x] Implement Hermes ACP subprocess manager design and task breakdown.
- [x] Implement our ACP Client connection layer using the Rust `agent-client-protocol` crate where appropriate.
- [x] Introduce context-sensitive strong runtime types such as `ChatSessionId`, `ChatRunId`, `ChatThreadId`, `AcpSubprocessId`, `AcpSessionSequenceNumber`, `AcpCapturedProtocolEvent`, and `AcpResponseStreamEvent`.
- [x] Implement chat session and run registry with optional `ChatThreadId` support and future multi-session-per-process compatibility.
- [x] Implement client-owned sequencing scoped to `ChatSessionId`.

## Phase 2: Redpanda-First Raw Capture
- [ ] Implement Redpanda publication for `hermes.l0_drop` with durable producer settings.
- [ ] Publish every observable boundary event: client-issued commands, observed ACP requests/responses/notifications, and lifecycle/error envelopes.
- [ ] Preserve replay metadata without Berg10 domain interpretation.

## Phase 3: Live Stream Now
- [x] Implement `AcpResponseStreamEvent` live fanout from ACP-observed events.
- [x] Keep assistant text fidelity honest and do not fabricate assistant deltas.
- [ ] Prepare a future hybrid seam for richer Hermes-specific stream adapters.

## Phase 4: Replay and Reliability
- [ ] Implement Redpanda-backed replay by `ChatSessionId` and `AcpSessionSequenceNumber`.
- [ ] Implement subprocess recovery and error surfacing with lifecycle persistence.

## Phase 5: Verification
- [ ] Verify raw capture fidelity against Hermes ACP and ephemeral Redpanda.
- [ ] Verify ordering and replay from Redpanda after restart.
- [ ] Verify current live stream fidelity: live tool/status events and coarse/final assistant text only.
- [ ] Verify multi-session process model assumptions.
- [ ] Verify ACP compatibility boundaries for future non-Hermes agents.

## Deferred to Future Changes
- Berg10 ingestion and semantic mapping from raw envelopes.
- Hermes-specific hybrid stream adapters for true incremental assistant-text deltas.
- Public client-facing HTTP/WebSocket APIs.
- File fallback as an active or recommended write path.
- Any modifications to `3rdParty/hermes-agent`.
