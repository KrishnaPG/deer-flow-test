# Design: Embed Hermes AIAgent Runner

## Architecture

The system introduces a new Python process (`hermes_bridge.py`) spawned by the Rust `deer_gui` frontend. It operates asynchronously, managing the `AIAgent` and pushing data to two disparate sinks.

### Components

1.  **`deer_gui` Rust App (`apps/deer_gui/src/bridge/adapter.rs`)**
    - Spawns `hermes_bridge.py` instead of the legacy `bridge.py` based on an environment variable or config toggle.
    - Communicates over `stdin`/`stdout` using JSON lines.

2.  **`hermes_bridge.py` (Python Adapter)**
    - Reads commands from `stdin` (`send_message`, `list_threads`, etc.).
    - Translates thread/message concepts into Hermes-compatible execution runs.
    - Registers callbacks on the `AIAgent` (`stream_delta_callback`, `tool_progress_callback`).
    
3.  **Sink A: GUI Output (Real-time)**
    - Maps Hermes text deltas and tool states into the `MessageRecord` format used by `deer_gui`.
    - Emits `{"kind": "event", "event": "message", ...}` payloads to `stdout`.

4.  **Sink B: L0 Drop Buffer (Durable Staging)**
    - Captures the *raw, unmodified* event payloads directly from the Hermes callbacks.
    - Organizes files by date: `<base_dir>/runners/hermes/l0_drop/YYYY-MM-DD/run_<uuid>.tmp`.
    - Appends events as JSON Lines (JSONL).
    - Executes `os.fsync()` for durability.
    - Atomically renames to `<uuid>.jsonl` on run completion.

## Data Model (Raw Event Shape)

The L0 drop files will contain a pure dump of what the Hermes engine produces. We do not impose canonical schemas at this stage.

Example JSONL row:
```json
{
  "timestamp": "2026-04-09T14:22:10Z",
  "run_id": "uuid-1234",
  "source_callback": "tool_progress",
  "raw_payload": {
    "name": "read_file",
    "status": "started",
    "args": {"path": "/etc/hosts"}
  }
}
```

## Storage Directory Strategy

- The system uses a centralized `<base_dir>` (resolved via environment variable `BERG10_BASE_DIR`).
- Agent runners have dedicated directories under `<base_dir>/runners/<engine_name>/` to isolate their subsystems.
- For Hermes:
  - Raw L0 drops: `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.tmp`
  - Engine-specific files: `<base_dir>/runners/hermes/skills/`, `<base_dir>/runners/hermes/config/`, etc.
- For DeerFlow:
  - `<base_dir>/runners/deerflow/` (future runner subsystems).
- For the VFS:
  - `<base_dir>/vfs/checkouts/`, `<base_dir>/vfs/catalog/`, `<base_dir>/vfs/content/` (managed by `berg10-storage`).
- **Schema Routing**: The ingestor determines how to parse events based on the path segment `runners/hermes/` vs `runners/deerflow/`, enabling schema-on-read without external registries.
- Directory structure:
  ```text
  <base_dir>/
  ├── vfs/
  │   ├── checkouts/
  │   ├── catalog/
  │   └── content/
  └── runners/
      ├── hermes/
      │   ├── l0_drop/
      │   │   ├── 2026-04-08/
      │   │   │   └── run_abc123.jsonl     (Completed, pending ingestion)
      │   │   └── 2026-04-09/
      │   │       ├── run_def456.jsonl     (Completed, pending ingestion)
      │   │       └── run_ghi789.tmp       (In-progress writing)
      │   ├── skills/
      │   └── config/
      └── deerflow/
          └── (future DeerFlow runner subsystems)
  ```

## State Machine (File Lifecycle)

1.  **Initialize:** User sends a prompt. Bridge generates `run_id`.
2.  **Open:** Open `<YYYY-MM-DD>/run_<run_id>.tmp` in append mode (`a`).
3.  **Stream:** Callbacks fire.
    - For `stream_delta`: Write to memory buffer.
    - For `tool_progress` / `completion`: Flush buffer to file, call `os.fsync(file.fileno())`.
4.  **Finalize:** Agent run returns. Close file handle. Execute `os.rename(tmp_path, json_path)`.
5.  **Failure:** If process exits abnormally, the file remains `.tmp`. Data up to the last `fsync` is preserved. Downstream cleaners can safely delete `.tmp` files older than 24 hours.