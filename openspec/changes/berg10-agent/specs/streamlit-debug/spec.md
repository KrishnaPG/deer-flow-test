## ADDED Requirements

### Requirement: Streamlit UI connects to running server

The debug UI SHALL connect to a running berg10-agent server via WebSocket.

#### Scenario: UI connects to server
- **WHEN** Streamlit app starts with server URL configured
- **THEN** a WebSocket connection is established to the server

### Requirement: UI shows streaming output

The debug UI SHALL display streaming tokens as they arrive.

#### Scenario: Tokens displayed
- **WHEN** server sends `{"type": "chunk", "content": "..."}` frames
- **THEN** the UI appends content to the chat display

### Requirement: UI shows tool status

The debug UI SHALL display tool execution status.

#### Scenario: Tool status displayed
- **WHEN** server sends tool status frames
- **THEN** the UI shows tool name and status (running/done)

### Requirement: UI sends messages

The debug UI SHALL allow users to send messages to the agent.

#### Scenario: User sends message
- **WHEN** user types a message and clicks send
- **THEN** the UI sends `{"type": "message", "content": "..."}` to server

### Requirement: UI operates in development mode only

The debug UI SHALL NOT be required for production operation.

#### Scenario: Production runs headless
- **WHEN** the server runs in production
- **THEN** the Streamlit UI is not launched (headless mode)