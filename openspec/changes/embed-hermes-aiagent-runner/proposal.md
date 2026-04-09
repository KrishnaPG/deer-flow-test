# Embed Hermes AIAgent Runner

## Summary
Provide an in-process Hermes AIAgent adapter (`hermes_bridge.py`) that can be launched as a backend generator. This adapter will act as a dual-sink: it streams live conversation events to the Rust Desktop GUI via `stdout` while simultaneously writing raw, unmapped session dumps (JSONL) to a local staging drop folder (`l0_drop`) on disk.

## Why
- Enables live integration with the existing `deer_gui` frontend with zero changes to the Rust UI codebase.
- Keeps storage canonical and durable: raw Hermes session dumps are staged safely on disk before being ingested into the content-addressable Virtual File System (VFS) as L0 records.
- Prevents "many small files" issues by batching a single conversational turn into a single file.
- Provides a clear boundary between raw data collection (Python) and structured data ingestion/normalization (Rust).

## Scope
In-scope:
- A new standalone Python bridge (`apps/deer_gui/python/hermes_bridge.py`) that instantiates `AIAgent`.
- Storage path resolution respects `<base_dir>/runners/<engine_name>/` hierarchy:
  - L0 raw drops: `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.tmp`
  - Supporting files: `<base_dir>/runners/hermes/skills/`, `<base_dir>/runners/hermes/config/`, etc.
- Dual-sink implementation: 
  - Emit UI-compatible JSON over `stdout`.
  - Append raw, unmapped Hermes callbacks to `<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/run_<uuid>.tmp`.
- Reliable file lifecycle: `flush` and `fsync` for durability, followed by an atomic rename to `.jsonl` when the turn completes.
- Rust GUI minor wiring to allow selecting the Hermes engine (launching `hermes_bridge.py` instead of `bridge.py`).
- Environment variable `BERG10_BASE_DIR` is passed from Rust to Python for path resolution.

Out-of-scope (initial iteration):
- Full ingestion pipeline implementation on the Rust side (the bridge just drops the files; ingestion/parsing is a separate downstream task).
- Full UI features (we are just providing the streaming data to the existing GUI).

## Acceptance Criteria
- Running the `deer_gui` with the Hermes engine selected launches `hermes_bridge.py`.
- Sending a prompt streams live text back to the `deer_gui` UI.
- Concurrently, a `.tmp` file is created in `<BERG10_BASE_DIR>/runners/hermes/l0_drop/<YYYY-MM-DD>/`.
- Raw Hermes callback events (deltas, tools, reasoning) are appended to the `.tmp` file.
- The `.tmp` file is atomically renamed to `.jsonl` when the agent finishes processing the prompt.
- The bridge does not crash on abandon/cancellation, leaving a partially written but uncorrupted `.tmp` file behind.
- Downstream ingestor can route events correctly by reading the `runners/hermes/` path segment.

## Risks & Mitigations
- **Disk I/O Blocking:** Heavy `fsync` calls could block the main agent thread, stuttering the `stdout` stream. 
  - *Mitigation:* Buffer low-value events (deltas) in memory and only flush/fsync periodically or on high-value events (tool boundaries, completion).
- **Orphaned Files:** Crashes may leave `.tmp` files forever.
  - *Mitigation:* The date-partitioned folder structure (`<base_dir>/runners/hermes/l0_drop/<YYYY-MM-DD>/`) allows a simple cron or cleanup script to prune orphaned files older than 24 hours.