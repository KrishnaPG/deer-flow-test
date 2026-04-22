## Why

Build a headless LLM agent (similar to hermes-agent) using PocketFlow as the agent framework, with bidirectional WebSockets for real-time streaming input/output instead of stdio. This enables integration with any client (TUI, web, mobile) while leveraging battle-tested OSS libraries (LiteLLM for multi-provider LLM routing, FastAPI for WebSockets).

## What Changes

- Create new Python package `berg10-agent` at `apps/berg10-agent/`
- Use PocketFlow (PyPI) for async agent orchestration
- Use LiteLLM for battle-tested multi-provider LLM routing (100+ providers)
- Implement bidirectional WebSocket gateway (FastAPI) for headless operation
- Include Streamlit debug UI that connects to a running agent server
- Session state managed by downstream (`berg10-storage-vfs`)
- All constants, enums, message types centralized in `constants.py`
- Configurable model via CLI flag or env var

## Capabilities

### New Capabilities

- `websocket-agent`: Headless LLM agent with WebSocket input/output streaming (JSON-RPC 3.0)
- `llm-routing`: Multi-provider LLM routing via LiteLLM (OpenAI, Anthropic, OpenRouter, etc.)
- `tool-nodes`: Reusable tool nodes (list_files, grep_search, read_file, patch_file, run_command)
- `agent-loop`: Cyclic AsyncFlow with decision-making and tool execution
- `streamlit-debug`: Debug UI connecting to running agent via WebSocket
- `config-management`: Configuration via python-dotenv with CLI overrides
- `validator-registry`: Extensible security validation for tool inputs (pattern-based, operation-based, schema-based)
- `session-interface`: In-memory session management with sync interface to downstream storage

### Modified Capabilities

(none - this is a new capability)

## Impact

- New package: `apps/berg10-agent/`
- Dependencies: pocketflow, litellm, fastapi, uvicorn, httpx, pydantic, python-dotenv
- No REST API for now - WebSocket only
- Session persistence delegated to downstream