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

### 1. Node Mapping: Activities, Actors, and Fast-Paths
**Decision**: Use Dapr Workflow Activities for stateless nodes and Dapr Actors for stateful nodes requiring persistent identity. Introduce a **"Local Fast-Path" (Node Clustering)** where multiple consecutive purely compute-bound, stateless nodes are executed locally in memory as a single Dapr Activity.
**Rationale**: Activities match the prep→exec→post pipeline; Actors provide turn-based concurrency. The Fast-Path optimization prevents Dapr orchestration roundtrips for extremely fast, consecutive Rust operations (e.g., chunking, formatting).
**Alternatives Considered**: Always Activities for everything (rejected - massive distributed overhead for trivial compute steps), always Actors (rejected - overkill for stateless nodes).

### 2. Flow Mapping: Dapr Workflows
**Decision**: Map each PocketFlow Flow to a Dapr Workflow definition.
**Rationale**: Dapr Workflows provide durable orchestration, built-in retry, compensation, and parallel execution.
**Alternatives Considered**: Custom orchestration (rejected - reinventing wheel), external scheduler (rejected - adds complexity).

### 3. SharedStore: Dapr State Management
**Decision**: Wrap Dapr State Management API in a `SharedStore` trait with key-value semantics.
**Rationale**: Provides pluggable backends (Redis, PostgreSQL, etc.), transactional updates, and durability.
**Alternatives Considered**: In-memory only (rejected - no persistence), custom storage (rejected - reinventing wheel).

### 4. Params: Immutable Workflow Input, Payload Pointers, & Zero-Copy
**Decision**: Pass `Params` as immutable workflow input data. For large payloads (embeddings, huge document arrays), store them in a Dapr-backed BlobStore and only pass references/pointers through the workflow. For local fast-paths, wrap data in `Arc<T>` to ensure zero-copy memory sharing. Additionally, mandate **multiplexed gRPC** for Dapr sidecar communication using binary serialization (like Protocol Buffers or MessagePack via `prost`) instead of HTTP/JSON.
**Rationale**: Matches Python's immutable params logically while avoiding catastrophic memory and latency spikes. `Arc<T>` gives us zero-copy memory speeds locally, and gRPC + Protobuf minimizes sidecar overhead for distributed steps.
**Alternatives Considered**: Full JSON over HTTP (rejected - unacceptable latency and memory overhead), Mutable shared params (rejected - breaks compatibility).

### 5. Retry Logic: Dapr Retry Policies
**Decision**: Use Dapr Workflow's `ActivityRetryPolicy` for node retries, with custom retry logic in `exec_fallback`.
**Rationale**: Leverages Dapr's built-in retry with exponential backoff, jitter, and max attempts.
**Alternatives Considered**: Custom retry implementation (rejected - reinventing wheel), no retry (rejected - enterprise requirement).

### 6. Fallback: Action-Based Semantic Fallbacks
**Decision**: Use Dapr Retries exclusively for *technical* failures (e.g., HTTP 500s, network timeouts). For *semantic* fallbacks (e.g., LLM provider down after max retries, switch to alternative provider), the node catches the error and returns a specific `Action::Fallback` enum. PocketFlow then routes to the backup node naturally.
**Rationale**: Dapr Compensation Handlers are designed for Saga-style state rollbacks, not forward-moving logic alternatives. Action-based semantic routing accurately maps to PocketFlow's dynamic graph traversal without fighting the orchestrator.
**Alternatives Considered**: Dapr Compensation Handlers for logic fallbacks (rejected - misuses Dapr primitives), Custom error handling (rejected - inconsistent).

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

### 10. Domain-Agnostic Orchestration (Decoupling from LLM)
**Decision**: The core `Node`, `Flow`, and orchestration mechanisms must be completely abstracted away from LLM-specific logic. A `Node` is a generic unit of durable work (e.g., business logic validation, database update, legacy API request, or LLM call). The `Action` enums returned by nodes dictate control flow irrespective of the domain. 

To enforce this strict boundary, the architecture is split into domain-specific crates that can be used independently:
*   **`berg10-execution-engine`**: The core, domain-agnostic orchestration engine containing the `Node` trait, `Flow` orchestrator, generic DAG traversal logic, and design patterns. Zero AI/LLM dependencies. It can run entirely standalone as a local execution engine.
*   **`berg10-dapr-driver`**: A swappable execution driver that bridges the generic `execution-engine` with the Dapr SDK for distributed durability. 
*   **`berg10-ai-driver`**: The LLM/AI specific implementations that implement the `Node` trait for AI tasks (e.g., generation, retrieval).

**Rationale**: An enterprise workflow engine must not be bottlenecked to a single domain. PocketFlow should be capable of orchestrating generic microservices, data pipelines, and legacy systems just as seamlessly as AI agents. This maximizes the reuse of the Rust+Dapr foundation and allows the execution engine to be embedded anywhere.
### 11. Graph Persistence: Dapr State Management
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

