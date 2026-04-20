## Context

PocketFlow-Rust provides a foundation with Node trait (`prepare → execute → post_process`), Flow struct (state-based transitions), Context (key-value store), and Params (`HashMap<String, serde_json::Value>`). However, it lacks Dapr integration and enterprise features like retry, fallback, and parallel execution. This design maps PocketFlow concepts to Dapr building blocks to achieve durable, scalable, and observable workflows.

We are taking a **fresh start approach**: use Python PocketFlow as inspiration ("in spirit" and "logic"), not as a strict line-by-line template. The Rust implementation should do everything Python can do, but better - faster, lower latency, zero-copy, zero-allocation where possible, with built-in enterprise features that Python lacks.

## Goals / Non-Goals

**Goals:**
- Map Node abstraction to Dapr Workflow Activities (stateless) or Actors (stateful)
- Map Flow abstraction to Dapr Workflows for orchestration
- Map SharedStore to Dapr State Management with pluggable backends
- Map Params to immutable workflow input data
- Add retry logic using Dapr retry policies
- Add fallback mechanisms using Dapr compensation handlers
- Add parallel execution using Dapr fan-out/fan-in patterns
- Provide functional equivalence with Python PocketFlow semantics
- Prioritize getting to working state quickly over nitpicking about 100% line equality
- Aim for zero-copy, zero-allocation where possible for performance
- Implement action-based routing where nodes return action enums that control graph traversal
- Support loop construction via back-edges (wiring later node back to earlier node)

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Redesign PocketFlow's core semantics in ways that break functional equivalence
- Support all Python dynamic features that don't translate to Rust
- Replace existing PocketFlow-Rust code wholesale
- Achieve line-by-line code equivalence with Python (not required)
- Implement runtime node addition/removal (graph is static at construction, traversal path is dynamic)

## Decisions

### 1. Node Mapping: Activities vs Actors
**Decision**: Use Dapr Workflow Activities for stateless nodes, Dapr Actors for stateful nodes requiring persistent identity.
**Rationale**: Activities match the prep→exec→post pipeline; Actors provide turn-based concurrency and state for long-running processes.
**Alternatives Considered**: Always Activities (rejected - no persistent state), always Actors (rejected - overkill for stateless nodes).

### 2. Flow Mapping: Dapr Workflows
**Decision**: Map each PocketFlow Flow to a Dapr Workflow definition.
**Rationale**: Dapr Workflows provide durable orchestration, built-in retry, compensation, and parallel execution.
**Alternatives Considered**: Custom orchestration (rejected - reinventing wheel), external scheduler (rejected - adds complexity).

### 3. SharedStore: Dapr State Management
**Decision**: Wrap Dapr State Management API in a `SharedStore` trait with key-value semantics.
**Rationale**: Provides pluggable backends (Redis, PostgreSQL, etc.), transactional updates, and durability.
**Alternatives Considered**: In-memory only (rejected - no persistence), custom storage (rejected - reinventing wheel).

### 4. Params: Immutable Workflow Input
**Decision**: Pass Params as serializable workflow input data, immutable per node.
**Rationale**: Matches Python's immutable params per node, simplifies serialization.
**Alternatives Considered**: Mutable shared params (rejected - breaks compatibility), separate param store (rejected - adds complexity).

### 5. Retry Logic: Dapr Retry Policies
**Decision**: Use Dapr Workflow's `ActivityRetryPolicy` for node retries, with custom retry logic in `exec_fallback`.
**Rationale**: Leverages Dapr's built-in retry with exponential backoff, jitter, and max attempts.
**Alternatives Considered**: Custom retry implementation (rejected - reinventing wheel), no retry (rejected - enterprise requirement).

### 6. Fallback: Dapr Compensation Handlers
**Decision**: Implement node's `exec_fallback` as Dapr Workflow compensation handler.
**Rationale**: Aligns with Dapr's compensation pattern for error recovery.
**Alternatives Considered**: Custom error handling (rejected - inconsistent), ignore fallback (rejected - enterprise requirement).

