## Context

PocketFlow is a minimalist 100-line Python framework for LLM workflows. To achieve industrial-grade enterprise deployment, we need to port it to Rust with Dapr for distributed application runtime capabilities. This meta-plan establishes the process to create detailed porting plans through a series of OpenSpec changes, each addressing a specific aspect of the porting project.

## Goals / Non-Goals

**Goals:**
- Create a systematic process for porting PocketFlow to Rust with Dapr
- Ensure all PocketFlow features are accounted for in porting plans
- Establish quality standards and validation criteria for each porting plan
- Integrate progress tracking into the OpenSpec change process
- Enable collaborative brainstorming with user approval at each step
- Provide clear, actionable OpenSpec changes for implementation teams

**Non-Goals:**
- Implement the actual PocketFlow to Rust port (this meta-plan only creates the plans)
- Make automatic decisions without user approval
- Skip validation or quality assurance steps
- Create plans without proper brainstorming and exploration

## Decisions

### 1. Meta-Plan as OpenSpec Change
**Decision**: Implement the meta-plan as an OpenSpec change that generates other OpenSpec changes.
**Rationale**: Leverages existing OpenSpec infrastructure for tracking, validation, and quality assurance. Ensures consistency with other project changes.
**Alternatives Considered**: Standalone markdown document (rejected - lacks tracking and validation), separate project management tool (rejected - adds complexity).

### 2. Skill-Based Task Execution
**Decision**: Each task uses appropriate OpenSpec skills (openspec-explore for brainstorming, openspec-propose for creation).
**Rationale**: Ensures thorough exploration and collaborative brainstorming before creating artifacts. Maintains user control over decisions.
**Alternatives Considered**: Automated task execution (rejected - lacks user input), manual file creation (rejected - inconsistent structure).

### 3. Progress Tracking Integration
**Decision**: Integrate progress tracking directly into the OpenSpec change structure.
**Rationale**: Makes progress tracking part of the process rather than an extra step. Updates automatically as tasks complete.
**Alternatives Considered**: Separate tracking document (rejected - extra step), external project management tool (rejected - adds complexity).

### 4. Validation at Multiple Levels
**Decision**: Implement meta-validation criteria for OpenSpec changes and feature validation criteria for ported features.
**Rationale**: Ensures both the planning process and the resulting plans meet quality standards. Provides clear success/failure criteria.
**Alternatives Considered**: Single-level validation (rejected - insufficient), no validation (rejected - quality risk).

### 5. Collaborative Brainstorming Approach
**Decision**: Each task involves brainstorming with user before creating OpenSpec change.
**Rationale**: Ensures all aspects are considered, user expertise is incorporated, and decisions are approved.
**Alternatives Considered**: Automatic decision-making (rejected - lacks user input), minimal user involvement (rejected - quality risk).

## Risks / Trade-offs

**Risk**: Overly complex meta-plan process → Mitigation: Keep meta-plan focused on essential tasks, avoid unnecessary bureaucracy.
**Risk**: User approval bottleneck → Mitigation: Clear approval criteria, parallel task execution where possible.
**Risk**: Incomplete feature coverage → Mitigation: Comprehensive inventory phase, validation against PocketFlow source.
**Risk**: Dapr mapping challenges → Mitigation: Early evaluation of Dapr capabilities, fallback strategies for gaps.
**Risk**: Progress tracking overhead → Mitigation: Integrate tracking into existing OpenSpec commands, automate updates.

## Migration Plan

1. Create this meta-plan OpenSpec change
2. Execute Task 1: Evaluate existing PocketFlow-Rust port
3. Based on evaluation, proceed with Tasks 2-8 in dependency order
4. Each task output becomes a new OpenSpec change
5. Progress tracked automatically as changes complete
6. Final validation of all porting plans against meta-validation criteria

## Core Architecture Decisions (Established)

The following architectural decisions have been established through design review and must guide all OpenSpec changes:

### 1. Hybrid Architecture: Custom Orchestrator + Dapr Enterprise Services

**Decision**: Use a custom workflow orchestrator for graph execution (NOT Dapr Workflows), but leverage Dapr for enterprise features (state persistence, resiliency, observability).

**Rationale**: Dapr Workflows require static workflow definitions known at build time. Python PocketFlow supports dynamic graph modification at runtime (adding/removing nodes, changing successors during execution). A custom orchestrator maintains this flexibility while Dapr provides durability and enterprise features.

**Implementation**:
- **Layer 1**: Custom `Orchestrator` with `Node`/`Flow` traits (full control, dynamic graphs)
- **Layer 2**: Dapr integration via `Durability` trait (state store, pub/sub, tracing, resiliency)

```
┌─────────────────────────────────────────────┐
│          Custom Orchestrator                │
│  (Dynamic graphs, runtime routing)          │
├─────────────────────────────────────────────┤
│          Durability Trait                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │  Dapr    │ │  Local   │ │  SQLite  │    │
│  │ (dist)   │ │ (memory) │ │ (file)   │    │
│  └──────────┘ └──────────┘ └──────────┘    │
└─────────────────────────────────────────────┘
```

### 2. Dynamic Graph Support (Python-Equivalent)

**Decision**: Replicate Python's dynamic graph capabilities exactly:
- Nodes can add/remove successors at runtime via shared `Arc<RwLock<HashMap>>`
- Routing happens at runtime based on action strings from `post()`
- Graph structure is mutable during execution

