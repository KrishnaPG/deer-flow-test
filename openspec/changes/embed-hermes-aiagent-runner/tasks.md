# Tasks: Embed Hermes AIAgent Runner

## Phase 1: Python Bridge Implementation
1. **Create `apps/deer_gui/python/hermes_bridge.py` scaffold:**
   - Copy the basic stdio loop and command handling structure from `bridge.py`.
   - Rip out the LangChain `DeerFlowClient` initialization.
   - Insert Hermes `AIAgent` initialization.

2. **Implement Sink A (GUI Stdout):**
   - Implement `on_delta` and `on_tool_progress` callbacks.
   - Map these callbacks to the `{"kind": "event", "event": "message", ...}` stdout protocol expected by the Rust UI.

3. **Implement Sink B (L0 Disk Drops):**
   - Create a `L0DropWriter` class to handle the file lifecycle.
   - Implement date-based directory creation (`<drop_root>/l0_drop/YYYY-MM-DD/`).
   - Implement buffered `.tmp` writing with explicit `flush()` and `os.fsync()`.
   - Wire the Hermes callbacks to dump raw JSON payloads into this writer.
   - Implement the atomic rename from `.tmp` to `.json` when the `AIAgent.run` completes.

## Phase 2: Rust Wiring
4. **Update `apps/deer_gui/src/bridge/adapter.rs`:**
   - Detect an environment variable (e.g., `DEER_ENGINE=hermes`).
   - If set, spawn `hermes_bridge.py` instead of `bridge.py`.
   - Ensure the process is started with the correct working directory and environment variables.

## Phase 3: Verification
5. **Manual Testing:**
   - Run `DEER_ENGINE=hermes cargo run -p deer-gui`.
   - Send a prompt.
   - Verify UI updates live.
   - Verify `<l0_drop>/YYYY-MM-DD/` contains the `.json` dump file with the raw events.
   - Verify the file contains valid JSON Lines.