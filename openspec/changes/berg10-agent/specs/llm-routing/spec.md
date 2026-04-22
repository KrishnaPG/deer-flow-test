## ADDED Requirements

### Requirement: LLM client routes to configured model

The client SHALL route LLM requests to the model specified in configuration.

#### Scenario: Default model used
- **WHEN** no model is explicitly configured
- **THEN** the default model `openrouter/anthropic/claude-3-5-sonnet-20250609` is used

#### Scenario: Custom model from CLI
- **WHEN** `--model "openai/gpt-4o"` is passed
- **THEN** requests are routed to `openai/gpt-4o`

#### Scenario: Custom model from env var
- **WHEN** `BERG10_MODEL="anthropic/claude-sonnet-4-20250514"` is set
- **THEN** requests are routed to `anthropic/claude-sonnet-4-20250514`

### Requirement: LLM client supports streaming

The client SHALL yield tokens as they arrive from the LLM provider.

#### Scenario: Streaming response
- **WHEN** a completion call is made with streaming enabled
- **THEN** tokens are yielded incrementally as they arrive

### Requirement: LLM client handles provider errors

The client SHALL propagate provider errors with meaningful messages.

#### Scenario: API error
- **WHEN** the LLM provider returns an error
- **THEN** the error is raised with the provider's error message