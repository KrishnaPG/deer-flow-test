## Context

PocketFlow-Rust lacks design patterns (Agent, Map-Reduce, RAG, Multi-Agent, Supervisor). This design maps each pattern to Dapr building blocks to achieve durable, scalable, and observable pattern implementations. The patterns build on the core abstractions (Node, Flow, SharedStore, Params) ported in the previous change.

## Goals / Non-Goals

**Goals:**
- Map Agent pattern to Dapr Actors for stateful agents with Dapr Activities for actions
- Map Map-Reduce pattern to Dapr Workflow fan-out/fan-in with parallel activities
- Map RAG pattern to Dapr Bindings for vector DB retrieval and Activities for generation
- Map Multi-Agent pattern to Dapr Actors for agents with Pub/Sub for communication
- Map Supervisor pattern to Dapr Workflow loops with evaluation activities
- Maintain behavioral compatibility with Python PocketFlow pattern semantics

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Redesign pattern semantics
- Support all Python dynamic features (e.g., runtime pattern modification)
- Replace existing PocketFlow-Rust examples wholesale

## Decisions

### 1. Agent Pattern: Actor + Activity
**Decision**: Implement agents as Dapr Actors (stateful) with actions as Dapr Activities.
**Rationale**: Actors provide persistent conversation state and identity; activities provide stateless tool execution.
**Alternatives Considered**: Always Activities (rejected - no persistent state), always Workflows (rejected - overkill for single agent).

### 2. Map-Reduce Pattern: Workflow Fan-out/Fan-in
**Decision**: Use Dapr Workflow's `for_each` with parallelism limit for map, and activity for reduce.
**Rationale**: Native Dapr support for parallel execution with durability.
**Alternatives Considered**: Custom parallel execution (rejected - reinventing wheel), sequential processing (rejected - performance).

### 3. RAG Pattern: Binding + Activity
**Decision**: Use Dapr vector DB binding for retrieval, LLM activity for generation.
**Rationale**: Bindings provide pluggable vector stores; activities provide stateless generation.
**Alternatives Considered**: Custom vector DB client (rejected - reinventing wheel), integrated RAG activity (rejected - less flexible).

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

## Open Questions

1. Should we support both Actor and Activity implementations for agents?
2. How to handle pattern-specific configuration (e.g., parallelism limits, retry policies)?
3. What serialization format for cross-pattern communication?
4. How to integrate pattern observability with Dapr's OpenTelemetry?
5. Should we provide pattern templates for common use cases?