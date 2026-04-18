# Tasks: Embed Hermes In Our ACP Client

## Phase 1: ACP Client Core
1. **Implement Hermes ACP subprocess manager:**
   - Launch Hermes ACP as a child process from Rust.
   - Reserve stdout for JSON-RPC protocol and stderr for diagnostics.
   - Detect startup, clean exit, abnormal exit, timeout, and crash conditions.
   - Emit subprocess lifecycle events for material lifecycle transitions.

2. **Implement our ACP Client connection layer:**
   - Use the Rust `agent-client-protocol` crate where appropriate for ACP request/response/session-update typing.
   - Wire child-process stdin/stdout into the ACP client connection.
   - Preserve full raw frames for durable publication.

3. **Introduce context-sensitive strong runtime types:**
   - Define strongly typed ids and events such as:
     - `ChatSessionId`
     - `ChatRunId`
     - `ChatThreadId`
     - `AcpSubprocessId`
     - `AcpSessionSequenceNumber`
     - `AcpCapturedProtocolEvent`
     - `AcpResponseStreamEvent`
   - Avoid generic names such as `RawObservedEvent`, `UiStreamEvent`, or plain `SessionId` for runtime-owned concepts.

4. **Implement chat session and run registry:**
   - Track `ChatSessionId` as the durable conversational boundary.
   - Track `ChatRunId` as one prompt execution within a session.
   - Keep optional `ChatThreadId` support without requiring a source today.
   - Ensure the abstraction allows one subprocess to host multiple sessions later.

5. **Implement client-owned sequencing:**
   - Assign monotonic `AcpSessionSequenceNumber` within each `ChatSessionId`.
   - Use `ChatSessionId` as the Redpanda message key.
   - Ensure ordering remains reconstructible across multiple runs in the same session.

## Phase 2: Redpanda-First Raw Capture
6. **Implement Redpanda publication for `hermes.l0_drop`:**
   - Use `rust-rdkafka`.
   - Configure high-durability producer settings:
     - `acks=all`
     - idempotent producer enabled
     - retries with backoff
     - ordering-safe in-flight configuration
   - Expose broker/topic/compression settings via configuration.

7. **Publish every observable boundary event:**
   - Publish client-issued command envelopes such as session create, session load, session resume, prompt, and cancel.
   - Publish observed ACP requests, responses, and notifications.
   - Publish lifecycle/error envelopes for subprocess and protocol failures.
   - Preserve raw payload exactly as observed at the boundary.

8. **Preserve replay metadata without domain interpretation:**
   - Include `engine`, `schema_version`, `session_id`, `run_id`, `seq`, and `timestamp` on every envelope.
   - Include optional identifiers such as `thread_id`, `message_id`, and `tool_call_id` only when observable.
   - Do not assign Berg10 semantic fields or perform classification in our ACP Client.

## Phase 3: Live Stream Now
9. **Implement `AcpResponseStreamEvent` live fanout:**
   - Expose a live response stream immediately from ACP-observed events.
   - Emit live subprocess/session/run status updates.
   - Emit live tool activity.
   - Emit live coarse thought/status updates available from Hermes ACP.
   - Emit assistant text only at the current Hermes ACP granularity.

10. **Do not fabricate assistant text deltas:**
    - Detect the current Hermes ACP limitation around assistant text granularity.
    - Do not synthesize token-level or delta-level assistant text from a final response blob.
    - Keep the live stream honest about source fidelity.

11. **Prepare a future hybrid seam:**
    - Keep ACP as the primary control/session boundary.
    - Design the live stream layer so a future Hermes-specific stream adapter can enrich `AcpResponseStreamEvent` with true text deltas.
    - Do not couple current ACP Client control logic to Hermes-specific TUI gateway details.

## Phase 4: Replay and Reliability
12. **Implement Redpanda-backed replay:**
    - Support replay by `ChatSessionId` and `AcpSessionSequenceNumber` from Redpanda as the durable source.
    - Add an optional bounded in-memory replay tail only as a performance optimization.
    - Ensure reconstruction does not depend on process-local memory surviving restarts.

13. **Implement subprocess recovery and error surfacing:**
    - Handle malformed JSON-RPC frames, broken pipes, unexpected exits, and startup failures.
    - Decide and implement restart policy hooks without assuming a strict 1:1 process/session model forever.
    - Persist recovery-relevant lifecycle envelopes into Redpanda.

## Phase 5: Verification
14. **Verify raw capture fidelity:**
    - Run against an ephemeral Redpanda instance.
    - Exercise session create/load/resume, prompt execution, cancellation, and error paths.
    - Verify that every observable client command and ACP frame is published to `hermes.l0_drop`.

15. **Verify ordering and replay:**
    - Confirm `ChatSessionId` partitioning and monotonic `AcpSessionSequenceNumber` ordering.
    - Reconstruct a full session from Redpanda alone.
    - Verify replay works after client restart without relying on local memory.

16. **Verify current live stream fidelity:**
    - Confirm live tool/status events appear before run completion.
    - Confirm assistant text appears only at the coarse/final granularity Hermes ACP currently exposes.
    - Confirm no synthetic assistant text deltas are emitted.

17. **Verify multi-session process model assumptions:**
    - Prove the code does not hard-code one subprocess per session.
    - Keep the default runtime policy at one subprocess per session.
    - Add tests or harness coverage showing registries and sequencing remain scoped by `ChatSessionId`.

18. **Verify ACP compatibility for future non-Hermes agents:**
    - Keep the ACP control/session model agent-agnostic.
    - Verify local runtime abstractions do not hard-code Hermes-specific assumptions beyond the current subprocess launcher.

## Deferred to Future Changes
- Berg10 ingestion and semantic mapping from raw envelopes.
- Hermes-specific hybrid stream adapters for true incremental assistant-text deltas.
- Public client-facing HTTP/WebSocket APIs.
- File fallback as an active or recommended write path.
- Any modifications to `3rdParty/hermes-agent`.
