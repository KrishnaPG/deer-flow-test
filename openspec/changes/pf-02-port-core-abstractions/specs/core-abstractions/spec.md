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

### Requirement: Node successors routing
The system SHALL implement node successor routing where each node tracks possible next nodes via action keys, enabling runtime path selection.

#### Scenario: Successors map
- **WHEN** a node is created
- **THEN** the node SHALL have a `successors: HashMap<Action, NodeId>` map
- **AND** successors SHALL map action keys (strings or enum variants) to target node IDs
- **AND** successors SHALL be set via a `next(node, action)` method

#### Scenario: Action-based routing
- **WHEN** a node completes execution (post method returns)
- **THEN** the return value SHALL be used as the action key
- **AND** the system SHALL lookup the next node from `node.successors.get(action)`
- **AND** if action not found in successors, the workflow SHALL terminate (no more nodes)

#### Scenario: Terminal nodes
- **WHEN** a node has no successors (successors is empty)
- **THEN** the node SHALL be considered terminal
- **AND** the workflow SHALL terminate after this node completes

#### Scenario: Operator overloading for graph construction
- **WHEN** constructing a workflow graph programmatically
- **THEN** the system SHALL support `>>` operator for sequential chaining (node_a >> node_b means action="default" goes to node_b)
- **AND** the system SHALL support `- "action"` operator for conditional branching (node - "search" >> target)
- **AND** the system SHALL use method chaining style for fluent graph construction

#### Scenario: _ConditionalTransition deferred binding
- **WHEN** using the `- "action"` operator
- **THEN** the system SHALL return a _ConditionalTransition helper object
- **AND** calling `>>` on the helper SHALL complete the binding: `node - "action" >> target`
- **AND** the helper stores src node and action until `>>` is called

#### Scenario: Node successor overwrite warning
- **WHEN** adding a successor with an action already defined
- **THEN** the system SHALL emit a warning about overwriting the existing successor
- **AND** the new successor SHALL replace the old one

### Requirement: Flow orchestration mechanics
The system SHALL implement flow orchestration via a distributed graph structure and an execution loop that iterates through nodes based on action routing.

#### Scenario: Distributed graph storage
- **WHEN** executing a flow
- **THEN** the flow SHALL only store `start_node_id` as the entry point
- **AND** each node SHALL store its own successors relationships
- **AND** the graph traversal SHALL dynamically look up next nodes via successors maps

#### Scenario: Execution loop (_orch pattern)
- **WHEN** executing a flow
- **THEN** the system SHALL iterate: current_node.execute() → returns Action → lookup next node
- **AND** the loop SHALL continue while current node exists and has successors
- **AND** the loop SHALL terminate when get_next_node() returns None
- **AND** the loop SHALL use `copy` semantics to avoid mutating original node state during traversal

#### Scenario: Loop via back-edge
- **WHEN** implementing workflow loops
- **THEN** the system SHALL support wiring a later node back to an earlier node
- **AND** the loop edge SHALL be defined at graph construction time (not runtime modification)
- **AND** the loop SHALL be represented as: `later_node - "action" >> earlier_node`
- **AND** the loop SHALL continue until a node returns an action that is not wired to the loop

#### Scenario: Conditional branching
- **WHEN** a node has multiple possible next nodes
- **THEN** the node SHALL support multiple edges with different action keys
- **AND** the system SHALL select the edge based on the node's return value (action)
- **AND** the branching SHALL be deterministic for the same input (same action → same edge)

### Requirement: Action type design
The system SHALL use a type-safe Action enum for routing decisions, with extensibility for custom actions.

#### Scenario: Action enum with known variants
- **WHEN** designing the Action type
- **THEN** the system SHALL define known action variants as enum members (e.g., `Continue`, `Retry`, `Done`, `Search`, `Answer`)
- **AND** the system SHALL provide a `Custom(&str)` variant for user-defined actions
- **AND** the system SHALL implement `Into<String>` and `TryFrom<String>` for serialization

#### Scenario: Default action
- **WHEN** a node returns no action or the default action
- **THEN** the system SHALL treat it as `Action::Continue` or the `"default"` key
- **AND** the system SHALL lookup `successors.get("default")` for the next node

### Requirement: Batch node semantics
The system SHALL support batch processing where a node processes multiple items sequentially or in parallel.

#### Scenario: BatchNode processing
- **WHEN** a BatchNode executes with a list of items
- **THEN** the system SHALL call exec once per item (iterating over the list)
- **AND** the system SHALL collect results into a list
- **AND** the system SHALL pass the list of results to post() method

#### Scenario: BatchNode prep
- **WHEN** BatchNode.prep() returns a list
- **THEN** exec SHALL be called once per item in the list
- **AND** the exec results SHALL be collected into a results list
- **AND** the results list SHALL be passed to post()

#### Scenario: BatchFlow processing
- **WHEN** a BatchFlow executes
- **THEN** the system SHALL call prep() to get a list of parameter sets
- **AND** the system SHALL run _orch() once for each parameter set
- **AND** each _orch() run SHALL use merged params (base params + per-run params)

### Requirement: Async node semantics
The system SHALL support async execution where nodes can pause and yield control during I/O operations.

#### Scenario: AsyncNode pipeline
- **WHEN** an AsyncNode executes
- **THEN** the system SHALL call prep_async(), then exec_async(), then post_async()
- **AND** async nodes SHALL be detected via `isinstance(curr, AsyncNode)` check
- **AND** the system SHALL use run_async() instead of run() for async nodes

