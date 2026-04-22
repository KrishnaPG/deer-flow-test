# Berg10 Agent

Headless LLM agent with WebSocket transport using PocketFlow and LiteLLM.

## Features

- **WebSocket Transport**: Bidirectional JSON-RPC 3.0 communication over WebSockets
- **Multi-Provider LLM**: Route to 100+ LLM providers via LiteLLM
- **Async Agent Loop**: PocketFlow-based cyclic agent flow with tool execution
- **Security**: Extensible validator registry with path traversal, pattern, and schema validation
- **Streaming**: Real-time token streaming from LLM to WebSocket clients
- **Configurable**: CLI flags, environment variables, and `.env` file support

## Quick Start

**Launch debug UI** (requires agent running on port 8765):

```bash
# Option 1: Using run.sh (recommended - automatically handles venv)
./run.sh --model openrouter/inclusionai/ling-2.6-flash:free --port 8765

# launch debug UI (optional)
.venv/bin/streamlit run berg10_agent/ui/app.py --server.port 8501
```

## Configuration

All options can be set via CLI flags, environment variables, or a `.env` file:

| Option       | CLI Flag         | Env Var                     | Default                                |
| ------------ | ---------------- | --------------------------- | -------------------------------------- |
| Model        | `--model`        | `BERG10_AGENT_MODEL`        | `openrouter/google/gemma-3-2b-it:free` |
| API Key      | —                | `BERG10_AGENT_API_KEY`      | —                                      |
| Host         | `--host`         | `BERG10_AGENT_HOST`         | `0.0.0.0`                              |
| Port         | `--port`         | `BERG10_AGENT_PORT`         | `8765`                                 |
| Work Dir     | `--work-dir`     | `BERG10_AGENT_WORK_DIR`     | `.`                                    |
| Tool Timeout | `--tool-timeout` | `BERG10_AGENT_TOOL_TIMEOUT` | `60`                                   |
| Max Turns    | `--max-turns`    | `BERG10_AGENT_MAX_TURNS`    | `50`                                   |

## Architecture

```
src/berg10_agent/
  constants.py       # All enums, defaults, config keys
  protocol.py        # JSON-RPC 3.0 message builders
  config.py          # Configuration management
  server.py          # FastAPI WebSocket server
  session.py         # In-memory session management
  jsonrpc/           # Standalone JSON-RPC 3.0 implementation
  llm/               # LiteLLM wrapper with streaming
  nodes/             # PocketFlow tool nodes
  flow/              # Agent flow wiring
  validators/        # Security validation registry
  utils/             # Memory and skills loading
  ui/                # Streamlit debug interface
```

## WebSocket Protocol

Connect to `ws://localhost:8765/ws` and send JSON-RPC 3.0 messages:

```json
// Send a message
{"type": "message", "content": "List all Python files", "id": "msg-1"}

// Cancel current operation
{"type": "interrupt", "id": "cancel-1"}
```

## Development

Always use the virtual environment (`.venv`) for all Python commands:

```bash
# Install dependencies
.venv/bin/pip install -e ".[dev,ui]"

# Run tests
.venv/bin/pytest

# Lint and format
.venv/bin/ruff check berg10_agent/ tests/
.venv/bin/black berg10_agent/ tests/
```
