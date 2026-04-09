# Tasks: Embed Hermes AIAgent Runner

## Phase 1: Hermes Producer and Bridge
1. **Create `apps/deer_gui/python/hermes_bridge.py` scaffold:**
   - Copy the basic stdio loop and command handling structure from `bridge.py`.
   - Remove LangChain `DeerFlowClient` initialization.
   - Insert Hermes `AIAgent` initialization.
   - Keep the bridge compatible with the current Rust `stdin`/`stdout` JSON-lines protocol.

2. **Implement Sink A (GUI Stdout):**
   - Implement `on_delta` and `on_tool_progress` callbacks.
   - Map these callbacks to the `{"kind": "event", "event": "message", ...}` stdout protocol expected by the Rust UI.
   - Ensure UI events remain thin and do not become the source of truth for storage.

3. **Implement Redpanda producer helper for raw L0 buffering:**
   - Add producer wiring in Python for topic `hermes.l0_drop`.
   - Configure reliable settings:
     - `acks=all`
     - idempotent producer enabled
     - retries with exponential backoff
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

## Phase 2: Rust Wiring
6. **Update `apps/deer_gui/src/bridge/adapter.rs`:**
   - Detect an environment variable (e.g., `DEER_ENGINE=hermes`).
   - If set, spawn `hermes_bridge.py` instead of `bridge.py`.
   - Export Redpanda configuration to the Python subprocess.
   - Keep `BERG10_BASE_DIR` available for runner-owned files and low-priority fallback layout, even though Redpanda is primary.
   - Ensure the process is started with the correct working directory and environment variables.

## Phase 3: Operations and Reliability
7. **Document and provision Redpanda topic behavior:**
   - Use topic name format `<engine>.l0_drop`.
   - Hermes topic is `hermes.l0_drop`.
   - Configure long-lived retention (target: one month).
   - Do not rely on short retention windows.
   - Deletion after successful ingestion is a future ingestor responsibility and must be reflected in broker/data lifecycle policy later.
   - Operational target: future ingestor runs at least daily in the worst case.

8. **Document low-priority file fallback (do not implement first):**
   - Fallback path remains `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.jsonl`.
   - This is dev-only and lower priority than the Redpanda path.

## Phase 4: Verification
9. **Producer verification:**
   - Run the GUI with Hermes selected and Redpanda configured.
   - Send prompts that exercise normal responses, tool calls, and reasoning.
   - Verify UI updates live.
   - Verify raw messages arrive on topic `hermes.l0_drop`.
   - Verify headers and payload fields are sufficient to reconstruct the session.

10. **Reliability verification:**
   - Simulate transient Redpanda failures and verify retries/backoff.
   - Verify duplicate publish protection with idempotent producer semantics.
   - Verify ordering within a session by consuming events back out of Redpanda.
