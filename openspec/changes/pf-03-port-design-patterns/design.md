## Context

PocketFlow-Rust lacks design patterns (Agent, Map-Reduce, RAG, Multi-Agent, Supervisor). This design maps each pattern to Dapr building blocks to achieve durable, scalable, and observable pattern implementations. The patterns build on the core abstractions (Node, Flow, SharedStore, Params) ported in the previous change.

We are taking a **fresh start approach**: use Python PocketFlow patterns as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust implementation should do everything Python patterns can do, but better - with built-in enterprise features that Python lacks.

## Goals / Non-Goals

**Goals:**
- Map Agent pattern to Dapr Actors for stateful agents with Dapr Activities for actions
- Map Map-Reduce pattern to Dapr Workflow fan-out/fan-in with parallel activities
- Map RAG pattern to Dapr Bindings for vector DB retrieval and Activities for generation
- Map Multi-Agent pattern to Dapr Actors for agents with Pub/Sub for communication
- Map Supervisor pattern to Dapr Workflow loops with evaluation activities
- Provide functional equivalence with Python PocketFlow pattern semantics
- Prioritize working implementation over nitpicking about 100% line equality
- Add Dapr-native resilience (retry, circuit breaker, fallback) that Python patterns lack

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Redesign pattern semantics in ways that break functional equivalence
- Support all Python dynamic features that don't translate to Rust
- Replace existing PocketFlow-Rust examples wholesale
- Achieve line-by-line code equivalence with Python (not required)

## Decisions

### 1. Agent Pattern: Stateless Routers + Sharded Actors
**Decision**: Implement agent execution and routing as stateless Dapr Activities (or Workflow logic) to ensure high-throughput fan-in without locking. Use Dapr Actors *only* when strict sequential ordering of state changes is required (e.g., appending to a specific conversation history). For highly concurrent stateful agents, rely on Dapr State with optimistic concurrency control (ETags) or Actor sharding.
**Rationale**: Defaulting to Dapr Actors for all agents bottlenecks the system due to strict single-threaded (turn-based) concurrency. Stateless routing prevents this bottleneck while maintaining functional equivalence.
**Alternatives Considered**: Always Actors for agents (rejected - causes severe throughput bottlenecks for popular agents/supervisors), always Activities (rejected - no persistent state).

### 2. Map-Reduce Pattern: Batched Workflow Fan-out/Fan-in
**Decision**: Use Dapr Workflow's `for_each` but strictly implement **Batched Chunking Fan-Out**. If a map operation generates 10,000 sub-tasks, they are processed and aggregated in safe chunks (e.g., 100) to prevent OOM errors in the workflow state.
**Rationale**: Native Dapr support provides parallel execution with durability, but naive fan-outs of large datasets will crash sidecar memory limits. Batching guarantees high throughput scaling.
**Alternatives Considered**: Unbounded parallel execution (rejected - causes memory exhaustion), sequential processing (rejected - performance).

### 3. RAG Pattern: Async Streaming Bindings
**Decision**: Use Dapr vector DB binding for retrieval, LLM activity for generation, but enforce `Stream` traits. Do not block generation waiting for the entire context to load; stream retrieved chunks directly.
**Rationale**: Bindings provide pluggable vector stores; streaming reduces time-to-first-byte (TTFB) significantly for heavy RAG workloads.
**Alternatives Considered**: Blocking retrieval (rejected - unacceptable latency), integrated RAG activity (rejected - less flexible).

### 4. Multi-Agent Pattern: Actors + Pub/Sub
**Decision**: Implement agents as Dapr Actors, communication via Dapr Pub/Sub topics.
**Rationale**: Actors maintain agent state; Pub/Sub enables decoupled asynchronous messaging.
**Alternatives Considered**: Direct workflow calls (rejected - tight coupling), shared state only (rejected - no async communication).

### 5. Supervisor Pattern: Workflow Loop
**Decision**: Implement supervisor as Dapr Workflow with evaluation activity and conditional looping.
**Rationale**: Workflows provide durable loops with compensation and retry.
**Alternatives Considered**: Actor with timer (rejected - less durable), custom loop (rejected - reinventing wheel).

### 6. Pattern Composition
**Decision**: Patterns can be composed (e.g., Agent inside Map-Reduce, Supervisor coordinating Multi-Agent).
**Rationale**: Real-world workflows combine patterns; Dapr's modular architecture supports composition.
**Alternatives Considered**: Fixed pattern hierarchy (rejected - inflexible), separate pattern implementations (rejected - duplication).

### 7. Generic Pattern Workloads (Decoupling from LLM)
**Decision**: Patterns (Agent, Map-Reduce, Supervisor) are generic DAG structures and must not be hardcoded to LLMs. "Map-Reduce" can process images or database rows; "Supervisor" can supervise legacy cron jobs. Any compatible work item must be executed durably through these patterns.
**Rationale**: Prevents framework lock-in to GenAI use-cases. Ensures the PocketFlow orchestration patterns are enterprise-ready for *any* durable workflow requirement.

## Risks / Trade-offs

**Risk**: Dapr Actor overhead for simple agents → Mitigation: Use Activities for stateless agents, Actors only when needed.
**Risk**: Pub/Sub latency for multi-agent communication → Mitigation: Use direct workflow calls for synchronous communication.
**Risk**: Pattern compatibility differences → Mitigation: Comprehensive compatibility tests.
**Risk**: Dapr sidecar adds operational complexity → Mitigation: Leverage existing Dapr deployment patterns.

## Migration Plan

1. Create pattern trait definitions building on core abstractions
2. Implement Agent pattern with Actor + Activity
3. Implement Map-Reduce pattern with Workflow fan-out/fan-in
4. Implement RAG pattern with Binding + Activity
5. Implement Multi-Agent pattern with Actors + Pub/Sub
6. Implement Supervisor pattern with Workflow loop
7. Create compatibility test suite
8. Document migration path from Python PocketFlow patterns

