## ADDED Requirements

### Requirement: Server accepts WebSocket connections

The server SHALL expose a WebSocket endpoint at `/ws` that accepts bidirectional connections.

#### Scenario: Client connects to WebSocket
- **WHEN** a client connects to `ws://host:port/ws`
- **THEN** the server accepts the connection and maintains it until disconnected

### Requirement: Server receives message frames

The server SHALL parse incoming JSON messages and extract the `type` and `content` fields.

#### Scenario: Client sends message
- **WHEN** client sends `{"type": "message", "content": "Fix the tests"}`
- **THEN** server extracts `type="message"` and `content="Fix the tests"` for processing

### Requirement: Server sends streaming chunks

The server SHALL send JSON frames with `type: "chunk"` as LLM tokens arrive.

#### Scenario: Server streams tokens
- **WHEN** LLM generates tokens
- **THEN** server immediately sends `{"type": "chunk", "content": "token"}` frames

### Requirement: Server sends tool status

The server SHALL send tool execution status frames.

#### Scenario: Tool starts executing
- **WHEN** a tool node begins execution
- **THEN** server sends `{"type": "tool", "name": "run_command", "status": "running", "args": {...}}`

#### Scenario: Tool completes
- **WHEN** a tool node completes execution
- **THEN** server sends `{"type": "tool", "name": "run_command", "status": "done", "output": "...}}`

### Requirement: Server handles interrupt

The server SHALL support interrupt messages to cancel ongoing operations.

#### Scenario: Client sends interrupt
- **WHEN** client sends `{"type": "interrupt"}`
- **THEN** server cancels any in-progress tool execution and returns control to client