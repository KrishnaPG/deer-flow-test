# Deer GUI

`deer-gui` is a desktop chat app built with `eframe/egui` that talks to Deer Flow through an embedded Python bridge instead of the HTTP server used by the web frontend.

## Architecture

- Rust `egui` desktop UI for threads, messages, uploads, artifacts, and model/mode controls
- Python bridge process over JSON-lines stdio
- `DeerFlowClient` as the in-process backend client
- Local thread metadata store for thread list, transcript persistence, and artifact tracking

## Runtime flow

1. The Rust app spawns `python/bridge.py` with unbuffered stdio.
2. The UI sends JSON commands such as `list_threads`, `create_thread`, `send_message`, and `resolve_artifact`.
3. The bridge executes Deer Flow client calls and emits streamed events back to the UI.
4. The UI updates transcript, title, artifacts, and suggestions as events arrive.

## Environment

The bridge resolves Deer Flow in this repo directly. Useful environment variables:

- `DEER_FLOW_CONFIG_PATH` to point at a Deer Flow `config.yaml`
- `DEER_FLOW_HOME` to control Deer Flow thread/artifact storage
- `DEER_GUI_HOME` to control the GUI's local metadata store
- `PYTHON_BIN` to override the Python executable used by the Rust app

## Run

From the repository root:

```bash
cargo run -p deer-gui
```
