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

### 1. Agent Pattern: Agent Trait with Auto-Selection
**Decision**: Define an `Agent` trait that abstracts agent behavior. Support both simple async implementation (default) and Dapr Actor implementation (for multi-agent scenarios). Auto-select implementation based on configuration via central config module.
**Rationale**: Single-agent flows don't need Actor overhead. Multi-agent scenarios benefit from Actor isolation. Central config allows runtime selection without code changes.
**Pattern Structure**:
```rust
/// Core Agent trait - implementation agnostic
pub trait Agent: Node {
    async fn decide(&self, context: &Context) -> Result<Decision>;
    async fn execute_tool(&self, tool_call: ToolCall) -> Result<ToolResult>;
    fn agent_config(&self) -> AgentConfig;
}

/// Simple async implementation (default, high performance)
pub struct SimpleAgent {
    llm_client: Arc<dyn LLMClient>,
    tools: Vec<Box<dyn Tool>>,
}

impl Agent for SimpleAgent {
    async fn decide(&self, context: &Context) -> Result<Decision> {
        // Direct LLM call, no Actor overhead
    }
}

/// Actor-based implementation (for multi-agent isolation)
pub struct ActorAgent {
    actor_id: ActorId,
    // Dapr Actor handle
}

impl Agent for ActorAgent {
    async fn decide(&self, context: &Context) -> Result<Decision> {
        // Actor turn-based execution
    }
}

/// Factory for auto-selection based on config
pub struct AgentFactory;

impl AgentFactory {
    pub fn create_agent(config: &AgentConfig) -> Box<dyn Agent> {
        match config.execution_mode {
            ExecutionMode::Simple => Box::new(SimpleAgent::new(config)),
            ExecutionMode::Actor => Box::new(ActorAgent::new(config)),
            ExecutionMode::Auto => {
                // Auto-select based on config module settings
                if config::is_multi_agent_enabled() {
                    Box::new(ActorAgent::new(config))
                } else {
                    Box::new(SimpleAgent::new(config))
                }
            }
        }
    }
}
```

**Configuration via Central Config Module**:
```rust
// In berg10-config
pub struct AgentConfiguration {
    pub execution_mode: ExecutionMode,
    pub enable_actor_isolation: bool,
    pub default_max_iterations: usize,
}

pub enum ExecutionMode {
    Simple,  // Default - high performance async
    Actor,   // Dapr Actor - isolation for multi-agent
    Auto,    // Select based on context
}
```

