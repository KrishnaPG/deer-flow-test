## Context

PocketFlow-Rust currently lacks enterprise features needed for production deployment. This design integrates Dapr's enterprise building blocks to add observability, durability, scalability, fail-safety, idempotency, tracability, and security. The integration builds on the core abstractions, design patterns, utilities, and cookbooks ported in previous changes.

We are taking a **fresh start approach**: build enterprise-grade features that Python PocketFlow cannot easily provide. Rust+Dapr enables features that would require external libraries in Python - retry/fallback built-in, native OpenTelemetry, Dapr-sidecar resilience patterns.

## Goals / Non-Goals

**Goals:**
- Integrate Dapr OpenTelemetry for distributed tracing and metrics
- Integrate Dapr State Management for durable workflow state (custom orchestrator uses Dapr)
- Leverage Dapr sidecar model for horizontal scaling and service discovery
- Implement Dapr Resiliency policies for fail-safety (retry, circuit breaker, timeout)
- Ensure idempotency through deterministic keys and deduplication
- Add distributed tracing and audit logging for tracability
- Integrate Dapr Secret Management and access control for security
- Provide enterprise features that Python requires external libraries for
- Support non-Dapr deployments via LocalDurability (SQLite) for single-user production
- Aim for zero-copy, zero-allocation where possible for performance

**Non-Goals:**
- Implement the actual integration (this is a plan only)
- Replace all existing PocketFlow-Rust code wholesale
- Provide GUI management interfaces
- Duplicate Python's limitations (we build better, not just different)

## Decisions

### 1. Observability: Adaptive Telemetry Sampling via OpenTelemetry
**Decision**: Use the battle-tested `tracing` crate in the core engine to emit standard spans. The specific execution driver (e.g., `dapr-driver`) is responsible for W3C TraceContext injection. Enforce **Adaptive Head-Based Sampling** (low normal rate, 100% on error/fallback).
**Rationale**: Leverages automatic trace propagation without coupling the core engine to OpenTelemetry SDKs, while preventing observability overhead from destroying performance at enterprise scale.
**Alternatives Considered**: 100% tracing (rejected - kills performance), Custom tracing macros (rejected - reinventing wheel).

### 2. Durability: Dapr State Management + Custom Orchestrator
**Decision**: Custom orchestrator uses Dapr State Management for checkpoints and recovery. Dapr provides persistence layer, not workflow engine.
**Rationale**: Custom orchestrator enables dynamic graphs (not possible with Dapr Workflows). Dapr State provides durability.
**Architecture**:
```
Custom Orchestrator
    ├── DaprDurability (distributed)
    │     ├── Dapr State Store (checkpoints)
    │     ├── Dapr Pub/Sub (messaging)
    │     └── Dapr Resiliency (retry/CB)
    ├── LocalDurability (single-user)
    │     └── SQLite (checkpoints)
    └── InMemoryDurability (dev)
          └── HashMap (no persistence)
```
**Alternatives Considered**: Dapr Workflows (rejected - static graphs only), Custom persistence layer (rejected - reinventing wheel).

### 3. Scalability: Dapr Sidecar + Kubernetes
**Decision**: Deploy PocketFlow workflows as Dapr-enabled services on Kubernetes.
**Rationale**: Leverages Dapr's service discovery, load balancing, and Kubernetes orchestration.
**Alternatives Considered**: Custom scaling logic (rejected - complex), single-node deployment (rejected - not scalable).

### 4. Fail-safety: Technical vs. Semantic Resilience
**Decision**: Define Dapr resiliency policies (retry, circuit breaker, timeout) exclusively for *technical* failures (e.g., sidecar networking issues, temporary 500s). For *semantic* resilience (e.g., fallback logic, switching LLM providers), handle routing via PocketFlow's dynamic Node Action returns (`Action::Fallback`).
**Rationale**: Separates infrastructure flakiness from business logic fallbacks. Relying on Dapr Compensation handlers for business logic alternatives is an anti-pattern. Dapr handles the network layer; Rust handles the application logic layer.
**Alternatives Considered**: Conflating technical and semantic fallbacks (rejected - makes code hard to reason about and misuses Saga compensations), Custom retry logic (rejected - scattered).

### 5. Idempotency: Deterministic Keys & Deduplication
**Decision**: Leverage Dapr Workflow's idempotency, but the Rust core must automatically generate a deterministic UUID (hashed from `Workflow_ID + Node_ID + Attempt_Count`) and inject it into external headers as an Idempotency-Key.
**Rationale**: Dapr replays activities if they fail. To protect external systems (e.g., charging APIs, database inserts) from double-execution, a cryptographic idempotency key is mathematically required.
**Alternatives Considered**: Relying solely on Dapr Activity at-least-once guarantees without keys (rejected - causes dangerous side effects).