**Key Mechanism**: Shallow copy pattern - cloning a node clones its ID and params but shares the `successors` HashMap via `Arc`, enabling runtime modification.

### 3. Async-By-Default

**Decision**: All node operations are async. No sync variants.

**Rationale**: Modern Rust async is the standard. Users can block if needed, but the framework is async-first.

### 4. Durability Trait with Multiple Backends

**Decision**: Abstract over durability via trait, supporting:
- **DaprDurability**: Distributed, enterprise-grade (Dapr sidecar required)
- **LocalDurability**: Single-user production (SQLite persistence, no sidecar)
- **InMemoryDurability**: Development/testing (no persistence)

**Use Cases**:
- **Dapr**: Multi-tenant SaaS, high availability, horizontal scaling
- **Local**: Single-user desktop apps, resource-constrained deployments
- **InMemory**: Unit tests, CI/CD, rapid development

### 5. Streaming Architecture (Three-Tier)

**Decision**: Support streaming where LLM utilities support it:

| Tier | Strategy | Use Case |
|------|----------|----------|
| Direct | Bypass Dapr, use channels | Real-time chat, UI updates |
| PubSub | Durable chunks via Dapr Pub/Sub | RAG with progress |
| Complete | Standard execution | Batch processing |

**Streaming Support Matrix**:
- OpenAI/Claude: ✅ Direct streaming
- Local LLMs: ✅ Direct streaming
- TTS: ✅ Audio chunk streaming
- STT: ✅ Real-time transcription
- Embeddings: ❌ Complete only
- Web Search: ❌ Complete only

### 6. Flow-as-Node Hierarchical Composition

**Decision**: Flow implements Node trait (like Python), enabling:
- `flow_a >> flow_b` (Flow as node in another flow)
- Nested workflow composition
- Sub-flows with their own orchestration

**Implementation**: Flow execution creates a sub-orchestrator. Shared state merges back to parent on completion.

### 7. Dapr Usage Matrix

| Feature | Custom | Dapr | Notes |
|---------|--------|------|-------|
| Graph execution | ✅ | ❌ | Custom orchestrator required |
| Node dispatch | ✅ | ❌ | Runtime routing |
| State persistence | ❌ | ✅ | Dapr State Store |
| Retry/Circuit Breaker | ❌ | ✅ | Dapr Resiliency |
| Pub/Sub messaging | ❌ | ✅ | Dapr Pub/Sub |
| Distributed tracing | ❌ | ✅ | Dapr + OpenTelemetry |
| Secret management | ❌ | ✅ | Dapr Secret Store |
| Sub-workflows | ✅ (optional) | ✅ | Complex cases only |

### 8. Checkpoint and Recovery Strategy

**Decision**: Configurable checkpoint policy (Option B)
- Default: Checkpoint every 5 node transitions
- Per-workflow configuration: User can specify checkpoint frequency
- Options: Every N nodes, every safe-point, or explicit checkpoint nodes

**Rationale**: Balance between durability and performance. Every node transition is too slow for high-throughput workflows; explicit only is too risky for long-running flows.

**Implementation**:
```rust
pub enum CheckpointPolicy {
    EveryN(usize),           // Checkpoint every N transitions
    SafePointsOnly,          // Only at marked safe points
    ExplicitOnly,            // Only explicit checkpoint nodes
}

impl Default for CheckpointPolicy {
    fn default() -> Self {
        CheckpointPolicy::EveryN(5)
    }
}
```

### 9. Graph Versioning Strategy

**Decision**: No built-in versioning (like Python)

**Rationale**: Python PocketFlow has no versioning. The graph is defined in code and executed. If code changes mid-execution, behavior is undefined. We match this simplicity.

**Implications**:
- Code updates should wait for workflows to complete
- Checkpoints include graph structure snapshot
- Recovery uses saved graph structure (may differ from current code)
- **Risk**: Code update during active workflow = undefined behavior

**Future Enhancement**: Could add explicit versioning later if needed

### 10. Trait Complexity: Runtime Capability Flags (Option A)

**Decision**: Use runtime flags in `ExecutionMode` struct

**Structure**:
```rust
pub struct ExecutionMode {
    pub durability: DurabilityLevel,    // Distributed/Persistent/Volatile
    pub streaming: StreamingPolicy,     // Stream/Batch/Complete
    pub checkpoint_policy: CheckpointPolicy,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode {
            durability: DurabilityLevel::Distributed,
            streaming: StreamingPolicy::Complete,
            checkpoint_policy: CheckpointPolicy::default(),
        }
    }
}
```

**Usage**: User specifies mode at call site; runtime enforces constraints

### 11. Non-Durable Deployment (Single-User Production)

**Target**: Desktop apps, single-tenant deployments, edge devices

**Configuration**:
- SQLite for state persistence
- File-based logging/tracing
- Local retry policies
- Resource limits enforced

## Open Questions

1. What is the exact scope of enterprise features required? (Partially answered: observability, durability, scalability are must-haves)
2. How to handle semantic differences between Python and Rust/Dapr?
3. What performance benchmarks should be used for validation?
4. How to ensure backward compatibility with existing PocketFlow users?
5. What is the timeline and resource allocation for the porting project?