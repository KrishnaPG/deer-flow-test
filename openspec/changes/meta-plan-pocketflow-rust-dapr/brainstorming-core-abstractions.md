# Brainstorming: Core Abstractions Dapr Mapping

## Current State (PocketFlow-Rust)
- **Node trait**: `prepare → execute → post_process` async pipeline
- **Flow struct**: State-based transitions with enums
- **Context**: Key-value store (in-memory)
- **Params**: `HashMap<String, serde_json::Value>`
- **Missing**: retry, fallback, parallel execution

## Dapr Building Blocks
- **Dapr Workflow**: Orchestrates business logic with durable execution
- **Dapr Workflow Activity**: Stateless function called by workflow
- **Dapr Actor**: Stateful virtual actor with identity
- **Dapr State Management**: Key-value storage with pluggable backends
- **Dapr Pub/Sub**: Messaging between components
- **Dapr Bindings**: Input/output connectors to external systems

## Mapping Decisions

### 1. Node → Dapr Activity vs Actor
**Option A: Stateless Activities**
- **Pros**: Simple, matches prep→exec→post pipeline, Dapr handles retries
- **Cons**: No persistent state between invocations, need external state store

**Option B: Stateful Actors**
- **Pros**: Persistent state, identity, turn-based concurrency
- **Cons**: More complex, may be overkill for stateless nodes

**Decision**: Use **Activities** for stateless nodes, **Actors** for nodes requiring persistent state (e.g., long-running processes, stateful agents). Provide a trait that can be implemented as either.

### 2. Flow → Dapr Workflow
- **Mapping**: Each PocketFlow Flow becomes a Dapr Workflow definition
- **Transitions**: Use Dapr Workflow's `continue_as_new` for loops, `call_activity` for node execution, `when_all` for parallel execution
- **Conditional branching**: Use workflow `switch` statements based on node output

### 3. SharedStore → Dapr State Management
- **Implementation**: Wrap Dapr state store API in a `SharedStore` trait
- **Key structure**: Use composite keys: `{workflow_id}/{node_id}/{key}`
- **Consistency**: Use Dapr's transactional state operations for atomic updates

### 4. Params → Workflow Input
- **Passing**: Params passed as workflow input data (serializable)
- **Immutable**: Params are read-only for nodes, can be overridden per node

### 5. Retry Logic
- **Dapr Workflow retry**: Use `ActivityRetryPolicy` for node retries
- **Custom retry**: For complex retry logic (exponential backoff, jitter), implement in node's `exec_fallback`

### 6. Fallback Mechanisms
- **Dapr compensation**: Use workflow compensation handlers
- **Custom fallback**: Node's `exec_fallback` called on failure, can trigger alternative workflow path

### 7. Parallel Execution
- **Fan-out/fan-in**: Use Dapr Workflow's `when_all` for parallel activities
- **Batch processing**: Use `for_each` with parallelism limit

## Architecture Sketch

```
┌─────────────────────────────────────────────────────────┐
│                 PocketFlow-Rust + Dapr                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Node      │    │   Flow      │    │  SharedStore│ │
│  │  (Activity) │    │ (Workflow)  │    │ (State Mgmt)│ │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘ │
│         │                  │                  │         │
│         ▼                  ▼                  ▼         │
│  ┌─────────────────────────────────────────────────────┐│
│  │              Dapr Sidecar (daprd)                  ││
│  └─────────────────────────────────────────────────────┘│
│         │                  │                  │         │
│         ▼                  ▼                  ▼         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  Workflow   │    │   State     │    │   Pub/Sub   │ │
│  │   Engine    │    │   Store     │    │   Broker    │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Crate Structure

```
crates/berg10/execution-engine/
├── src/
│   ├── lib.rs
│   ├── node.rs          # Node trait + Activity/Actor implementations
│   ├── flow.rs          # Flow struct + Workflow definition
│   ├── context.rs       # SharedStore trait + Dapr state implementation
│   ├── params.rs        # Params type
│   ├── retry.rs         # Retry policies
│   ├── error.rs         # Error types
│   └── dapr/            # Dapr-specific implementations
│       ├── activity.rs  # Dapr Activity node
│       ├── actor.rs     # Dapr Actor node
│       ├── workflow.rs  # Dapr Workflow flow
│       └── state.rs     # Dapr State store
├── Cargo.toml
└── AGENTS.md
```

## Open Questions

1. **Serialization**: Use `serde_json::Value` or typed structs? Recommendation: Use generic `serde_json::Value` for flexibility, but provide type-safe wrappers.

2. **Error handling**: How to map PocketFlow errors to Dapr errors? Need custom error enum.

3. **Idempotency**: Dapr activities must be idempotent. How to ensure? Use deterministic node IDs.

4. **Observability**: How to integrate Dapr's OpenTelemetry with PocketFlow tracing?

5. **Testing**: How to test Dapr integration without running actual Dapr sidecar? Use mock implementations.

## Next Steps

1. Create OpenSpec change for core abstractions porting plan
2. Define detailed specs for each mapping
3. Design Rust API with examples
4. Plan implementation phases