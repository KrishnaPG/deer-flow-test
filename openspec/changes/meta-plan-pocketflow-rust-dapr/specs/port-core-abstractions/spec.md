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

### Requirement: Port Params to Rust
The system SHALL port the PocketFlow Params abstraction to Rust as immutable parameters passed to nodes.

#### Scenario: Parameter passing
- **WHEN** passing parameters to nodes
- **THEN** the system SHALL support immutable parameter dictionaries
- **AND** the system SHALL support parameter inheritance from parent flows
- **AND** the system SHALL preserve the set_params method semantics

### Requirement: Maintain compatibility with Python PocketFlow semantics
The system SHALL ensure Rust implementation preserves the exact semantics of Python PocketFlow core abstractions.

#### Scenario: Behavioral compatibility
- **WHEN** executing the same workflow in Python and Rust
- **THEN** the system SHALL produce identical results for identical inputs
- **AND** the system SHALL handle edge cases (empty shared store, missing parameters, etc.) identically
- **AND** the system SHALL preserve warning and error messages where applicable