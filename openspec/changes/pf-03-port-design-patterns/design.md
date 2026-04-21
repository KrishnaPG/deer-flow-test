## Context

PocketFlow-Rust lacks design patterns (Agent, Map-Reduce, RAG, Multi-Agent, Supervisor). This design maps each pattern to Dapr building blocks to achieve durable, scalable, and observable pattern implementations. The patterns build on the core abstractions (Node, Flow, SharedStore, Params) ported in the previous change.

We are taking a **fresh start approach**: use Python PocketFlow patterns as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust implementation should do everything Python patterns can do, but better - with built-in enterprise features that Python lacks.

## Goals / Non-Goals

**Goals:**
- Implement Agent pattern using custom orchestrator with Dapr State for persistence
- Implement Map-Reduce pattern using custom orchestrator with async join! or Dapr fan-out/fan-in
- Implement RAG pattern with streaming support where LLM utilities support it
- Implement Multi-Agent pattern with Dapr Pub/Sub for communication
- Implement Supervisor pattern using custom orchestrator with evaluation loops
- Support Flow-as-Node composition (patterns can be nested)
- Provide functional equivalence with Python PocketFlow pattern semantics
- Prioritize working implementation over nitpicking about 100% line equality
- Add Dapr-native resilience (retry, circuit breaker, fallback) that Python patterns lack
- All patterns are async-by-default

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Redesign pattern semantics in ways that break functional equivalence
- Support all Python dynamic features that don't translate to Rust
- Replace existing PocketFlow-Rust examples wholesale
- Achieve line-by-line code equivalence with Python (not required)

## Decisions

### 1. Agent Pattern: Custom Orchestrator + Dapr State
**Decision**: Implement agent using custom orchestrator with Dapr State for conversation persistence. Agent decision logic runs in standard async Rust (not Dapr Actors/Activities). State (conversation history) is stored in Dapr State Store.
**Rationale**: Custom orchestrator supports dynamic graph modification needed for agent routing. Dapr State provides durability without Actor turn-based concurrency bottlenecks.
**Pattern Structure**:
```rust
pub struct AgentNode {
    llm_client: Arc<dyn LLMClient>,
    tools: Vec<Box<dyn Tool>>,
}

impl Node for AgentNode {
    async fn exec(&self, input: PrepResult) -> Result<ExecResult> {
        // Call LLM with tools
        // Return action: "tool_call", "respond", "fallback"
    }
}
```

### 2. Map-Reduce Pattern: Async Parallel with Batching
**Decision**: Use async `join!` for local parallel execution or Dapr's `when_all` for distributed. Implement **Batched Chunking** to process large datasets in safe chunks (e.g., 100 items at a time).
**Rationale**: Custom orchestrator with async Rust provides parallel execution. Batching prevents memory exhaustion.
**Implementation**:
```rust
pub struct MapReduceFlow {
    map_node: Arc<dyn Node>,
    reduce_node: Arc<dyn Node>,
    batch_size: usize,
}

impl Flow for MapReduceFlow {
    async fn orchestrate(&self, input: Vec<Item>) -> Result<Output> {
        // Batch processing
        for chunk in input.chunks(self.batch_size) {
            // Parallel map
            let mapped = futures::future::try_join_all(
                chunk.iter().map(|item| self.map_node.exec(item))
            ).await?;
            
            // Reduce
            let reduced = self.reduce_node.exec(mapped).await?;
            results.push(reduced);
        }
        Ok(aggregate(results))
    }
}
```

### 3. RAG Pattern: Streaming Retrieval + Generation
**Decision**: Use Dapr vector DB binding for retrieval, LLM with streaming support for generation. Support three streaming strategies:
- **Direct**: Real-time streaming for responsive UIs
- **Durable**: Chunked via Dapr Pub/Sub for progress tracking
- **Complete**: Standard non-streaming for batch processing
**Rationale**: Streaming reduces TTFB. Multiple strategies support different use cases (chat vs batch).
**Implementation**:
```rust
pub struct RAGNode {
    retriever: Arc<dyn VectorRetriever>,
    generator: Arc<dyn StreamingLLM>,
    stream_policy: StreamingPolicy,
}

impl StreamingNode for RAGNode {
    fn exec_stream(&self, query: String) -> Pin<Box<dyn Stream<Item = Result<Chunk>> + Send>> {
        // 1. Retrieve documents (async)
        // 2. Stream generation chunks as they're produced
        Box::pin(stream! {
            let docs = self.retriever.retrieve(&query).await?;
            let mut stream = self.generator.generate_stream(&query, &docs);
            while let Some(chunk) = stream.next().await {
                yield chunk;
            }
        })
    }
}
```

