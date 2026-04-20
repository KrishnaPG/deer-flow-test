## ADDED Requirements

### Requirement: Port LLM utility functions to Rust with Dapr Conversation API
The system SHALL port PocketFlow LLM utility functions to Rust using Dapr Conversation API for LLM interactions.

#### Scenario: LLM calls
- **WHEN** making LLM calls
- **THEN** the system SHALL use Dapr Conversation API for provider-agnostic LLM access
- **AND** the system SHALL support multiple LLM providers (OpenAI, Anthropic, etc.)
- **AND** the system SHALL handle rate limiting and retries

#### Scenario: Streaming responses
- **WHEN** streaming LLM responses
- **THEN** the system SHALL support streaming via Dapr Conversation API
- **AND** the system SHALL preserve token-by-token delivery semantics
- **AND** the system SHALL handle user interrupts and cancellations

### Requirement: Port chunking utilities to Rust
The system SHALL port PocketFlow chunking utilities to Rust for text processing.

#### Scenario: Text chunking
- **WHEN** chunking text for processing
- **THEN** the system SHALL support multiple chunking strategies (fixed size, semantic, etc.)
- **AND** the system SHALL preserve overlap and boundary handling
- **AND** the system SHALL maintain compatibility with Python chunking output

### Requirement: Port embedding utilities to Rust with Dapr bindings
The system SHALL port PocketFlow embedding utilities to Rust using Dapr bindings for vector generation.

#### Scenario: Text embedding
- **WHEN** generating embeddings for text
- **THEN** the system SHALL use Dapr bindings for embedding providers
- **AND** the system SHALL support batch embedding for efficiency
- **AND** the system SHALL handle embedding dimensionality and normalization

### Requirement: Port vector search utilities to Rust with Dapr state stores
The system SHALL port PocketFlow vector search utilities to Rust using Dapr state stores for similarity search.

#### Scenario: Similarity search
- **WHEN** searching for similar vectors
- **THEN** the system SHALL use Dapr state stores with vector capabilities
- **AND** the system SHALL support multiple distance metrics (cosine, euclidean, etc.)
- **AND** the system SHALL handle filtering and ranking

### Requirement: Port visualization utilities to Rust
The system SHALL port PocketFlow visualization utilities to Rust for workflow visualization.

#### Scenario: Workflow visualization
- **WHEN** visualizing PocketFlow workflows
- **THEN** the system SHALL generate Mermaid diagrams compatible with Python version
- **AND** the system SHALL support interactive visualization where possible
- **AND** the system SHALL preserve layout and styling options

### Requirement: Port web search utilities to Rust
The system SHALL port PocketFlow web search utilities to Rust for information retrieval.

#### Scenario: Web search
- **WHEN** performing web searches
- **THEN** the system SHALL support multiple search providers
- **AND** the system SHALL handle rate limiting and caching
- **AND** the system SHALL preserve result parsing and formatting

### Requirement: Port text-to-speech utilities to Rust
The system SHALL port PocketFlow text-to-speech utilities to Rust for audio generation.

#### Scenario: Speech synthesis
- **WHEN** converting text to speech
- **THEN** the system SHALL support multiple TTS providers
- **AND** the system SHALL handle audio formatting and playback
- **AND** the system preserve voice and quality options

### Requirement: Maintain utility compatibility with Python PocketFlow
The system SHALL ensure Rust utilities produce identical outputs to Python utilities for identical inputs.

#### Scenario: Output compatibility
- **WHEN** processing the same input with Python and Rust utilities
- **THEN** the system SHALL produce identical or functionally equivalent outputs
- **AND** the system SHALL handle edge cases and error conditions identically
- **AND** the system SHALL preserve configuration and option semantics