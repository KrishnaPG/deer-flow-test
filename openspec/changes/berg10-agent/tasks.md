## 1. Project Setup

- [x] 1.1 Create `apps/berg10-agent/` directory structure
- [x] 1.2 Create `pyproject.toml` with all dependencies and latest versions
- [x] 1.3 Create `src/berg10_agent/__init__.py` package init
- [x] 1.4 Create `src/berg10_agent/__main__.py` CLI entry with click

## 2. Constants & Protocol

- [x] 2.1 Create `src/berg10_agent/constants.py` with all enums (MessageType, ToolName, JsonRpcVersion, ErrorCode, etc.)
- [x] 2.2 Create `src/berg10_agent/protocol.py` with JSON-RPC 3.0 message builder helper functions
- [x] 2.3 Create `src/berg10_agent/config.py` for configuration management

## 2.5 Validator Registry

- [x] 2.4 Create `src/berg10_agent/validators/__init__.py` validator registry
- [x] 2.5 Implement pattern-based validators (regex)
- [x] 2.6 Implement operation-based validators (path_traversal)
- [x] 2.7 Implement schema-based validators (JSON Schema)
- [x] 2.8 Create validator configuration loading from config files

## 3. LLM Client

- [x] 3.1 Create `src/berg10_agent/llm/__init__.py`
- [x] 3.2 Create `src/berg10_agent/llm/client.py` with LiteLLM wrapper and streaming support

## 4. Tool Nodes

- [x] 4.1 Create `src/berg10_agent/nodes/__init__.py`
- [x] 4.2 Create `src/berg10_agent/nodes/compact_history.py`
- [x] 4.3 Create `src/berg10_agent/nodes/decide_action.py` with streaming
- [x] 4.4 Create `src/berg10_agent/nodes/list_files.py`
- [x] 4.5 Create `src/berg10_agent/nodes/grep_search.py`
- [x] 4.6 Create `src/berg10_agent/nodes/read_file.py`
- [x] 4.7 Create `src/berg10_agent/nodes/patch_file.py`
- [x] 4.8 Create `src/berg10_agent/nodes/run_command.py`

## 5. Agent Flow

- [x] 5.1 Create `src/berg10_agent/flow/__init__.py`
- [x] 5.2 Create `src/berg10_agent/flow/agent_flow.py` with AsyncFlow wiring

## 6. Gateway & Session

- [x] 6.1 Create `src/berg10_agent/server.py` with FastAPI WebSocket endpoint
- [x] 6.2 Create `src/berg10_agent/session.py` session interface (delegated to downstream)

## 7. Utilities

- [x] 7.1 Create `src/berg10_agent/utils/__init__.py`
- [x] 7.2 Create `src/berg10_agent/utils/memory.py` for .memory.md handling
- [x] 7.3 Create `src/berg10_agent/utils/skills.py` for AGENTS.md loading

## 8. Debug UI

- [x] 8.1 Create `src/berg10_agent/ui/__init__.py`
- [x] 8.2 Create `src/berg10_agent/ui/app.py` Streamlit debug interface

## 9. Testing

- [x] 9.1 Create `tests/__init__.py`
- [x] 9.2 Create `tests/test_protocol.py` for message builders
- [x] 9.3 Create `tests/test_nodes.py` for tool nodes
- [x] 9.4 Create `tests/test_flow.py` for agent flow

## 10. Documentation & Examples

- [x] 10.1 Create `AGENTS.md.example` example skills file
- [x] 10.2 Create `README.md` for the package
