## 1. Project Setup

- [ ] 1.1 Create `apps/berg10-agent/` directory structure
- [ ] 1.2 Create `pyproject.toml` with all dependencies and latest versions
- [ ] 1.3 Create `src/__init__.py` package init
- [ ] 1.4 Create `src/__main__.py` CLI entry with click

## 2. Constants & Protocol

- [ ] 2.1 Create `src/constants.py` with all enums (MessageType, ToolName, JsonRpcVersion, ErrorCode, etc.)
- [ ] 2.2 Create `src/protocol.py` with JSON-RPC 3.0 message builder helper functions
- [ ] 2.3 Create `src/config.py` for configuration management

## 2.5 Validator Registry

- [ ] 2.4 Create `src/validators/__init__.py` validator registry
- [ ] 2.5 Implement pattern-based validators (regex)
- [ ] 2.6 Implement operation-based validators (path_traversal)
- [ ] 2.7 Implement schema-based validators (JSON Schema)
- [ ] 2.8 Create validator configuration loading from config files

## 3. LLM Client

- [ ] 3.1 Create `src/llm/__init__.py`
- [ ] 3.2 Create `src/llm/client.py` with LiteLLM wrapper and streaming support

## 4. Tool Nodes

- [ ] 4.1 Create `src/nodes/__init__.py`
- [ ] 4.2 Create `src/nodes/compact_history.py`
- [ ] 4.3 Create `src/nodes/decide_action.py` with streaming
- [ ] 4.4 Create `src/nodes/list_files.py`
- [ ] 4.5 Create `src/nodes/grep_search.py`
- [ ] 4.6 Create `src/nodes/read_file.py`
- [ ] 4.7 Create `src/nodes/patch_file.py`
- [ ] 4.8 Create `src/nodes/run_command.py`

## 5. Agent Flow

- [ ] 5.1 Create `src/flow/__init__.py`
- [ ] 5.2 Create `src/flow/agent_flow.py` with AsyncFlow wiring

## 6. Gateway & Session

- [ ] 6.1 Create `src/server.py` with FastAPI WebSocket endpoint
- [ ] 6.2 Create `src/session.py` session interface (delegated to downstream)

## 7. Utilities

- [ ] 7.1 Create `src/utils/__init__.py`
- [ ] 7.2 Create `src/utils/memory.py` for .memory.md handling
- [ ] 7.3 Create `src/utils/skills.py` for AGENTS.md loading

## 8. Debug UI

- [ ] 8.1 Create `src/ui/__init__.py`
- [ ] 8.2 Create `src/ui/app.py` Streamlit debug interface

## 9. Testing

- [ ] 9.1 Create `tests/__init__.py`
- [ ] 9.2 Create `tests/test_protocol.py` for message builders
- [ ] 9.3 Create `tests/test_nodes.py` for tool nodes
- [ ] 9.4 Create `tests/test_flow.py` for agent flow

## 10. Documentation & Examples

- [ ] 10.1 Create `AGENTS.md.example` example skills file
- [ ] 10.2 Create `README.md` for the package