#### Scenario: Async execution in mixed graphs
- **WHEN** a flow contains both sync and async nodes
- **THEN** sync nodes SHALL use `_run(shared)`
- **AND** async nodes SHALL use `_run_async(shared)`
- **AND** the flow SHALL handle both node types in the same orchestration loop

### Requirement: Nested flow as node
The system SHALL support treating a Flow as a node within another Flow, enabling hierarchical composition.

#### Scenario: Flow as sub-node
- **WHEN** a Flow is connected as a successor to another node
- **THEN** the Flow SHALL be treated as a single node in the parent graph
- **AND** the Flow's internal orchestration SHALL execute when the parent reaches it
- **AND** the Flow's return value SHALL be used for routing in the parent graph

#### Scenario: Nested flow parameter passing
- **WHEN** a parent flow calls a sub-flow
- **THEN** the sub-flow SHALL have access to the parent's shared state
- **AND** the sub-flow SHALL receive params merged from parent and per-run context

### Requirement: Node copy semantics during orchestration
The system SHALL use copy semantics when traversing nodes to avoid mutating original node state.

#### Scenario: Node copying in orchestration loop
- **WHEN** a flow orchestrates nodes
- **THEN** the system SHALL copy the start_node at loop initialization: `curr = copy.copy(start_node)`
- **AND** the system SHALL copy each next node after lookup: `curr = copy.copy(get_next_node(...))`
- **AND** original node instances SHALL NOT be mutated during traversal
- **AND** only the copied instance's params SHALL be modified via set_params()

#### Scenario: Why copy semantics matter
- **WHEN** understanding copy semantics
- **THEN** note that node state (params, successors) is preserved on the original node
- **AND** each traversal gets a fresh copy with initialized params
- **AND** this enables same node class to be reused in multiple flows

### Requirement: Node.run() vs Node._run() distinction
The system SHALL distinguish between standalone node execution and orchestrated execution.

#### Scenario: Standalone execution (run method)
- **WHEN** a node's run() method is called directly
- **THEN** the node SHALL execute prep → exec → post pipeline
- **AND** the node SHALL NOT follow successors (even if defined)
- **AND** a warning SHALL be emitted if successors are defined

#### Scenario: Orchestrated execution (_run method)
- **WHEN** a flow orchestrates a node via _run()
- **THEN** the flow's _orch() loop calls node._run(shared)
- **AND** node._run() returns action string for routing
- **AND** the action string determines next node

#### Scenario: AsyncNode _run raises error
- **WHEN** an AsyncNode's _run() is called
- **THEN** the system SHALL raise RuntimeError("Use run_async.")
- **AND** callers SHALL use _run_async() for async nodes

### Requirement: Node retry with cur_retry tracking
The system SHALL track retry attempts via an instance variable accessible to exec and exec_fallback.

#### Scenario: Retry loop instance variable
- **WHEN** a node executes with max_retries > 0
- **THEN** the system SHALL set self.cur_retry for each retry attempt (0, 1, 2, ...)
- **AND** exec() method SHALL read self.cur_retry to customize behavior
- **AND** exec_fallback(prep_res, exc) SHALL receive exception after final retry

### Requirement: AsyncFlow hybrid execution model
The system SHALL support executing both sync and async nodes within an AsyncFlow orchestration loop.

#### Scenario: AsyncFlow hybrid execution (sync + async nodes)
- **WHEN** an AsyncFlow orchestrates nodes
- **THEN** the system SHALL check if each node `isinstance(curr, AsyncNode)`
- **AND** async nodes SHALL use `_run_async(shared)`
- **AND** sync nodes SHALL use `_run(shared)`
- **AND** the same orchestration loop SHALL handle both types seamlessly

#### Scenario: AsyncFlow loop termination
- **WHEN** get_next_node() returns None
- **THEN** the orchestration loop SHALL terminate
- **AND** the last_action SHALL be returned

#### Scenario: AsyncParallelBatchNode parallel execution
- **WHEN** an AsyncParallelBatchNode executes
- **THEN** the system SHALL use asyncio.gather() to execute items in parallel
- **AND** all items SHALL be processed concurrently
- **AND** results SHALL be collected into a list

#### Scenario: AsyncParallelBatchFlow parallel flow replay
- **WHEN** an AsyncParallelBatchFlow executes
- **THEN** the system SHALL use asyncio.gather() to run flows in parallel
- **AND** each flow SHALL receive merged params
- **AND** all flows SHALL complete before returning

### Requirement: Maintain compatibility with Python PocketFlow semantics
The system SHALL ensure Rust implementation provides functional equivalence with Python PocketFlow core abstractions, improving upon Python where beneficial.

#### Scenario: Behavioral compatibility
- **WHEN** executing the same workflow in Python and Rust
- **THEN** the system SHALL produce functionally equivalent results for identical inputs
- **AND** the system SHALL handle edge cases (empty shared store, missing parameters, etc.) equivalently
- **AND** the system SHALL preserve warning and error messages where applicable
- **AND** the system MAY produce different (improved) internal behavior that achieves the same outcome

#### Scenario: API compatibility
- **WHEN** using the Rust API
- **THEN** the system SHALL provide methods matching Python PocketFlow's public API
- **AND** the system SHALL maintain similar method signatures where possible
- **AND** the system SHALL document differences due to Rust's type system
- **AND** the system SHALL prioritize getting to working state over 100% line equality

#### Scenario: Performance improvements
- **WHEN** implementing core abstractions
- **THEN** the system SHALL aim for zero-copy, zero-allocation where possible
- **AND** the system SHALL achieve lower latency than Python implementation
- **AND** the system SHALL leverage Rust's type safety for compile-time error detection
- **AND** the system SHALL add built-in retry/fallback that Python requires external libraries for