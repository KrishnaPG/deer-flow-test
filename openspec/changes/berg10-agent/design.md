## Context

Build a headless LLM agent using PocketFlow + LiteLLM with WebSocket transport. The agent should:

- Accept input via WebSocket messages (not stdio)
- Stream tokens and tool status back via WebSocket
- Be configurable (model, provider) via CLI/env
- Support tool nodes similar to hermes-agent (list, grep, read, patch, run)
- Run headless in production; Streamlit UI for dev/debug only
- Session state handled by downstream storage layer

## Goals / Non-Goals

**Goals:**

- Bidirectional WebSocket communication for all I/O
- Async-native streaming (no thread-safety bridging needed)
- LiteLLM for multi-provider routing (OpenAI, Anthropic, OpenRouter, etc.)
- Configurable LLM model via CLI (`--model`) or env (`BERG10_MODEL`)
- Extensible message types via centralized constants
- All constants/enums in one file for easy modification
- Streamlit debug UI that connects to running server

**Non-Goals:**

- REST API (WebSocket-only for now)
- Session persistence in this crate (delegated to downstream)
- Platform gateway adapters (Telegram, Discord, etc.)
- Cron/scheduled automations
- Built-in auth (handled by downstream)

## Decisions

### D1: PocketFlow from PyPI vs vendored

- **Decision**: Install from PyPI (`pip install pocketflow`)
- **Rationale**: 200 LOC, no transitive deps, officially maintained

### D2: LLM routing library

- **Decision**: LiteLLM (42K stars, production-ready, 100+ providers)
- **Rationale**: Battle-tested, OpenAI-compatible, streaming support

### D3: Config management

- **Decision**: python-dotenv for env file loading + CLI overrides
- **Rationale**: Minimal dep, well-tested, standard OSS pattern

### D4: Message type extensibility

- **Decision**: Centralized constants.py with Enums + protocol.py helper functions
- **Rationale**: Add new message types by editing constants.py only; no scattered strings

### D5: Streamlit integration

- **Decision**: UI connects to running WS server (not embedded)
- **Rationale**: Separation of concerns; server can run headless without UI

### D6: WebSocket library

- **Decision**: Built-in FastAPI WebSocket (no custom websockets library)
- **Rationale**: FastAPI includes battle-tested WebSocket support; no extra deps needed

## Risks / Trade-offs

- [Risk] WebSocket connection drops during long tool execution → Mitigation: Add heartbeat/keepalive mechanism in protocol
- [Risk] LiteLLM API changes → Mitigation: Pin to major version, update on release cycles
- [Risk] No built-in auth → Mitigation: Handled by downstream berg10-stack; this is headless by design

## Open Questions

- Q1: Default model selection - should we default to a specific provider or leave unspecified?
- Q2: Rate limiting - should be handled by downstream or added here?
- Q3: Tool timeout - what default should we use (30s, 60s, configurable)?