### 4. Multi-Agent Pattern: Actors + Pub/Sub
**Decision**: Implement agents as Dapr Actors, communication via Dapr Pub/Sub topics.
**Rationale**: Actors maintain agent state; Pub/Sub enables decoupled asynchronous messaging.
**Alternatives Considered**: Direct workflow calls (rejected - tight coupling), shared state only (rejected - no async communication).

### 5. Supervisor Pattern: Custom Orchestrator Loop
**Decision**: Implement supervisor using custom orchestrator with evaluation loop and checkpointing.
**Rationale**: Custom orchestrator supports dynamic agent routing. Durable loops with checkpointing.
**Implementation**:
```rust
pub struct SupervisorFlow {
    agents: Vec<Arc<dyn Node>>,
    evaluator: Arc<dyn Node>,
    max_iterations: usize,
}

impl Flow for SupervisorFlow {
    async fn orchestrate(&self, ctx: &ExecutionContext) -> Result<Action> {
        for iteration in 0..self.max_iterations {
            // Checkpoint every iteration
            if iteration % 5 == 0 {
                ctx.checkpoint().await?;
            }
            
            // Delegate to agents
            let results = futures::future::try_join_all(
                self.agents.iter().map(|agent| agent.exec(...))
            ).await?;
            
            // Evaluate
            let decision = self.evaluator.exec(results).await?;
            
            if decision.is_complete() {
                return Ok(Action::Complete);
            }
        }
        
        Ok(Action::MaxIterationsReached)
    }
}
```

### 6. Flow-as-Node Composition
**Decision**: All patterns implement Flow trait, which implements Node trait. Patterns can be used as nodes in other patterns.
**Rationale**: Matches Python's `class Flow(BaseNode)`. Enables hierarchical composition.
**Example**:
```rust
// Map-Reduce with Agent nodes
let agent_flow = create_agent_flow();
let map_reduce = MapReduceFlow::new(agent_flow);

// Supervisor with Map-Reduce as agent
let supervisor = SupervisorFlow::new(vec![
    Arc::new(map_reduce),
    Arc::new(other_agent),
]);
```
**Composition Rules**:
- Flows can be nested arbitrarily
- SharedStore is scoped per Flow (merged back to parent)
- Checkpoints work at each Flow level
- Durability settings inherited from parent unless overridden

### 7. Async-First Patterns
**Decision**: All pattern implementations are async. No sync variants.
**Rationale**: Modern Rust async is standard. Patterns use async for I/O (LLM calls, DB queries).
**Note**: Users can block if needed, but framework is async-only.

### 8. Generic Pattern Workloads (Decoupling from LLM)
**Decision**: Patterns (Agent, Map-Reduce, Supervisor) are generic DAG structures and must not be hardcoded to LLMs. "Map-Reduce" can process images or database rows; "Supervisor" can supervise legacy cron jobs. Any compatible work item must be executed durably through these patterns.
**Rationale**: Prevents framework lock-in to GenAI use-cases. Ensures the PocketFlow orchestration patterns are enterprise-ready for *any* durable workflow requirement.

## Risks / Trade-offs

**Risk**: Custom orchestrator complexity → Mitigation: Start with simple implementation, add features incrementally.
**Risk**: Streaming increases complexity → Mitigation: Make streaming opt-in, not required.
**Risk**: Flow-as-Node composition overhead → Mitigation: Benchmark nested flows, optimize if needed.
**Risk**: Pattern compatibility differences → Mitigation: Comprehensive compatibility tests against Python cookbooks.
**Risk**: Dapr sidecar adds operational complexity → Mitigation: Provide LocalDurability for single-user deployments.

## Migration Plan

1. Create pattern trait definitions building on core abstractions (Node, Flow)
2. Implement Agent pattern with custom orchestrator
3. Implement Map-Reduce pattern with async parallel execution
4. Implement RAG pattern with streaming support
5. Implement Multi-Agent pattern with Pub/Sub communication
6. Implement Supervisor pattern with custom orchestrator loop
7. Test Flow-as-Node composition (nesting patterns)
8. Create compatibility test suite
9. Document migration path from Python PocketFlow patterns

