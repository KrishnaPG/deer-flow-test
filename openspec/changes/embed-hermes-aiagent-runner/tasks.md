# Tasks: Embed Hermes AIAgent Runner

## Phase 1: Backend Hermes Runner Service
1. **Create a backend-hosted Hermes runner service:**
   - Implement the service in Rust.
   - Host Hermes `AIAgent` in a backend process/service instead of a UI-spawned subprocess.
   - Define the initial API surface as `POST /runs`, `POST /runs/{run_id}/cancel`, and `WS /runs/{run_id}/events`.
   - Keep the service reusable by desktop and future web/WASM clients.

2. **Implement Sink A (UI stream API):**
   - Implement `on_delta` and `on_tool_progress` callbacks.
   - Map these callbacks to a backend stream event model usable by desktop and web/WASM clients.
   - Use WebSocket as the primary streaming transport.
   - Deliver the same logical event envelope to the UI that is published to Redpanda.
   - Ensure UI events remain thin and do not become the source of truth for storage.

3. **Implement Redpanda producer helper for raw L0 buffering:**
   - Add producer wiring in the Hermes runner for topic `hermes.l0_drop`.
   - Use `rust-rdkafka` for broker integration.
   - Configure reliable settings:
      - `acks=all`
      - idempotent producer enabled
      - retries with exponential backoff
      - ordering-safe in-flight request configuration
    - Accept optional gzip compression for message values to reduce broker storage usage.
    - Expose producer config via environment variables (bootstrap servers, topic override, compression toggle).

4. **Publish raw Hermes events to Redpanda:**
   - Wrap every Hermes callback payload in a raw envelope containing:
     - `engine`
     - `schema_version`
     - `session_id`
     - `thread_id` when available
     - `run_id`
     - `message_id` when available
     - `tool_call_id` when available
     - `seq`
     - `event_kind`
     - `timestamp`
     - `raw_payload`
   - Publish each envelope to `hermes.l0_drop` using `session_id` as the message key.
   - Emit explicit finish/completion events so future ingestion can reconstruct final assistant responses and run boundaries.

5. **Preserve session reconstruction fidelity:**
    - Ensure user request payloads and assistant response payloads are both emitted.
    - Preserve tool call/tool result linkage.
    - Preserve reasoning payloads and finish reasons when Hermes makes them available.
    - Add monotonic sequencing within a session so consumers can replay the session deterministically.

6. **Add reconnect and replay support in the runner:**
   - Keep a bounded in-memory replay window per run/session for fast reconnects.
   - Support client replay requests using `replay_from_seq` on the WebSocket subscription.
   - Fall back to Redpanda-backed replay when the requested sequence is older than the in-memory cache.

## Phase 2: Client Wiring
7. **Update `apps/deer_gui` to use the backend API:**
    - Replace direct subprocess assumptions in the bridge/orchestrator layer with a transport that can talk to the Hermes runner service.
    - Replace the Python subprocess path with HTTP control calls plus WebSocket event streaming.
    - Preserve the higher-level app/orchestrator model so the UI can remain mostly unchanged.

8. **Prepare `apps/deer_chat_lab` as a thin consumer of the same backend API:**
    - Reuse the shared transport-neutral event contract.
    - Ensure future web/WASM compatibility by avoiding desktop-only process APIs.

## Phase 3: Operations and Reliability
9. **Document and provision Redpanda topic behavior:**
   - Use topic name format `<engine>.l0_drop`.
   - Hermes topic is `hermes.l0_drop`.
   - Configure long-lived retention (target: one month).
   - Do not rely on short retention windows.
   - Deletion after successful ingestion is a future ingestor responsibility and must be reflected in broker/data lifecycle policy later.
   - Operational target: future ingestor runs at least daily in the worst case.

10. **Document low-priority file fallback (do not implement first):**
   - Fallback path remains `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.jsonl`.
   - This is dev-only and lower priority than the Redpanda path.

## Phase 4: Verification
11. **Producer verification:**
   - Run the backend Hermes service with Redpanda configured.
   - Connect from a client (`deer_gui` first, later web/WASM-capable clients).
   - Send prompts that exercise normal responses, tool calls, and reasoning.
   - Verify UI updates live through the backend stream API.
   - Verify raw messages arrive on topic `hermes.l0_drop`.
   - Verify headers and payload fields are sufficient to reconstruct the session.

12. **Reliability verification:**
    - Simulate transient Redpanda failures and verify retries/backoff.
    - Verify duplicate publish protection with idempotent producer semantics.
    - Verify ordering within a session by consuming events back out of Redpanda.

13. **Replay verification:**
    - Run integration tests with an ephemeral Redpanda instance.
    - Disconnect and reconnect a client mid-run and request replay from a prior `seq`.
    - Verify the runner serves replay from memory when available and from Redpanda when the replay point is older than the local cache.
    - Verify WebSocket-delivered events match Redpanda envelopes for identifiers and ordering.
