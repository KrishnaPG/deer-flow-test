# Tasks: Embed Hermes ACP Producer

## Phase 1: ACP Subprocess Producer Core
1. **Implement Hermes ACP subprocess manager:**
   - Launch Hermes ACP as a child process from Rust.
   - Reserve stdout for JSON-RPC protocol and stderr for diagnostics.
   - Detect startup, clean exit, abnormal exit, timeout, and crash conditions.
   - Emit producer lifecycle envelopes for material subprocess events.

2. **Implement ACP JSON-RPC stdio client:**
   - Encode producer-issued JSON-RPC commands to Hermes ACP.
   - Decode JSON-RPC requests, responses, and notifications observed on the boundary.
   - Preserve full raw frames for durable publication.

3. **Implement session and run registry:**
   - Track `session_id` as the durable conversational boundary.
   - Track `run_id` as one prompt execution within a session.
   - Keep optional `thread_id` support without requiring a source today.
   - Ensure the abstraction allows one subprocess to host multiple sessions later.

4. **Implement producer-owned sequencing:**
   - Assign monotonic `seq` within each `session_id`.
   - Use `session_id` as the Redpanda message key.
   - Ensure ordering remains reconstructible across multiple runs in the same session.

## Phase 2: Redpanda-First Raw Capture
5. **Implement Redpanda producer for `hermes.l0_drop`:**
   - Use `rust-rdkafka`.
   - Configure high-durability producer settings:
     - `acks=all`
     - idempotent producer enabled
     - retries with backoff
     - ordering-safe in-flight configuration
   - Expose broker/topic/compression settings via configuration.

6. **Publish every observable boundary event:**
   - Publish producer-issued command envelopes such as session create, session load, session resume, prompt, and cancel.
   - Publish observed ACP requests, responses, and notifications.
   - Publish lifecycle/error envelopes for subprocess and protocol failures.
   - Preserve `raw_payload` exactly as observed at the boundary.

7. **Preserve replay metadata without domain interpretation:**
   - Include `engine`, `schema_version`, `session_id`, `run_id`, `seq`, and `timestamp` on every envelope.
   - Include optional identifiers such as `thread_id`, `message_id`, and `tool_call_id` only when observable.
   - Do not assign Berg10 semantic fields or perform classification in the producer.

## Phase 3: Replay and Reliability
8. **Implement Redpanda-backed replay:**
   - Support replay by `session_id` and `seq` from Redpanda as the durable source.
   - Add an optional bounded in-memory replay tail only as a performance optimization.
   - Ensure reconstruction does not depend on process-local memory surviving restarts.

9. **Implement subprocess recovery and error surfacing:**
   - Handle malformed JSON-RPC frames, broken pipes, unexpected exits, and startup failures.
   - Decide and implement restart policy hooks without assuming a strict 1:1 process/session model forever.
   - Persist recovery-relevant lifecycle envelopes into Redpanda.

## Phase 4: Verification
10. **Verify raw capture fidelity:**
    - Run against an ephemeral Redpanda instance.
    - Exercise session create/load/resume, prompt execution, cancellation, and error paths.
    - Verify that every observable producer command and ACP frame is published to `hermes.l0_drop`.

11. **Verify ordering and replay:**
    - Confirm `session_id` partitioning and monotonic `seq` ordering.
    - Reconstruct a full session from Redpanda alone.
    - Verify replay works after producer restart without relying on local memory.

12. **Verify multi-session process model assumptions:**
    - Prove the code does not hard-code one subprocess per session.
    - Keep the default runtime policy at one subprocess per session.
    - Add tests or harness coverage showing registries and sequencing remain scoped by `session_id`.

## Deferred to Future Changes
- Berg10 ingestion and semantic mapping from raw envelopes.
- Public client-facing HTTP/WebSocket APIs.
- Presentation-layer chat streaming contract.
- File fallback as an active or recommended write path.
- Any modifications to `3rdParty/hermes-agent`.