### 2. Map-Reduce Pattern: Async Parallel with Configurable Policies
**Decision**: Use async `join!` or `join_all` for parallel execution. Support batched chunking for memory safety. Batch failure policies are **optional and configurable** - default to simple sequential semantics matching Python PocketFlow.
**Rationale**: Python's BatchNode is simple and effective. Advanced policies are opt-in for specific use cases. Default behavior should be intuitive and match Python semantics.
**Implementation**:
```rust
pub struct MapReduceFlow {
    map_node: Arc<dyn Node>,
    reduce_node: Arc<dyn Node>,
    batch_size: usize,
    failure_policy: BatchFailurePolicy,
    max_retries_per_item: usize,
}

/// Batch failure policies - ADVANCED, opt-in features
/// Default behavior matches Python: sequential processing with simple retry
pub enum BatchFailurePolicy {
    /// Default - match Python semantics
    /// Sequential processing, fail on first error
    Sequential,
    
    /// All items must succeed, or entire batch fails
    /// Use case: Transactional operations, related data
    Semantic,
    
    /// Individual items can fail, retry only failed items
    /// Use case: Independent operations, bulk processing
    Independent {
        checkpoint_successful: bool,
    },
    
    /// Continue processing, collect all errors at end
    /// Use case: Reporting, data migration
    BestEffort,
}

impl Default for BatchFailurePolicy {
    fn default() -> Self { BatchFailurePolicy::Sequential }
}

pub struct BatchResult<T> {
    pub successful: Vec<T>,
    pub failed: Vec<(Item, BatchItemError)>,
    pub partial_checkpoint: Option<CheckpointId>,
}

impl Flow for MapReduceFlow {
    async fn orchestrate(&self, input: Vec<Item>) -> Result<Output> {
        match self.failure_policy {
            BatchFailurePolicy::Semantic => {
                // All-or-nothing: If any fails, entire batch fails
                for chunk in input.chunks(self.batch_size) {
                    let mapped = futures::future::try_join_all(
                        chunk.iter().map(|item| self.map_with_retry(item))
                    ).await?; // ? propagates error - entire batch fails
                    
                    let reduced = self.reduce_node.exec(mapped).await?;
                    results.push(reduced);
                }
                Ok(aggregate(results))
            }
            
            BatchFailurePolicy::Independent { checkpoint_successful } => {
                // Process individually, checkpoint successful, retry failed
                let mut results = Vec::new();
                let mut failed_items = Vec::new();
                
                for chunk in input.chunks(self.batch_size) {
                    // Process items with individual error handling
                    let chunk_results: Vec<_> = futures::future::join_all(
                        chunk.iter().map(|item| async {
                            match self.map_with_retry(item).await {
                                Ok(result) => Ok(result),
                                Err(e) => {
                                    failed_items.push((item.clone(), e));
                                    Err(e)
                                }
                            }
                        })
                    ).await;
                    
                    // Collect successful results
                    let successful: Vec<_> = chunk_results.into_iter()
                        .filter_map(|r| r.ok())
                        .collect();
                    
                    // Checkpoint successful items if enabled
                    if checkpoint_successful && !successful.is_empty() {
                        self.checkpoint_successful_items(&successful).await?;
                    }
                    
                    results.extend(successful);
                }
                
                // Retry failed items individually
                if !failed_items.is_empty() {
                    for (item, error) in failed_items {
                        match self.retry_failed_item(item).await {
                            Ok(result) => results.push(result),
                            Err(e) => return Err(BatchError::ItemFailed(item, e)),
                        }
                    }
                }
                
                self.reduce_node.exec(results).await
            }
            
            BatchFailurePolicy::BestEffort => {
                // Process all, collect errors
                let mut successful = Vec::new();
                let mut errors = Vec::new();
                
                for chunk in input.chunks(self.batch_size) {
                    let results: Vec<_> = futures::future::join_all(
                        chunk.iter().map(|item| self.map_node.exec(item))
                    ).await;
                    
                    for (item, result) in chunk.iter().zip(results) {
                        match result {
                            Ok(r) => successful.push(r),
                            Err(e) => errors.push((item.clone(), e)),
                        }
                    }
                }
                
                // Reduce includes both successful and error summary
                self.reduce_node.exec(BatchResult {
                    successful,
                    failed: errors,
                    partial_checkpoint: None,
                }).await
            }
        }
    }
}
```

**Partial Failure Recovery**:
- **Semantic**: No checkpoint on failure, entire batch retried
- **Independent**: Checkpoint successful items, resume from failed items
- **Best Effort**: Continue processing, report all errors at end

**Checkpoint Granularity**:
- Semantic: Checkpoint after entire batch succeeds
- Independent: Checkpoint individual items as they succeed
- Best Effort: Optional checkpointing

**Required For**: pocketflow-map-reduce, pocketflow-batch-flow, pocketflow-parallel-batch-flow

### 3. RAG Pattern: Streaming Retrieval + Generation
**Decision**: Use Dapr vector DB binding for retrieval, LLM with streaming support for generation. Support four streaming strategies:
- **Direct**: Real-time streaming for responsive UIs (bypasses Dapr)
- **WebSocket**: Bidirectional streaming for chat interfaces
- **Durable**: Chunked via Dapr Pub/Sub for progress tracking
- **Complete**: Standard non-streaming for batch processing
**Rationale**: Streaming reduces TTFB. Multiple strategies support different use cases (chat vs batch). WebSocket support required for pocketflow-fastapi-websocket.
**Implementation**:
```rust
pub struct RAGNode {
    retriever: Arc<dyn VectorRetriever>,
    generator: Arc<dyn StreamingLLM>,
    stream_policy: StreamingPolicy,
}

pub enum StreamingPolicy {
    Direct,              // Real-time via channels
    WebSocket {         // Bidirectional WebSocket
        ws_sender: WebSocketSender,
        message_queue: Arc<RwLock<Vec<Message>>>,
    },
    Durable {           // Chunked via Pub/Sub
        topic: String,
        chunk_size: usize,
    },
    Complete,           // Non-streaming
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

### 4. Multi-Agent Pattern: Flexible Communication with Optional Actor Isolation
**Decision**: Support multiple multi-agent communication patterns: shared state (simplest), Dapr Pub/Sub (decoupled), and optional Dapr Actor isolation (advanced). Default to shared state for simplicity; escalate to Actors only when isolation is required.
**Rationale**: Not all multi-agent scenarios need Actor overhead. Shared state is simplest and fastest. Pub/Sub adds decoupling. Actors add isolation when needed.
**Communication Patterns**:
```rust
pub enum MultiAgentCommunication {
    /// Shared state - fastest, simplest (default)
    SharedState {
        shared_store: Arc<RwLock<SharedStore>>,
    },
    /// Dapr Pub/Sub - decoupled, async
    PubSub {
        topic_prefix: String,
    },
    /// Direct channels - for local coordination
    Channels {
        message_queues: HashMap<AgentId, Arc<Queue>>,
    },
}

