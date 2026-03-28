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

- `DEER_FLOW_CONFIG_PATH` to point at a Deer Flow `config.yaml` (defaults to `apps/deer_gui/config.yaml`)
- `DEER_FLOW_HOME` to control Deer Flow thread/artifact storage
- `DEER_GUI_HOME` to control the GUI's local metadata store
- `PYTHON_BIN` to override the Python executable used by the Rust app
- `DEER_GUI_LOG` to set log level for deer_gui modules (debug, info, warn, error, trace)
- `RUST_LOG` for fine-grained log control across all crates

## Configuration

Edit `apps/deer_gui/config.yaml` to set your LLM provider, model, and API key.
See `3rdParty/deer-flow/config.example.yaml` for all available options.

## Run

First setup the `.env` file:
```bash
cp apps/deer_gui/.env.example apps/deer_gui/.env
```

From the repository root:

```bash
cargo run -p deer-gui
```
or

```bash
OPENAI_API_KEY=sk-xxx DEER_GUI_LOG=debug cargo run -p deer-gui
```

# Reference:
 - Free [2D Spires and 3D Models](https://kenney.nl/assets)