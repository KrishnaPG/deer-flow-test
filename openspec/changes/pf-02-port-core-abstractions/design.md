## Context

PocketFlow-Rust provides a foundation with Node trait (`prepare → execute → post_process`), Flow struct (state-based transitions), Context (key-value store), and Params (`HashMap<String, serde_json::Value>`). However, it lacks Dapr integration and enterprise features like retry, fallback, and parallel execution. This design maps PocketFlow concepts to Dapr building blocks to achieve durable, scalable, and observable workflows.

We are taking a **fresh start approach**: use Python PocketFlow as inspiration ("in spirit" and "logic"), not as a strict line-by-line template. The Rust implementation should do everything Python can do, but better - faster, lower latency, zero-copy, zero-allocation where possible, with built-in enterprise features that Python lacks.

## Goals / Non-Goals

**Goals:**
- Implement custom orchestrator (NOT Dapr Workflows) for dynamic graph execution
- Map Node abstraction to executable units with prep→exec→post lifecycle
- Support runtime graph modification (nodes can add/remove successors dynamically)
- Map Flow abstraction to orchestrated subgraphs (Flow is a Node)
- Map SharedStore to Dapr State Management (distributed) or local storage (single-user)
- Map Params to immutable workflow input data
- Add retry logic using Dapr resiliency policies (or local equivalents)
- Add fallback mechanisms via Action-based routing (semantic fallbacks)
- Add parallel execution using async join! or Dapr fan-out/fan-in
- Provide functional equivalence with Python PocketFlow semantics
- Prioritize getting to working state quickly over nitpicking about 100% line equality
- Aim for zero-copy, zero-allocation where possible for performance
- Implement action-based routing where nodes return action strings that control graph traversal
- Support loop construction via back-edges (wiring later node back to earlier node)
- Use Dapr for enterprise features (state, retry, observability), NOT for workflow orchestration

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Redesign PocketFlow's core semantics in ways that break functional equivalence
- Support all Python dynamic features that don't translate to Rust
- Replace existing PocketFlow-Rust code wholesale
- Achieve line-by-line code equivalence with Python (not required)
- Use Dapr Workflows for orchestration (rejected - doesn't support dynamic graphs)

## Decisions

### 1. Custom Orchestrator with Dapr Enterprise Layer
**Decision**: Implement a custom `Orchestrator` for graph execution (NOT Dapr Workflows). Use Dapr ONLY for enterprise durability features: State Store (persistence), Pub/Sub (messaging), Resiliency (retry/CB), Tracing (observability), Secret Store (security). Dapr is NOT used for workflow orchestration.
**Rationale**: Dapr Workflows require static workflow definitions known at build time. Python PocketFlow supports dynamic graph modification at runtime (nodes can add/remove successors during execution). A custom orchestrator maintains this flexibility while Dapr provides durability and enterprise features.
**Architecture**:
```
┌─────────────────────────────────────────────┐
│          Custom Orchestrator                │
│  (Dynamic graph traversal, action routing)  │
├─────────────────────────────────────────────┤
│         Durability Trait                    │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ InMemory│ │  ReDB   │ │  Dapr   │       │
│  │ (default│ │(local   │ │(cloud)  │       │
│  │  dev)   │ │ prod)   │ │         │       │
│  └─────────┘ └─────────┘ └─────────┘       │
└��────────────────────────────────────────────┘
```
**Key Principles**:
- Dapr is ONLY for durability/state management, NEVER for orchestration
- Orchestrator is custom-built for dynamic graph support
- Three-tier durability: InMemory (default) → ReDB → Dapr
- Auto-detect Dapr sidecar; warn/fail if DaprDurability requested but sidecar missing
- **Remote Dapr Sidecar**: When `DAPR_HTTP_ENDPOINT` or `DAPR_GRPC_ENDPOINT` environment variable is set, the SDK will connect to a remote Dapr sidecar instead of localhost. This enables running the Dapr sidecar on a different machine while the application runs locally without Docker.
**Alternatives Considered**: Dapr Workflows (rejected - static graphs only), Temporal (rejected - similar static constraint).

### 1a. Remote Dapr Sidecar Connectivity
**Decision**: Support connecting to a Dapr sidecar running on a remote machine, not just localhost. This is implemented via environment variables following Dapr v1.12+ specification.
**Rationale**: Enables hybrid deployments where the application runs locally (without Docker) while optionally connecting to a remote Dapr sidecar for distributed features. The local ReDB + sqlite-vss provides full functionality without requiring Docker at all.
**Configuration**:
```rust
// Environment variables for remote Dapr sidecar connection
// DAPR_HTTP_ENDPOINT - e.g., https://dapr-api.example.com:3500
// DAPR_GRPC_ENDPOINT - e.g., https://dapr-grpc.example.com:50001
// DAPR_API_TOKEN - optional authentication token

pub enum DaprConnectionMode {
    /// Connect to local sidecar (localhost) - default
    Local,
    /// Connect to remote sidecar via environment variables
    Remote { endpoint: String },
    /// No Dapr sidecar - use ReDB only for local production
    None,
}

impl DaprConnectionMode {
    pub fn detect() -> Self {
        if let Ok(endpoint) = std::env::var("DAPR_HTTP_ENDPOINT") {
            DaprConnectionMode::Remote { endpoint }
        } else if let Ok(endpoint) = std::env::var("DAPR_GRPC_ENDPOINT") {
            DaprConnectionMode::Remote { endpoint }
        } else {
            DaprConnectionMode::None
        }
    }
}
```

**Three Deployment Models**:
```
┌─────────────────────────────────────────────────────────────────┐
│  Model 1: Development (No Dapr, No Docker)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Your App   │  │    ReDB      │  │  sqlite-vss  │    │
│  │  (Rust)      │  │ (Durability)  │  │ (Vector DB)  │    │
│  └─────────────┘  └──────────────┘  └──────────────┘    │
│         │              │                   │                   │
│         └─────────────┴───────────────────┘                   │
│         (All local, file-based, no network required)          │
│                                                          │
├───────────────────────────────────────────────────────────────  │
│  Model 2: Local + Remote Dapr Sidecar                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Your App   │  │    ReDB      │  │  sqlite-vss  │    │
│  │  (Rust)      │  │ (Durability)  │  │ (Vector DB)  │    │
│  ��─────────────┘  └──────────────┘  └──────────────┘    │
│         │                                                  │
│         │ DAPR_HTTP_ENDPOINT / DAPR_GRPC_ENDPOINT            │
│         ▼                                                  │
│  ┌──────────────────────────────────────────────┐        │
│  │     DAPR SIDE CAR (Remote Machine)              │        │
│  │  ┌────────┐ ┌────────┐ ┌────────┐         │        │
│  │  │ State  │ │ Pub/Sub│ │ Secrets│         │        │
│  │  └───────┘ └───────┘ └───────┘         │        │
│  └──────────────────────────────────────────────┘        │
│                                                          │
├───────────────────────────────────────────────────────────────  │
│  Model 3: Full Dapr Deployment (Kubernetes)                  │
│  ┌──────────────────────────────────────────────┐          │
│  │     Your App + Dapr Sidecar (same pod)         │          │
│  │  ┌────────┐ ┌────────┐ ┌────────┐         │          │
│  │  │ State  │ │ Pub/Sub│ │ Secrets│         │          │
│  │  └───────┘ └───────┘ └───────┘         │          │
│  └──────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────┘
```

### 2. Flow as Node with Custom Orchestration
**Decision**: Flow implements the Node trait (like Python's `class Flow(BaseNode)`). Flow execution creates a sub-orchestrator that runs the flow's internal graph.
**Rationale**: Enables hierarchical composition where Flows can be used as nodes in other Flows (`flow_a >> flow_b`). Custom orchestration within Flow maintains dynamic graph capabilities.
**Execution**:
```rust
impl Node for Flow {
    async fn exec(&self, input: PrepResult) -> Result<ExecResult> {
        let orchestrator = Orchestrator::new(self.durability());
        let final_action = orchestrator.run(
            self.start_node(),
            &mut local_store
        ).await?;
        Ok(ExecResult::FlowCompleted { action: final_action })
    }
}
```
**Alternatives Considered**: Dapr Workflows for Flow (rejected - static graphs), Flattening Flow into parent graph (rejected - breaks composition).

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

### 8. Strict Action Enum Routing and Builder Pattern

**Decision**: Replace dynamic string routing with an `Action` enum (including a `Custom(String)` variant for flexibility) and mandate a Builder pattern for graph construction rather than dynamic dictionary modification or operator overloading.

**Rationale**: `HashMap<String, Arc<RwLock<Node>>>` for dynamic graph mutation is an anti-pattern in Rust that leads to deadlock risks and runtime panics. The graph structure must be deterministically constructed using a Builder pattern to ensure safety and clear topology.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Default,
    Done,
    Retry,
    Fallback,
    Custom(String),
}

pub struct FlowBuilder {
    nodes: HashMap<String, Arc<dyn Node>>,
    edges: HashMap<(String, Action), String>,
    start_node: Option<String>,
}

impl FlowBuilder {
    pub fn new() -> Self { ... }
    
    pub fn add_node(mut self, name: &str, node: impl Node + 'static) -> Self { ... }
    
    pub fn route(mut self, from: &str, action: Action, to: &str) -> Self { ... }
    
    pub fn start_at(mut self, node: &str) -> Self { ... }
    
    pub fn build(self) -> Result<Flow> { ... }
}
```

**Dynamic Capabilities Removed/Replaced**:
- The graph structure is **immutable** once built.
- Flow-as-Node composition is fully supported via the Builder.
- Routing is defined explicitly, ensuring no unhandled `Action` returns.

### 10. Engine/Driver Separation & Domain-Agnostic Orchestration
**Decision**: **Dapr is not the engine; it provides enterprise durability services ONLY.** The core `Node`, `Flow`, and orchestration patterns must be completely decoupled from both LLM-specific logic AND the execution environment.

Architecture split:
*   **`berg10-execution-engine`**: Core engine with `Node` trait, `Flow` orchestrator, custom `Orchestrator` implementation. Zero AI dependencies. Zero Dapr dependencies.
*   **`berg10-durability`**: Durability trait and implementations:
    - `InMemoryDurability`: Default for development - zero dependencies, fastest iteration
    - `ReDBDurability`: Single-user production - file-based persistence using ReDB
    - `DaprDurability`: Distributed enterprise - requires Dapr sidecar, multi-tenant, HA
*   **`berg10-ai-nodes`**: LLM/AI specific Node implementations

**Four-Tier Deployment Model** (No Docker Required for Dev/Local):
```
Development (Default - Zero Dependencies):
  cargo run --example hello-world
  → Uses InMemoryDurability
  → Zero setup, instant feedback
  → No Docker, no sidecar, no external services
  → CheckpointPolicy::Disabled (max performance)
  → SharedStorePolicy::Ephemeral

Local Production (Single-User, No Docker):
  cargo run --example hello-world --features local-prod
  → Uses ReDBDurability (file-based persistence)
  → No sidecar needed, no Docker needed
  → ReDB + embedvec for vector search
  → CheckpointPolicy::EveryN(5)
  → Works on any machine with just Rust installed

Hybrid (Local App + Remote Dapr Sidecar):
  DAPR_HTTP_ENDPOINT=https://dapr-server:3500 cargo run --example hello-world
  → Uses DaprRemoteDurability
  → App runs locally without Docker
  → Connects to remote Dapr sidecar for distributed features
  → ReDB + embedvec still available locally for resilience

Distributed/Cloud (Full Enterprise Features):
  dapr run -- cargo run --example hello-world
  → Uses DaprDurability
  → Full enterprise features (retry, circuit breaker, distributed tracing)
  → Kubernetes-ready
  → Optional: Enable batch checkpointing for high throughput
```

**Key Principle**: The same code runs in all three modes. Only the durability implementation changes. No Docker required for development or local production deployment.

**Rationale**: Custom orchestrator enables dynamic graphs. Durability abstraction allows same code to run anywhere: dev (memory) → local prod (ReDB) → cloud (Dapr).
### 11. Checkpoint and Recovery Strategy (State-Driven with Configurable Policy)
**Decision**: Pure state-driven architecture with configurable checkpoint policies. Users select the appropriate policy for their use case - from maximum durability to maximum performance.
**Rationale**: Different workflows have different durability requirements. A chat agent needs frequent checkpoints; a high-throughput ETL pipeline may prefer batch checkpointing. The policy is user-configurable, not hardcoded.

**State Structure**:
```rust
pub struct State {
    pub workflow_id: WorkflowId,
    pub current_node: NodeId,              // Where we are now
    pub action: Action,                    // Where to go next
    pub shared_store: HashMap<String, Value>, // Ephemeral node communication
    pub timestamp: DateTime<Utc>,
}
```

**Checkpoint Policy** (User-Configurable):
```rust
pub enum CheckpointPolicy {
    /// Checkpoint every N node transitions (default: 5)
    /// Good for: Chat agents, research loops, long-running workflows
    EveryN(usize),
    
    /// Only checkpoint at developer-marked safe points
    /// Good for: Multi-stage pipelines, ETL workflows
    SafePointsOnly,
    
    /// Only at explicit CheckpointNode in the graph
    /// Good for: Developer-controlled durability
    ExplicitOnly,
    
    /// Disable checkpointing (in-memory only)
    /// Good for: High-throughput, recoverable workflows, dev mode
    Disabled,
}

pub struct CheckpointConfig {
    pub policy: CheckpointPolicy,
    pub async_flush: bool,           // Buffer and flush async (opt-in)
    pub compression: bool,           // Compress checkpoint data
    pub max_buffer_size: usize,      // Max buffered checkpoints before forced flush
}
```

**SharedStore Persistence Options**:
```rust
pub enum SharedStorePolicy {
    /// Ephemeral - not persisted (default for performance)
    /// Requires nodes to be idempotent
    Ephemeral,
    
    /// Persisted - saved with checkpoints
    /// Adds overhead but survives crashes without re-execution
    Persisted,
    
    /// Selective - only persist specific keys marked with #[persist]
    Selective(HashSet<String>),
}
```

**Batch Checkpointing** (Opt-In):
- Disabled by default for maximum durability
- Enable via `CheckpointConfig::async_flush = true`
- Configurable buffer size and flush interval
- Risk: Up to `flush_interval` of work may be lost on crash

**Checkpoint History**:
- Checkpoints are append-only
- History retained for audit/debugging
- Recovery always uses the latest checkpoint
- Old checkpoints can be garbage collected based on policy

### 12. Nested Flow Execution and Serialization
**Decision**: Composite nodes (Flows) spawn inner orchestrators. Internal state for nodes that need to suspend execution MUST be persisted directly to the `SharedStore` and MUST implement `Serialize`/`Deserialize` (e.g. `serde_json`).
**Rationale**: Rust cannot serialize trait objects like `Arc<dyn Node>` into opaque blobs. By enforcing that all suspension state lives in the serializable `SharedStore`, the orchestrator only needs to serialize the top-level key-value store to pause/resume an entire workflow reliably.

**Architecture**:
```rust
pub struct State {
    pub workflow_id: WorkflowId,
    pub current_node: NodeId,
    pub action: Action,
    pub shared_store: HashMap<String, Value>, // MUST be serializable
    pub timestamp: DateTime<Utc>,
}
```

**Nested Flow Execution**:
- Parent orchestrator calls `node.run(&state)`
- Composite node spawns inner orchestrator
- Composite serializes inner state into `checkpoint_data`
- Parent checkpoints after composite completes
- Recovery: Parent resumes composite → Composite restores inner from checkpoint_data

**Implementation**:
```rust
// Node trait with default run() implementation
pub trait Node {
    // Default: handle resume vs exec automatically
    async fn run(&self, state: &State) -> Result<(Action, State)> {
        if state.checkpoint_data.is_empty() {
            self.exec(state).await
        } else {
            self.resume(state).await
        }
    }
    
    // Implement these
    async fn exec(&self, state: &State) -> Result<(Action, State)>;
    async fn resume(&self, state: &State) -> Result<(Action, State)>;
}

// Simple Node - stateless
impl Node for SimpleNode {
    async fn exec(&self, state: &State) -> Result<(Action, State)> {
        // Do work
        let mut new_state = state.clone();
        new_state.action = Action::Done;
        Ok((Action::Done, new_state))
    }
    
    async fn resume(&self, state: &State) -> Result<(Action, State)> {
        // Stateless nodes just re-run exec
        self.exec(state).await
    }
}

// Composite Node (Flow) - stateful
impl Node for Flow {
    async fn exec(&self, state: &State) -> Result<(Action, State)> {
        // Start fresh - create initial inner state
        let inner_state = InnerState::new(self.start_node);
        self.run_inner(state, inner_state).await
    }
    
    async fn resume(&self, state: &State) -> Result<(Action, State)> {
        // Restore from checkpoint_data
        let inner_state: InnerState = deserialize(&state.checkpoint_data)?;
        self.run_inner(state, inner_state).await
    }
    
    async fn run_inner(&self, parent_state: &State, inner_state: InnerState) -> Result<(Action, State)> {
        // Spawn inner orchestrator
        let inner = Orchestrator::new(self.durability.clone());
        
        // Run inner flow (may checkpoint internally)
        let (action, final_inner_state) = inner.run_with_state(inner_state).await?;
        
        // Serialize inner state for parent checkpoint
        let mut new_state = parent_state.clone();
        new_state.checkpoint_data = serialize(&final_inner_state)?;
        new_state.action = action;
        
        Ok((action, new_state))
    }
}
```

**Key Principle**: Orchestrator is completely agnostic. It just sees nodes and actions. Whether a node is simple or composite is an implementation detail.

### 12a. Node Idempotency Enforcement (Compile-Time)
**Decision**: All nodes must be either idempotent (safe to re-run) or implement side-effect tracking. This is enforced at compile time via the type system.
**Rationale**: When a workflow crashes and recovers, the last node must re-execute to repopulate SharedStore. If that node has side effects (API calls, DB writes), idempotency prevents duplicate operations. This is critical for correctness.

**Idempotency Trait System**:
```rust
/// Marker trait for idempotent nodes (safe to re-run)
pub trait Idempotent {}

/// Trait for nodes with side effects (not idempotent)
pub trait SideEffects {
    /// Generate deterministic idempotency key
    fn idempotency_key(&self, workflow_id: &WorkflowId, attempt: u32) -> String;
}

/// Node trait with compile-time idempotency checking
pub trait Node {
    type Input;
    type Output;
    
    async fn exec(&self, input: Self::Input) -> Result<Self::Output>;
    
    /// Must return true if node is idempotent
    fn is_idempotent(&self) -> bool;
    
    /// Must provide idempotency key if not idempotent
    fn idempotency_key(&self, ctx: &ExecutionContext) -> Option<String>;
}

/// Compile-time enforcement: Node must implement either Idempotent or SideEffects
pub struct NodeBuilder<T> {
    _phantom: PhantomData<T>,
}

impl<T: Idempotent> NodeBuilder<T> {
    pub fn build(self) -> Box<dyn Node> {
        // Automatically marked as idempotent
    }
}

impl<T: SideEffects> NodeBuilder<T> {
    pub fn build(self) -> Box<dyn Node> {
        // Must provide idempotency key generation
    }
}

// Example: Idempotent node
#[derive(Idempotent)]
pub struct TransformNode {
    transformation: Transformation,
}

impl Node for TransformNode {
    fn is_idempotent(&self) -> bool { true }
    fn idempotency_key(&self, _ctx: &ExecutionContext) -> Option<String> { None }
}

// Example: Non-idempotent node with side effects
pub struct PaymentNode {
    gateway: PaymentGateway,
}

impl SideEffects for PaymentNode {
    fn idempotency_key(&self, workflow_id: &WorkflowId, attempt: u32) -> String {
        // Deterministic key prevents duplicate charges
        format!("payment:{}:{}", workflow_id, attempt)
    }
}

impl Node for PaymentNode {
    fn is_idempotent(&self) -> bool { false }
    fn idempotency_key(&self, ctx: &ExecutionContext) -> Option<String> {
        Some(self.idempotency_key(&ctx.workflow_id, ctx.attempt))
    }
}
```

**Compile-Time Error Example**:
```rust
// This will FAIL to compile:
pub struct BadNode;

impl Node for BadNode {
    fn is_idempotent(&self) -> bool { false }
    // ERROR: Non-idempotent node must implement SideEffects trait
    //        or mark as Idempotent
}
```

**Helper Macros**:
```rust
// Derive macro for idempotent nodes
#[derive(Node, Idempotent)]
pub struct MyTransform {
    config: Config,
}

// Attribute macro for side-effect nodes
#[derive(Node)]
#[side_effects(key = "payment:{workflow_id}:{node_id}")]
pub struct MyPayment {
    gateway: PaymentGateway,
}
```

### 13. Async-By-Default
**Decision**: All node operations are async. No sync variants.
**Rationale**: Modern Rust async is the standard. Users can block if needed with `tokio::runtime::Handle::current().block_on()`.

### 13. Runtime Execution Mode
**Decision**: Use runtime capability flags in `ExecutionMode` struct.
**Structure**:
```rust
pub struct ExecutionMode {
    pub durability: DurabilityLevel,      // Distributed/Persistent/Volatile
    pub streaming: StreamingPolicy,       // Stream/Batch/Complete
    pub checkpoint_policy: CheckpointPolicy,
}

pub enum DurabilityLevel {
    InMemory,     // Default - zero dependencies, dev/testing
    ReDB,         // Single-user production - file-based persistence
    Dapr,         // Distributed - multi-tenant, HA, requires sidecar
    DaprRemote,   // Connect to remote Dapr sidecar via env vars
}

impl Default for DurabilityLevel {
    fn default() -> Self { DurabilityLevel::InMemory }
}
```
**Usage**: User specifies mode at workflow start; runtime enforces constraints.

### 14. Python Feature Parity: BatchFlow and AsyncBatchFlow
**Decision**: Implement all Python PocketFlow flow variants to ensure functional equivalence.
**Rationale**: Python PocketFlow provides BatchFlow and AsyncBatchFlow which are essential for batch processing workflows.
**Python Equivalents**:
```rust
/// BatchFlow - runs flow with batch parameter sets
/// (Python: lines 53-57 in pocketflow/__init__.py)
pub struct BatchFlow {
    start: Arc<RwLock<Node>>,
    params: HashMap<String, Value>,
}

impl BatchFlow {
    pub fn _run(&self, shared: &SharedStore) -> Option<String> {
        let prep = self.prep(shared).unwrap_or_default();
        for bp in prep {
            self._orch(shared, self.params.clone().merge(bp));
        }
        self.post(shared, prep, ())
    }
}

/// AsyncBatchFlow - async variant with sequential batch execution
/// (Python: lines 90-94 in pocketflow/__init__.py)
pub struct AsyncBatchFlow {
    start: Arc<RwLock<Node>>,
    params: HashMap<String, Value>,
}

impl AsyncBatchFlow {
    pub async fn _run_async(&self, shared: &SharedStore) -> Option<String> {
        let prep = self.prep_async(shared).await.unwrap_or_default();
        for bp in prep {
            self._orch_async(shared, self.params.clone().merge(bp)).await;
        }
        self.post_async(shared, prep, ()).await
    }
}

/// AsyncParallelBatchFlow - async variant with parallel batch execution
/// (Python: lines 96-100 in pocketflow/__init__.py)
pub struct AsyncParallelBatchFlow {
    start: Arc<RwLock<Node>>,
    params: HashMap<String, Value>,
}

impl AsyncParallelBatchFlow {
    pub async fn _run_async(&self, shared: &SharedStore) -> Option<String> {
        let prep = self.prep_async(shared).await.unwrap_or_default();
        let tasks: Vec<_> = prep.iter()
            .map(|bp| self._orch_async(shared, self.params.clone().merge(bp.clone())))
            .collect();
        futures::future::join_all(tasks).await;
        self.post_async(shared, prep, ()).await
    }
}
```

### 15. Node Idempotency: Exact Python Semantics
**Decision**: Match Python's exact retry/fallback behavior for feature parity.
**Rationale**: Python's exec_fallback is called ONLY after ALL retries are exhausted (line 33 in __init__.py). This must be preserved.
```rust
/// Match Python behavior exactly:
/// for self.cur_retry in range(self.max_retries):
///     try: return self.exec(prep_res)
///     except Exception as e:
///         if self.cur_retry==self.max_retries-1: return self.exec_fallback(prep_res,e)
///         if self.wait>0: time.sleep(self.wait)
pub fn _exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
    for self.cur_retry in 0..self.max_retries {
        match self.exec(prep_res).await {
            Ok(result) => return Ok(result),
            Err(e) if self.cur_retry == self.max_retries - 1 => {
                // Call fallback ONLY after ALL retries exhausted - matches Python
                return self.exec_fallback(prep_res, e).await;
            }
            Err(_) if self.wait > 0 => {
                tokio::time::sleep(Duration::from_secs(self.wait as u64)).await;
            }
            Err(e) => return Err(e), // Shouldn't reach here but handle gracefully
        }
    }
    unreachable!()
}
```

## Risks / Trade-offs

**Risk**: Dapr SDK for Rust may be immature → Mitigation: Use stable APIs, contribute upstream if needed.
**Risk**: Serialization overhead with `serde_json::Value` → Mitigation: Provide type-safe wrappers, optimize hot paths.
**Risk**: Behavioral differences between Python and Rust/Dapr → Mitigation: Comprehensive compatibility tests.
**Risk**: Dapr sidecar adds operational complexity → Mitigation: Provide LocalDurability for single-user deployments.
**Risk**: Custom orchestrator complexity → Mitigation: Start simple, add features incrementally.
**Risk**: Arc<RwLock<>> overhead for successors → Mitigation: Benchmark, optimize if needed.

## Migration Plan

1. Create crate structure with trait definitions (engine + durability)
2. Implement custom Orchestrator with dynamic graph support
3. Implement DaprDurability (state store, pub/sub, tracing, resiliency)
4. Implement DaprRemoteDurability (connect to remote sidecar via env vars)
5. Implement LocalDurability (ReDB/SQLite backend for single-user)
6. Implement InMemoryDurability (dev/testing)
7. Add retry, fallback, parallel execution features
8. Ensure BatchFlow, AsyncBatchFlow, AsyncParallelBatchFlow parity
9. Implement compile-time idempotency enforcement
10. Create compatibility test suite
11. Document migration path from Python PocketFlow