pub struct MultiAgentFlow {
    agents: Vec<Box<dyn Agent>>,
    communication: MultiAgentCommunication,
    use_actor_isolation: bool,  // Optional: wrap agents in Dapr Actors
}
```
**Alternatives Considered**: Actors-only (rejected - unnecessary overhead for simple cases), Direct calls (rejected - tight coupling).

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

### 10. Heartbeat / Always-On Pattern
**Decision**: Implement continuous execution pattern with graceful checkpointing between iterations for always-on monitoring agents.
**Rationale**: pocketflow-heartbeat requires monitoring agents that run continuously, checkpointing state between cycles without requiring full workflow completion.
**Architecture**:
```rust
pub struct HeartbeatFlow {
    monitor_node: Arc<dyn HeartbeatNode>,
    interval: Duration,
    checkpoint_every: usize,  // Checkpoint every N iterations
}

pub trait HeartbeatNode: Node {
    /// Execute single heartbeat iteration
    async fn heartbeat(&self, iteration: u64, state: &mut SharedStore) -> Result<HeartbeatAction>;
}

pub enum HeartbeatAction {
    Continue,      // Continue to next iteration
    Pause,         // Checkpoint and wait for resume signal
    Shutdown,      // Graceful shutdown
    Alert(String), // Trigger alert and continue
}

impl HeartbeatFlow {
    async fn run(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut iteration = 0u64;
        let mut state = SharedStore::new();
        
        // Check for recovery from checkpoint
        if let Some(checkpoint) = ctx.load_checkpoint().await? {
            iteration = checkpoint.iteration;
            state = checkpoint.state;
            log::info!("Resumed heartbeat from iteration {}", iteration);
        }
        
        loop {
            iteration += 1;
            
            // Checkpoint periodically without stopping
            if iteration % self.checkpoint_every == 0 {
                ctx.checkpoint(HeartbeatCheckpoint {
                    iteration,
                    state: state.clone(),
                    timestamp: Utc::now(),
                }).await?;
            }
            
            // Execute heartbeat iteration
            match self.monitor_node.heartbeat(iteration, &mut state).await? {
                HeartbeatAction::Continue => {
                    // Continue after interval
                    tokio::time::sleep(self.interval).await;
                }
                HeartbeatAction::Pause => {
                    // Checkpoint and wait
                    ctx.checkpoint_and_pause().await?;
                    // Wait for resume signal
                    ctx.wait_for_resume().await?;
                }
                HeartbeatAction::Shutdown => {
                    // Final checkpoint and exit
                    ctx.checkpoint(HeartbeatCheckpoint {
                        iteration,
                        state,
                        timestamp: Utc::now(),
                    }).await?;
                    log::info!("Heartbeat shutdown gracefully at iteration {}", iteration);
                    break;
                }
                HeartbeatAction::Alert(msg) => {
                    // Send alert but continue
                    ctx.send_alert(msg).await?;
                    tokio::time::sleep(self.interval).await;
                }
            }
        }
        
        Ok(())
    }
}
```

**Signal Handling**:
- **SIGTERM**: Graceful shutdown, checkpoint current state
- **SIGUSR1**: Pause execution (for maintenance)
- **SIGUSR2**: Resume execution
- **SIGHUP**: Reload configuration

**Features**:
- Checkpoint between iterations without workflow completion
- Graceful shutdown on SIGTERM with state preservation
- Resume from last iteration on restart
- Pause/resume capability for maintenance
- Alert integration for monitoring

**Required For**: pocketflow-heartbeat, pocketflow-deep-research (iterative)

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

### 9. Simplified Configuration via Deployment Profiles
**Decision**: Provide preset deployment profiles that configure all settings appropriately. Users can start with a preset and customize if needed. Advanced users can use full manual configuration.
**Rationale**: The original design had 576+ configuration combinations (3 checkpoint policies × 4 batch policies × 3 communication patterns × 4 streaming strategies × 4 sandbox types). This is overwhelming. Presets reduce cognitive load while maintaining flexibility.

**Deployment Profiles**:
```rust
pub enum DeploymentProfile {
    /// Development - zero dependencies, maximum performance
    /// No Docker, no sidecar, no persistence
    Dev,
    
