## ADDED Requirements

### Requirement: Port Node abstraction to Rust with custom orchestrator
The system SHALL port the PocketFlow Node abstraction to Rust using a custom orchestrator for dynamic graph execution. Dapr is used ONLY for durability/state management, NEVER for orchestration or Node execution.

#### Scenario: Node trait definition
- **WHEN** defining the Node trait in Rust
- **THEN** the system SHALL provide `exec()` for fresh execution
- **AND** the system SHALL provide `resume(state)` for recovery from checkpoint
- **AND** the system SHALL return `Action` for orchestrator routing
- **AND** the orchestrator SHALL be agnostic to node internals

#### Scenario: Node execution
- **WHEN** the orchestrator calls `node.exec()`
- **THEN** the node SHALL execute its logic
- **AND** the node SHALL have access to SharedStore for state
- **AND** the node SHALL return an Action for routing

#### Scenario: Node resumption
- **WHEN** the orchestrator calls `node.resume(state)`
- **THEN** the node SHALL restore from the opaque state blob
- **AND** the node SHALL continue execution from where it left off
- **AND** the orchestrator SHALL NOT interpret the state content

#### Scenario: Composite nodes (Flow-as-Node)
- **WHEN** a node is a composite (Flow)
- **THEN** the node SHALL spawn an inner orchestrator
- **AND** the inner orchestrator SHALL manage its own execution stack
- **AND** the node SHALL serialize its inner state for resumption
- **AND** the outer orchestrator SHALL remain agnostic

#### Scenario: Dapr durability integration
- **WHEN** using DaprDurability for state persistence
- **THEN** the system SHALL use Dapr State Store for checkpoint persistence
- **AND** the system SHALL use Dapr Pub/Sub for multi-agent communication when needed
- **AND** the system SHALL auto-detect Dapr sidecar presence and fail fast if missing

### Requirement: Port Flow abstraction to Rust with custom orchestrator
The system SHALL port the PocketFlow Flow abstraction to Rust using the custom orchestrator. Flow is-a Node (implements Node trait) enabling hierarchical composition.

#### Scenario: Flow orchestration
- **WHEN** orchestrating nodes with the custom orchestrator
- **THEN** the system SHALL support sequential execution via action-based routing
- **AND** the system SHALL support conditional transitions via action keys
- **AND** the system SHALL support loops via back-edges (later node → earlier node)

#### Scenario: Nested flows
- **WHEN** implementing nested flows
- **THEN** the system SHALL support Flow-as-Node composition
- **AND** the system SHALL preserve parameter passing between parent and child flows
- **AND** the system SHALL support hierarchical checkpoints for nested flows

#### Scenario: Parallel execution
- **WHEN** executing nodes in parallel
- **THEN** the system SHALL use Rust async/await with tokio::join! or futures::join_all
- **AND** the system SHALL support configurable parallelism limits
- **AND** the system SHALL maintain durability through checkpointing

### Requirement: SharedStore with durability
The system SHALL implement a durable SharedStore for workflow state that persists with every checkpoint.

#### Scenario: SharedStore API
- **WHEN** implementing SharedStore
- **THEN** the system SHALL provide simple key-value operations: get, put, delete
- **AND** the system SHALL NOT provide atomic transactions (not needed)
- **AND** the system SHALL support any serializable value type

#### Scenario: SharedStore durability
- **WHEN** checkpointing workflow state
- **THEN** the SharedStore SHALL be serialized and persisted with the checkpoint
- **AND** the SharedStore SHALL be restored on recovery
- **AND** all durability backends SHALL support SharedStore persistence

#### Scenario: SharedStore implementations
- **WHEN** configuring SharedStore
- **THEN** InMemoryDurability SHALL use in-memory HashMap
- **AND** ReDBDurability SHALL use ReDB key-value store
- **AND** DaprDurability SHALL use Dapr State Management

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
The system SHALL implement retry logic and fallback mechanisms with clear separation between technical retries (infrastructure) and semantic fallbacks (business logic).

