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

```bash
pip install -e ".[dev,ui]"

# Run the agent server
berg10-agent --model openrouter/google/gemma-3-2b-it:free --port 8765

# Run with custom config
berg10-agent --model openrouter/anthropic/claude-sonnet-4 --work-dir ./my-project

# Launch debug UI
streamlit run src/berg10_agent/ui/app.py
```

## Configuration

All options can be set via CLI flags, environment variables, or a `.env` file:

| Option | CLI Flag | Env Var | Default |
|--------|----------|---------|---------|
| Model | `--model` | `BERG10_AGENT_MODEL` | `openrouter/google/gemma-3-2b-it:free` |
| API Key | — | `BERG10_AGENT_API_KEY` | — |
| Host | `--host` | `BERG10_AGENT_HOST` | `0.0.0.0` |
| Port | `--port` | `BERG10_AGENT_PORT` | `8765` |
| Work Dir | `--work-dir` | `BERG10_AGENT_WORK_DIR` | `.` |
| Tool Timeout | `--tool-timeout` | `BERG10_AGENT_TOOL_TIMEOUT` | `60` |
| Max Turns | `--max-turns` | `BERG10_AGENT_MAX_TURNS` | `50` |

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

```bash
pip install -e ".[dev,ui]"
pytest
ruff check src/ tests/
black src/ tests/
```
