## ADDED Requirements

### Requirement: Port LLM utility functions to Rust with Dapr Conversation API
The system SHALL port PocketFlow LLM utility functions to Rust using Dapr Conversation API for LLM interactions with caching and rate limiting.

#### Scenario: LLM calls
- **WHEN** making LLM calls
- **THEN** the system SHALL use Dapr Conversation API for provider-agnostic LLM access
- **AND** the system SHALL support multiple LLM providers (OpenAI, Anthropic, etc.)
- **AND** the system SHALL handle rate limiting and retries
- **AND** the system SHALL cache responses with configurable TTL

#### Scenario: Streaming responses
- **WHEN** streaming LLM responses
- **THEN** the system SHALL support streaming via Dapr Conversation API
- **AND** the system SHALL preserve token-by-token delivery semantics
- **AND** the system SHALL handle user interrupts and cancellations
- **AND** the system SHALL support partial response caching

#### Scenario: Fallback to direct HTTP
- **WHEN** Dapr Conversation API is unavailable
- **THEN** the system SHALL fallback to direct HTTP calls to LLM providers
- **AND** the system SHALL maintain consistent API across fallback
- **AND** the system SHALL log fallback usage for observability

### Requirement: Port chunking utilities to Rust
The system SHALL port PocketFlow chunking utilities to Rust for text processing using pure Rust implementations.

#### Scenario: Text chunking
- **WHEN** chunking text for processing
- **THEN** the system SHALL support multiple chunking strategies (fixed size, semantic, etc.)
- **AND** the system SHALL preserve overlap and boundary handling
- **AND** the system SHALL maintain compatibility with Python chunking output

#### Scenario: Token-aware chunking
- **WHEN** chunking for LLM context limits
- **THEN** the system SHALL support token-based chunking using tokenizer crates
- **AND** the system SHALL respect model-specific token limits
- **AND** the system SHALL provide chunking statistics (count, sizes)

### Requirement: Port embedding utilities to Rust with Dapr Conversation API + local models
The system SHALL port PocketFlow embedding utilities to Rust using Dapr Conversation API for cloud embeddings and local Rust models for offline use.

#### Scenario: Cloud embedding
- **WHEN** generating embeddings via cloud providers
- **THEN** the system SHALL use Dapr Conversation API for provider-agnostic access
- **AND** the system SHALL support batch embedding for efficiency
- **AND** the system SHALL handle embedding dimensionality and normalization
- **AND** the system SHALL cache embeddings in Dapr State

#### Scenario: Local embedding
- **WHEN** generating embeddings offline
- **THEN** the system SHALL use local Rust embedding models
- **AND** the system SHALL support model downloading and caching
- **AND** the system SHALL provide consistent API across cloud and local

### Requirement: Port vector search utilities to Rust with Dapr Vector bindings
The system SHALL port PocketFlow vector search utilities to Rust using Dapr Vector bindings for similarity search.

#### Scenario: Similarity search
- **WHEN** searching for similar vectors
- **THEN** the system SHALL use Dapr Vector bindings for provider-agnostic access
- **AND** the system SHALL support multiple distance metrics (cosine, euclidean, etc.)
- **AND** the system SHALL handle filtering and ranking
- **AND** the system SHALL cache search results in Dapr State

#### Scenario: Vector database operations
- **WHEN** performing CRUD operations on vectors
- **THEN** the system SHALL use Dapr Vector bindings for insert, update, delete
- **AND** the system SHALL support batch operations for efficiency
- **AND** the system SHALL handle vector database-specific features

### Requirement: Port web search utilities to Rust with Dapr bindings
The system SHALL port PocketFlow web search utilities to Rust using Dapr bindings for information retrieval with caching.

#### Scenario: Web search
- **WHEN** performing web searches
- **THEN** the system SHALL use Dapr bindings for provider-agnostic search
- **AND** the system SHALL support multiple search providers
- **AND** the system SHALL handle rate limiting and caching
- **AND** the system SHALL preserve result parsing and formatting

#### Scenario: Search result caching
- **WHEN** caching search results
- **THEN** the system SHALL use Dapr State for result caching
- **AND** the system SHALL support configurable TTL and invalidation
- **AND** the system SHALL provide cache hit/miss metrics

### Requirement: Port visualization utilities to Rust with Dapr State caching
The system SHALL port PocketFlow visualization utilities to Rust for workflow visualization with distributed caching.

#### Scenario: Workflow visualization
- **WHEN** visualizing PocketFlow workflows
- **THEN** the system SHALL generate Mermaid diagrams compatible with Python version
- **AND** the system SHALL support interactive visualization where possible
- **AND** the system SHALL preserve layout and styling options
- **AND** the system SHALL cache generated diagrams in Dapr State

#### Scenario: Diagram rendering
- **WHEN** rendering diagrams to images
- **THEN** the system SHALL support multiple output formats (SVG, PNG, etc.)
- **AND** the system SHALL use pure Rust rendering libraries
- **AND** the system SHALL provide rendering statistics

### Requirement: Port text-to-speech utilities to Rust with Dapr bindings
The system SHALL port PocketFlow text-to-speech utilities to Rust using Dapr bindings for audio generation with caching.

#### Scenario: Speech synthesis
- **WHEN** converting text to speech
- **THEN** the system SHALL use Dapr bindings for provider-agnostic TTS
- **AND** the system SHALL support multiple TTS providers
- **AND** the system SHALL handle audio formatting and playback
- **AND** the system SHALL preserve voice and quality options

#### Scenario: Audio caching
- **WHEN** caching generated audio
- **THEN** the system SHALL use Dapr State for audio caching
- **AND** the system SHALL support configurable TTL and invalidation
- **AND** the system SHALL provide cache hit/miss metrics

### Requirement: Utility caching and rate limiting
The system SHALL implement caching and rate limiting across all utilities using Dapr State and Dapr components.

#### Scenario: Distributed caching
- **WHEN** caching utility results
- **THEN** the system SHALL use Dapr State for distributed caching
- **AND** the system SHALL support configurable TTL per utility
- **AND** the system SHALL provide cache invalidation mechanisms
- **AND** the system SHALL track cache performance metrics

#### Scenario: Rate limiting
- **WHEN** rate limiting utility calls
- **THEN** the system SHALL use Dapr components for rate limiting
- **AND** the system SHALL support per-provider rate limits
- **AND** the system SHALL handle rate limit exceeded gracefully
- **AND** the system SHALL provide rate limit monitoring

### Requirement: Maintain utility compatibility with Python PocketFlow
The system SHALL ensure Rust utilities produce functionally equivalent outputs to Python utilities for identical inputs, with potential improvements.

#### Scenario: Output compatibility
- **WHEN** processing the same input with Python and Rust utilities
- **THEN** the system SHALL produce functionally equivalent outputs
- **AND** the system SHALL handle edge cases and error conditions equivalently
- **AND** the system SHALL preserve configuration and option semantics
- **AND** the system MAY produce improved outputs (faster, more accurate) that are still functionally equivalent

#### Scenario: API compatibility
- **WHEN** using the Rust utility API
- **THEN** the system SHALL provide methods matching Python PocketFlow's utility API
- **AND** the system SHALL maintain similar method signatures where possible
- **AND** the system SHALL document differences due to Rust's type system
- **AND** the system SHALL prioritize working utility over nitpicking about exact output matching