#### Scenario: Technical retry (Dapr resiliency)
- **WHEN** a node encounters technical failures (network, HTTP 5xx, timeouts)
- **THEN** the system SHALL retry using Dapr resiliency policies when DaprDurability is used
- **AND** the system SHALL use exponential backoff with jitter
- **AND** local durability implementations SHALL provide equivalent retry logic
- **AND** the system SHALL distinguish technical failures from business logic failures

#### Scenario: Semantic fallback (Action-based)
- **WHEN** a node fails after max_retries or encounters business logic failure
- **THEN** the system SHALL execute the node's exec_fallback method
- **AND** exec_fallback SHALL return an Action indicating the fallback path
- **AND** the orchestrator SHALL route to the appropriate successor based on the Action
- **AND** fallback SHALL support both recovery paths and alternative workflow branches

#### Scenario: Retry configuration
- **WHEN** configuring retry behavior
- **THEN** the system SHALL support max_retries and wait parameters per node
- **AND** retry policies SHALL be configurable via central config module
- **AND** the system SHALL expose cur_retry counter for custom retry logic

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

### Requirement: Stack-based orchestrator
The system SHALL implement a stack-based orchestrator that executes nodes using DFS traversal. The orchestrator is agnostic to node types and nesting depth.

#### Scenario: Orchestrator execution loop
- **WHEN** executing a flow
- **THEN** the orchestrator SHALL maintain an execution stack of Frames
- **AND** the loop SHALL: pop Frame → resolve Node → execute/resume → push next Frame
- **AND** the loop SHALL continue until the stack is empty
- **AND** the orchestrator SHALL NOT know or care about node semantics (simple vs composite)

#### Scenario: Frame structure
- **WHEN** creating a Frame for the execution stack
- **THEN** the Frame SHALL contain: `node_id` and optional `state` (opaque blob)
- **AND** if `state` is Some, the orchestrator SHALL call `node.resume(state)`
- **AND** if `state` is None, the orchestrator SHALL call `node.exec()`
- **AND** the orchestrator SHALL NOT interpret the state content

#### Scenario: Checkpoint after every transition
- **WHEN** a node completes execution
- **THEN** the system SHALL persist a checkpoint BEFORE proceeding to next node
- **AND** the checkpoint SHALL contain the current stack and SharedStore
- **AND** checkpoints SHALL be append-only with history
- **AND** recovery SHALL load the latest checkpoint from history

#### Scenario: Loop via back-edge
- **WHEN** implementing workflow loops
- **THEN** the system SHALL support wiring a later node back to an earlier node
- **AND** the loop SHALL be represented as: `later_node - Action::Continue >> earlier_node`
- **AND** the loop SHALL continue until a node returns an action not wired to the loop

#### Scenario: Conditional branching
- **WHEN** a node has multiple possible next nodes
- **THEN** the node SHALL support multiple edges with different Action keys
- **AND** the system SHALL select the edge based on the node's return Action
- **AND** the branching SHALL be deterministic (same Action → same edge)

### Requirement: Action type design (Performance-First)
The system SHALL use a type-safe Action enum for routing decisions. NO string-based actions are supported - only enum variants for zero-cost performance.

#### Scenario: Action enum definition
- **WHEN** defining the Action type
- **THEN** the system SHALL use enum variants only (no strings)
- **AND** the enum SHALL include: `Default`, `Done`, `Retry`, `Continue`, `Fallback`
- **AND** the system SHALL provide `Custom(u64)` for user-defined actions
- **AND** Action SHALL be `#[repr(u8)]` for compact representation
- **AND** Action SHALL implement `Copy` trait (no allocations)

#### Scenario: Default action
- **WHEN** a node completes without explicit action
- **THEN** the system SHALL use `Action::Default`
- **AND** the orchestrator SHALL lookup successors using `Action::Default`

#### Scenario: Successors mapping
- **WHEN** mapping actions to successor nodes
- **THEN** successors SHALL use `Action` as the key type directly
- **AND** the map SHALL be `HashMap<Action, NodeId>` for O(1) lookup

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