### 6. Tracability: Distributed Tracing + Audit Logs
**Decision**: Use Dapr's distributed tracing with custom audit logs for compliance.
**Rationale**: Automatic trace propagation, custom spans for PocketFlow operations, audit logs for lineage.
**Alternatives Considered**: Custom tracing (rejected - complex), no tracability (rejected - compliance risk).

### 7. Security: Dapr Security Features & Centralized Config
**Decision**: Use Dapr secret stores for credentials, ACLs for access control, mTLS for encryption. Enforce that *all* secret retrieval routes exclusively through a centralized `config` module, preventing random `env::var` usage across nodes.
**Rationale**: Leverages Dapr's security features without custom implementation, while ensuring the execution engine remains completely agnostic of how secrets are fetched.
**Alternatives Considered**: Custom security (rejected - reinventing wheel), Direct `.env` parsing (rejected - security risk).

### 8. Compliance & Benchmarking
**Decision**: Meet strict enterprise compliance standards (SOC2, GDPR) by enforcing Data Isolation (multi-tenancy via keys/namespaces), Right-to-be-forgotten (State Store TTLs, PII redaction filters before tracing), and Audit trails (Event Sourcing logs on DAG transitions). For benchmarks, measure explicit **Driver Overhead**: compare `berg10-execution-engine` latency running on the `in-memory-driver` vs the `dapr-driver` to isolate exact serialization and network costs.
**Rationale**: Compliance is mandatory for enterprise adoption. Driver benchmarking objectively proves the efficiency of the implementation.

### 9. Checkpoint Policy: Configurable Frequency
**Decision**: Configurable checkpoint policy with default every 5 node transitions.
**Policy Options**:
```rust
pub enum CheckpointPolicy {
    EveryN(usize),      // Default: EveryN(5)
    SafePointsOnly,     // Only at marked safe points
    ExplicitOnly,       // Only explicit checkpoint nodes
}
```
**Rationale**: Balance durability with performance. Long-running workflows (chat, research) need frequent checkpoints; short workflows don't.
**Use Cases**:
- **EveryN(5)**: Chat agents, research loops (frequent recovery points)
- **SafePointsOnly**: Multi-stage pipelines (checkpoint at stage boundaries)
- **ExplicitOnly**: Developer-controlled checkpoints

### 10. HITL (Human-in-the-Loop) Support
**Decision**: Implement durable suspension points for human-in-the-loop workflows with timeout handling and resume capabilities.
**Rationale**: Required for production workflows requiring human approval, review, or input. Must survive process restarts.
**Architecture**:
```rust
pub struct SuspensionPoint {
    pub workflow_id: WorkflowId,
    pub node_id: NodeId,
    pub suspended_at: DateTime<Utc>,
    pub timeout_at: Option<DateTime<Utc>>,
    pub assigned_to: Option<UserId>,
    pub context: Vec<u8>, // Serialized state for UI rendering
    pub human_input_schema: JsonSchema, // Expected input format
    pub status: SuspensionStatus,
}

pub enum SuspensionStatus {
    AwaitingInput,
    TimedOut,
    Cancelled,
    Resumed,
}

pub trait HITLDurability {
    /// Suspend workflow, release execution resources
    async fn suspend(&self, point: SuspensionPoint) -> Result<SuspensionId>;
    
    /// Resume workflow with human input
    async fn resume(&self, suspension_id: SuspensionId, input: HumanInput) -> Result<()>;
    
    /// Query suspended workflows (for UI/admin)
    async fn query_suspended(&self, filters: HITLFilters) -> Result<Vec<SuspensionPoint>>;
    
    /// Cancel suspended workflow
    async fn cancel(&self, suspension_id: SuspensionId, reason: &str) -> Result<()>;
    
    /// Handle timeout - automatically resume with fallback
    async fn handle_timeout(&self, suspension_id: SuspensionId) -> Result<()>;
}

pub struct HITLNode {
    pub prompt_template: String,
    pub timeout: Option<Duration>,
    pub fallback_action: Option<String>, // Action if timeout
    pub assigned_users: Vec<UserId>,
    pub input_schema: JsonSchema,
}
```
**Timeout Handling**:
- If timeout specified and expires, execute fallback_action
- If no timeout, suspend indefinitely
- Cancellation possible via API
- Timeout checks run via Dapr Actor reminders or background task
**Use Cases**: CLI HITL, FastAPI review endpoints, Gradio web forms, Streamlit FSM

