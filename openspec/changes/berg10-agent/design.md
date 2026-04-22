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

## Coding Standards

- Avoid monolith files (> 400 LOC); split into smaller files;
- Avoid large functions > 50 LOC
- Maintain all hardcoded values in a central location easy to edit and configure.
- Do not inline the paths or hardcoded structures (e.g. json responses); use reusable builder helper methods; 
- Aim for high type-safety for all domain types; Avoid using basic types (e.g. string, number) as much as possible; 
- Avoid memory allocations or memory copies; aim for zero-copy pipelines;
- Start at 30000 feet (identify all the high-level interfaces, types, function prototypes, contracts, constants, response structures, helper methods, imports) and work down towards the function level code step by step using a todo-list to keep track of the steps;

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

### D6: WebSocket protocol

- **Decision**: JSON-RPC 3.0 over FastAPI WebSocket
- **Rationale**: Built-in streaming, ACK, cancellation, and structured error codes. Standardizes multi-stream handling and cancel semantics.

### D7: Session management

- **Decision**: In-memory session state managed by berg10-agent, with abstract sync interface to downstream storage
- **Rationale**: Agent needs live session state (history, context) during a WebSocket connection. Downstream (`berg10-storage-vfs`) handles durability/long-term persistence.

### D8: Security validation extensibility

- **Decision**: Pluggable validator registry for tool inputs (pattern-based, operation-based, schema-based)
- **Rationale**: Security checks should be configurable/extensible, not hardcoded. Allow custom validators for path traversal, dangerous commands, format validation, etc.

## Risks / Trade-offs

- [Risk] WebSocket connection drops during long tool execution → Mitigation: JSON-RPC 3.0 stream cancellation (client can send `request.cancel`)
- [Risk] LiteLLM API changes → Mitigation: Pin to major version, update on release cycles
- [Risk] No built-in auth → Mitigation: Handled by downstream berg10-stack; this is headless by design
- [Risk] Tool inputs can escape workdir → Mitigation: Pluggable validator registry with path-traversal checks enabled by default

## Open Questions

- Q1: Default model selection - should we default to a specific provider or leave unspecified?
- Q2: Rate limiting - should be handled by downstream or added here?
- Q3: Tool timeout - what default should we use (30s, 60s, configurable)?