## ADDED Requirements

### Requirement: Port Node abstraction to Rust with Dapr
The system SHALL port the PocketFlow Node abstraction to Rust using Dapr Workflow Activities for stateless execution or Dapr Actors for stateful execution.

#### Scenario: Node trait definition
- **WHEN** defining the Node trait in Rust
- **THEN** the system SHALL include prep, exec, and post methods matching Python Node behavior
- **AND** the system SHALL support retry logic with max_retries and wait parameters
- **AND** the system SHALL support exec_fallback for error handling

#### Scenario: Dapr Activity mapping
- **WHEN** mapping Node to Dapr Activity
- **THEN** the system SHALL preserve the prep→exec→post pipeline semantics
- **AND** the system SHALL use Dapr retry policies for node retries
- **AND** the system SHALL handle failures through Dapr's built-in error handling

#### Scenario: Dapr Actor mapping
- **WHEN** mapping Node to Dapr Actor for stateful execution
- **THEN** the system SHALL maintain actor state across invocations
- **AND** the system SHALL support actor turn-based concurrency
- **AND** the system SHALL preserve node identity via actor ID

### Requirement: Port Flow abstraction to Rust with Dapr
The system SHALL port the PocketFlow Flow abstraction to Rust using Dapr Workflows for orchestration.

#### Scenario: Flow orchestration
- **WHEN** orchestrating nodes with Dapr Workflow
- **THEN** the system SHALL support sequential execution via workflow activities
- **AND** the system SHALL support conditional transitions via workflow branching
- **AND** the system SHALL support loops via workflow continue signals

#### Scenario: Nested flows
- **WHEN** implementing nested flows
- **THEN** the system SHALL support sub-workflows in Dapr
- **AND** the system SHALL preserve parameter passing between parent and child flows

#### Scenario: Parallel execution
- **WHEN** executing nodes in parallel
- **THEN** the system SHALL use Dapr Workflow's when_all for fan-out/fan-in
- **AND** the system SHALL support configurable parallelism limits
- **AND** the system SHALL maintain durability across parallel executions

### Requirement: Port SharedStore to Rust with Dapr State Management
The system SHALL port the PocketFlow SharedStore to Rust using Dapr State Management for persistence.

#### Scenario: State persistence
- **WHEN** storing shared state
- **THEN** the system SHALL use Dapr State Management with pluggable backends (Redis, PostgreSQL, etc.)
- **AND** the system SHALL support concurrent access with appropriate locking mechanisms
- **AND** the system SHALL preserve the key-value semantics of Python's shared dictionary

#### Scenario: State recovery
- **WHEN** recovering from failures
- **THEN** the system SHALL persist state after each node execution
- **AND** the system SHALL support recovery from the last persisted state
- **AND** the system SHALL maintain state consistency across workflow retries

#### Scenario: Transactional updates
- **WHEN** updating multiple state keys atomically
- **THEN** the system SHALL use Dapr's transactional state operations
- **AND** the system SHALL ensure atomicity across all keys in the transaction

### Requirement: Port Params to Rust
The system SHALL port the PocketFlow Params abstraction to Rust as immutable parameters passed to nodes.

#### Scenario: Parameter passing
- **WHEN** passing parameters to nodes
- **THEN** the system SHALL support immutable parameter dictionaries
- **AND** the system SHALL support parameter inheritance from parent flows
- **AND** the system SHALL preserve the set_params method semantics

#### Scenario: Serialization
- **WHEN** serializing parameters for Dapr workflow input
- **THEN** the system SHALL use serde_json::Value for flexibility
- **AND** the system SHALL provide type-safe wrappers for common parameter types
- **AND** the system SHALL support cross-language compatibility with Python

### Requirement: Retry and fallback mechanisms
The system SHALL implement retry logic and fallback mechanisms using Dapr capabilities.

#### Scenario: Retry with exponential backoff
- **WHEN** a node fails and max_retries > 0
- **THEN** the system SHALL retry with exponential backoff using Dapr retry policies
- **AND** the system SHALL respect the wait parameter between retries
- **AND** the system SHALL log retry attempts for observability

#### Scenario: Fallback execution
- **WHEN** a node fails after max_retries
- **THEN** the system SHALL execute the node's exec_fallback method
- **AND** the system SHALL treat fallback as a compensation action
- **AND** the system SHALL allow fallback to trigger alternative workflow paths

### Requirement: Maintain compatibility with Python PocketFlow semantics
The system SHALL ensure Rust implementation preserves the exact semantics of Python PocketFlow core abstractions.

#### Scenario: Behavioral compatibility
- **WHEN** executing the same workflow in Python and Rust
- **THEN** the system SHALL produce identical results for identical inputs
- **AND** the system SHALL handle edge cases (empty shared store, missing parameters, etc.) identically
- **AND** the system SHALL preserve warning and error messages where applicable

#### Scenario: API compatibility
- **WHEN** using the Rust API
- **THEN** the system SHALL provide methods matching Python PocketFlow's public API
- **AND** the system SHALL maintain similar method signatures where possible
- **AND** the system SHALL document differences due to Rust's type system