### 7. Parallel Execution: Dapr Fan-out/Fan-in
**Decision**: Use Dapr Workflow's `when_all` for parallel activities, `for_each` with parallelism limit for batch processing.
**Rationale**: Native Dapr support for parallel execution with durability.
**Alternatives Considered**: Custom thread pool (rejected - no durability), sequential execution (rejected - performance).

### 8. Graph Architecture: Distributed Successors Map
**Decision**: Model PocketFlow's graph as a distributed structure where each node has a `successors: HashMap<Action, NodeId>` map, with action-based routing at runtime.
**Rationale**: This captures PocketFlow's core "dynamic" mechanic: the graph structure is static, but which edge is taken is determined at runtime by the node's return value.

**PocketFlow Internals** (for reference when implementing):
```
BaseNode:
  - params: Dict[str, Any]           # Node parameters
  - successors: Dict[str, Node]       # action -> next node mapping
  
  - next(node, action="default")      # Add edge to successors
  - __rshift__(other) -> next(other) # Sequential: node >> next_node
  - __sub__(action) -> _ConditionalTransition  # For conditional: node - "action" >> target

Flow:
  - start_node: Node                  # Entry point only
  - get_next_node(curr, action) -> lookup curr.successors.get(action)
  - _orch():                         # Core loop:
    while curr:
        curr.set_params(p)
        last_action = curr._run(shared)  # prep -> exec -> post, returns action
        curr = copy(curr.get_next_node(curr, last_action))
```

**Action Type Decision**: Use `enum Action` with variants for known actions, and a catch-all `Custom(&str)` for extensibility. This provides:
- Type safety (compile-time checking for known actions)
- Extensibility (custom actions via `Custom("search")`)
- Pattern matching for routing decisions

**Routing Behavior**:
- `post()` method returns `Action` enum
- `get_next_node()` does: `node.successors.get(action).cloned()`
- If action not in successors and successors is non-empty: warn and return None (flow ends)
- If successors is empty: current node is terminal

### 9. Graph Persistence: Dapr State Management
**Decision**: Store graph structure (nodes and successors relationships) in Dapr State Management for persistence and recovery.
**Rationale**: Enables workflow restart from persisted state, version history for auditing, and external graph modification tools.

**Storage Schema** (to be refined in implementation):
```rust
struct WorkflowGraph {
    id: WorkflowId,
    version: u64,
    nodes: Vec<NodeState>,      // Each node's state including its successors
    start_node_id: NodeId,
}

struct NodeState {
    id: NodeId,
    node_type: NodeType,
    params: Params,
    successors: HashMap<Action, NodeId>,  // action -> target node
}
```

## Risks / Trade-offs

**Risk**: Dapr SDK for Rust may be immature → Mitigation: Use stable APIs, contribute upstream if needed.
**Risk**: Serialization overhead with `serde_json::Value` → Mitigation: Provide type-safe wrappers, optimize hot paths.
**Risk**: Behavioral differences between Python and Rust/Dapr → Mitigation: Comprehensive compatibility tests.
**Risk**: Dapr sidecar adds operational complexity → Mitigation: Leverage existing Dapr deployment patterns.

## Migration Plan

1. Create crate structure with trait definitions
2. Implement Dapr Activity node wrapper
3. Implement Dapr Workflow flow wrapper
4. Implement Dapr State Management SharedStore
5. Add retry, fallback, parallel execution features
6. Create compatibility test suite
7. Document migration path from Python PocketFlow

## Open Questions

1. Should we support both Dapr Workflow and Actor models for nodes, or choose one?
2. How to handle Python's dynamic typing vs Rust's static typing?
3. What serialization format to use for cross-language compatibility?
4. How to integrate Dapr's OpenTelemetry with PocketFlow tracing?
5. Should we support non-Dapr backends for development/testing?