### 11. Chat Session Persistence
**Decision**: Support durable conversation state that persists across sessions with session affinity.
**Rationale**: Chat agents (pocketflow-chat, pocketflow-chat-memory) need conversation history to survive restarts and be available across multiple interactions.
**Architecture**:
```rust
pub struct ChatSession {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub metadata: HashMap<String, Value>,
    pub vector_index: Option<VectorIndex>, // For chat-memory
}

pub trait ChatDurability {
    /// Save chat session
    async fn save_session(&self, session: &ChatSession) -> Result<()>;
    
    /// Load chat session by ID
    async fn load_session(&self, session_id: SessionId) -> Result<Option<ChatSession>>;
    
    /// Load all sessions for user
    async fn load_user_sessions(&self, user_id: UserId) -> Result<Vec<ChatSession>>;
    
    /// Archive old sessions
    async fn archive_sessions(&self, before: DateTime<Utc>) -> Result<u64>;
    
    /// Update session with new message
    async fn append_message(&self, session_id: SessionId, message: Message) -> Result<()>;
}
```
**Features**:
- Automatic session persistence after each turn
- Vector memory support for semantic search (chat-memory)
- Session TTL for cleanup
- Archive/compression for old sessions
**Storage Backends**: Dapr State (distributed), SQLite (single-user)

### 12. Code Execution Sandboxing
**Decision**: Implement sandboxed code execution for pocketflow-coding-agent with security controls.
**Rationale**: Code execution is inherently dangerous; must be isolated and controlled.
**Architecture**:
```rust
pub struct CodeExecutionConfig {
    pub timeout: Duration,
    pub max_memory_mb: usize,
    pub allowed_packages: Vec<String>,
    pub network_access: bool,
    pub filesystem_access: FileSystemAccess,
    pub sandbox_type: SandboxType,
}

pub enum SandboxType {
    Docker,      // Docker container isolation
    Firecracker, // MicroVM isolation
    WASI,        // WebAssembly sandbox
    Seccomp,     // Linux seccomp-bpf
}

pub trait CodeExecutor {
    async fn execute(&self, code: &str, config: &CodeExecutionConfig) -> Result<ExecutionResult>;
}
```
**Security Measures**:
- Resource limits (CPU, memory, time)
- Network isolation
- Filesystem sandboxing
- Package whitelist
- Audit logging
**Required For**: pocketflow-coding-agent

### 13. Connection Pooling
**Decision**: Implement connection pooling for database and external service integrations.
**Rationale**: Tool integrations (pocketflow-tool-database, web crawling) need efficient connection reuse.
**Architecture**:
```rust
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

pub trait PooledConnection {
    type Connection;
    async fn acquire(&self) -> Result<PooledConn<Self::Connection>>;
    async fn health_check(&self) -> Result<bool>;
}
```
**Implementations**: SQLx (databases), reqwest (HTTP), deadpool (generic)

### 14. Heartbeat / Always-On Pattern
**Decision**: Support continuous execution with graceful checkpointing between iterations.
**Rationale**: pocketflow-heartbeat requires always-on monitoring agents that checkpoint between cycles without full workflow completion.
**Architecture**:
```rust
pub struct HeartbeatConfig {
    pub interval: Duration,
    pub checkpoint_every: usize, // Checkpoint every N iterations
    pub graceful_shutdown_timeout: Duration,
}

pub trait HeartbeatNode: Node {
    async fn heartbeat(&self, iteration: u64) -> Result<HeartbeatAction>;
}

pub enum HeartbeatAction {
    Continue,
    Pause,      // Checkpoint and wait
    Shutdown,   // Graceful shutdown
}
```
**Features**:
- Checkpoint between iterations without workflow completion
- Graceful shutdown on SIGTERM with state preservation
- Resume from last iteration on restart
- Signal handling for pause/resume

**Risk**: Dapr sidecar adds latency → Mitigation: Optimize Dapr configuration, use LocalDurability for development.
**Risk**: Dapr configuration complexity → Mitigation: Provide templates and automation.
**Risk**: Kubernetes operational overhead → Mitigation: Use managed Kubernetes services or LocalDurability for simpler deployments.
**Risk**: Vendor lock-in to Dapr → Mitigation: Durability trait abstracts Dapr, supports LocalDurability (SQLite) alternative.

## Migration Plan

1. Create Durability trait with DaprDurability implementation
2. Create Dapr component configurations for each building block
3. Integrate OpenTelemetry instrumentation in Rust code
4. Implement custom orchestrator with Dapr State Management for checkpoints
5. Implement LocalDurability with SQLite for single-user production
6. Define and apply Dapr resiliency policies
7. Implement configurable checkpoint policy (EveryN, SafePointsOnly, ExplicitOnly)
8. Implement idempotency with deterministic keys and deduplication
9. Implement HITL support with suspension/resume API
10. Implement Chat Session Persistence with vector memory support
11. Implement Code Execution Sandboxing
12. Implement Connection Pooling for external services
13. Implement Heartbeat/Always-On pattern
14. Set up distributed tracing and audit logging
15. Configure Dapr security features
16. Create deployment templates for Kubernetes (Dapr) and standalone (LocalDurability)
17. Establish monitoring and alerting dashboards