    /// Local Production - file-based persistence, no Docker
    /// Single-user, SQLite/ReDB storage
    LocalProd,
    
    /// Cloud - full Dapr features
    /// Multi-tenant, distributed, Kubernetes-ready
    Cloud,
    
    /// High Throughput - minimal durability
    /// For recoverable workflows where replay is acceptable
    HighThroughput,
    
    /// Critical Data - maximum durability
    /// Checkpoint everything, persisted SharedStore
    CriticalData,
}

impl DeploymentProfile {
    /// Convert preset to full configuration
    pub fn to_config(&self) -> FullConfig {
        match self {
            DeploymentProfile::Dev => FullConfig {
                durability: DurabilityLevel::InMemory,
                checkpoint_policy: CheckpointPolicy::Disabled,
                batch_failure_policy: BatchFailurePolicy::Sequential,
                communication: CommunicationPattern::SharedState,
                streaming: StreamingPolicy::Complete,
                sandbox: SandboxType::Seccomp,
                shared_store_policy: SharedStorePolicy::Ephemeral,
            },
            
            DeploymentProfile::LocalProd => FullConfig {
                durability: DurabilityLevel::ReDB,
                checkpoint_policy: CheckpointPolicy::EveryN(5),
                batch_failure_policy: BatchFailurePolicy::Independent { 
                    checkpoint_successful: true 
                },
                communication: CommunicationPattern::SharedState,
                streaming: StreamingPolicy::Complete,
                sandbox: SandboxType::Seccomp,
                shared_store_policy: SharedStorePolicy::Ephemeral,
            },
            
            DeploymentProfile::Cloud => FullConfig {
                durability: DurabilityLevel::Dapr,
                checkpoint_policy: CheckpointPolicy::EveryN(5),
                batch_failure_policy: BatchFailurePolicy::Independent { 
                    checkpoint_successful: true 
                },
                communication: CommunicationPattern::PubSub,
                streaming: StreamingPolicy::Direct,
                sandbox: SandboxType::Docker,
                shared_store_policy: SharedStorePolicy::Ephemeral,
            },
            
            DeploymentProfile::HighThroughput => FullConfig {
                durability: DurabilityLevel::Dapr,
                checkpoint_policy: CheckpointPolicy::Disabled, // No checkpointing
                batch_failure_policy: BatchFailurePolicy::BestEffort,
                communication: CommunicationPattern::SharedState,
                streaming: StreamingPolicy::Direct,
                sandbox: SandboxType::Seccomp,
                shared_store_policy: SharedStorePolicy::Ephemeral,
            },
            
            DeploymentProfile::CriticalData => FullConfig {
                durability: DurabilityLevel::Dapr,
                checkpoint_policy: CheckpointPolicy::EveryN(1), // Every node
                batch_failure_policy: BatchFailurePolicy::Semantic,
                communication: CommunicationPattern::PubSub,
                streaming: StreamingPolicy::Durable,
                sandbox: SandboxType::Firecracker,
                shared_store_policy: SharedStorePolicy::Persisted,
            },
        }
    }
}

/// Advanced: Full manual configuration (opt-in)
pub struct FullConfig {
    pub durability: DurabilityLevel,
    pub checkpoint_policy: CheckpointPolicy,
    pub batch_failure_policy: BatchFailurePolicy,
    pub communication: CommunicationPattern,
    pub streaming: StreamingPolicy,
    pub sandbox: SandboxType,
    pub shared_store_policy: SharedStorePolicy,
}
```

**Usage**:
```rust
// Simple: Use a preset
let config = DeploymentProfile::LocalProd.to_config();

// Advanced: Start from preset and customize
let mut config = DeploymentProfile::Cloud.to_config();
config.checkpoint_policy = CheckpointPolicy::SafePointsOnly;
config.batch_failure_policy = BatchFailurePolicy::Sequential;
```

**Key Benefits**:
- **5 presets** cover 95% of use cases
- **No Docker required** for Dev and LocalProd profiles
- **Sensible defaults** - users don't need to understand all options
- **Extensible** - advanced users can customize
- **Type-safe** - invalid combinations caught at compile time

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

