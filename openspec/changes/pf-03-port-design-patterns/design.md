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

### 2. Map-Reduce Pattern: Async Parallel with Batching & Failure Policies
**Decision**: Use async `join!` for local parallel execution or Dapr's `when_all` for distributed. Implement **Batched Chunking** to process large datasets in safe chunks (e.g., 100 items at a time). Support user-configurable batch failure policies.
**Rationale**: Custom orchestrator with async Rust provides parallel execution. Batching prevents memory exhaustion. Failure policies let user decide semantic vs convenience batching.
**Implementation**:
```rust
pub struct MapReduceFlow {
    map_node: Arc<dyn Node>,
    reduce_node: Arc<dyn Node>,
    batch_size: usize,
    failure_policy: BatchFailurePolicy,
    max_retries_per_item: usize,
}

pub enum BatchFailurePolicy {
    /// All items must succeed, or entire batch fails (semantic batching)
    /// Use case: Transactional operations, related data
    /// If any item fails, entire batch fails
    Semantic,
    
    /// Individual items can fail, retry only failed items (convenience batching)
    /// Use case: Independent operations, bulk processing
    /// Checkpoint succeeded items, retry only failed
    Independent {
        checkpoint_successful: bool,  // Save successful items
    },
    
    /// Continue processing, collect all errors at end (best-effort)
    /// Use case: Reporting, data migration
    /// Process all items, return results + errors
    BestEffort,
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

### 9. Voice/Audio Streaming Pattern
**Decision**: Support real-time audio streaming for voice chat applications (pocketflow-voice-chat).
**Rationale**: Voice interfaces require low-latency bidirectional audio streaming (STT + LLM + TTS pipeline).
**Architecture**:
```rust
pub struct VoiceChatFlow {
    stt_client: Arc<dyn STTClient>,      // Speech-to-text
    llm_client: Arc<dyn StreamingLLM>,   // LLM with streaming
    tts_client: Arc<dyn TTSClient>,      // Text-to-speech
    audio_config: AudioConfig,
}

pub struct AudioConfig {
    pub sample_rate: u32,           // 16kHz, 44.1kHz, etc.
    pub channels: u16,              // Mono=1, Stereo=2
    pub chunk_duration_ms: u64,     // Audio chunk size
    pub codec: AudioCodec,          // Opus, PCM, etc.
}

pub enum AudioCodec {
    Opus,
    Pcm16LE,
    PcmFloat,
}

impl VoiceChatFlow {
    async fn run(&self, ctx: &ExecutionContext) -> Result<()> {
        // Pipeline: Audio Input -> STT -> LLM -> TTS -> Audio Output
        let audio_input = self.receive_audio_stream(ctx).await?;
        
        // STT with streaming transcription
        let transcription_stream = self.stt_client.transcribe_stream(audio_input);
        
        // LLM processes transcription, streams response
        let llm_stream = self.llm_client.generate_stream(transcription_stream);
        
        // TTS converts to audio chunks
        let audio_output = self.tts_client.synthesize_stream(llm_stream);
        
        // Send audio back to user
        self.send_audio_stream(audio_output).await?;
        
        Ok(())
    }
}
```
**Features**:
- Real-time audio streaming (STT + LLM + TTS pipeline)
- Bidirectional WebSocket for audio I/O
- Audio buffering and jitter handling
- Silence detection for turn-taking
- Interrupt handling (user can interrupt TTS)
**Required For**: pocketflow-voice-chat

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

