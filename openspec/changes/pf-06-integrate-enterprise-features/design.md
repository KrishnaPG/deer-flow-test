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
    ├── DaprRemoteDurability (hybrid - local app + remote sidecar)
    │     └── Dapr sidecar via DAPR_HTTP_ENDPOINT
    ├── LocalDurability (single-user)
    │     └── ReDB/SQLite (checkpoints)
    └── InMemoryDurability (dev)
          └── HashMap (no persistence)
```
**Full Deployment Matrix**:
| Mode | Dapr Sidecar | Local DB | Use Case |
|------|-------------|----------|---------|
| InMemory | None | None | Development |
| ReDB | None | ReDB file | Local production |
| DaprRemote | Remote via env var | Optional | Hybrid deployment |
| Dapr | Local/ Kubernetes | Optional | Full cloud |
| SqliteVss | None | ReDB + sqlite-vss | Local with Vector |

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

### 9. Checkpoint Policy: User-Configurable with Opt-In Batch Mode
**Decision**: User-configurable checkpoint policy with opt-in batch checkpointing for high-throughput scenarios. Default is synchronous checkpointing for maximum durability.
**Policy Options**:
```rust
pub enum CheckpointPolicy {
    EveryN(usize),      // Default: EveryN(5)
    SafePointsOnly,     // Only at marked safe points
    ExplicitOnly,       // Only explicit checkpoint nodes
    Disabled,           // No checkpointing (in-memory only)
}

pub struct CheckpointConfig {
    pub policy: CheckpointPolicy,
    pub batch_mode: BatchCheckpointConfig, // Opt-in only
}

/// Batch checkpointing - OPT-IN for high-throughput scenarios
/// Trade-off: Up to flush_interval of work may be lost on crash
pub struct BatchCheckpointConfig {
    pub enabled: bool,              // Disabled by default
    pub flush_interval: Duration,   // How often to flush buffer
    pub max_buffer_size: usize,     // Max buffered checkpoints
    pub compression: bool,          // Compress checkpoint data
}

impl Default for BatchCheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // OPT-IN: Disabled by default
            flush_interval: Duration::from_secs(5),
            max_buffer_size: 100,
            compression: true,
        }
    }
}
```
**Rationale**: Different workflows have different durability needs. Batch checkpointing improves performance but risks losing recent work on crash. Users must explicitly opt-in to this trade-off.
**Use Cases**:
- **EveryN(5)**: Chat agents, research loops (frequent recovery points)
- **SafePointsOnly**: Multi-stage pipelines (checkpoint at stage boundaries)
- **ExplicitOnly**: Developer-controlled checkpoints
- **Disabled**: High-throughput ETL where replay is acceptable
- **Batch Mode (Opt-In)**: High-throughput scenarios where losing a few seconds of work is acceptable

**Batch Checkpointing Flow**:
```
Synchronous (Default - Maximum Durability):
┌─────────┐    ┌─────────────┐    ┌──────────────┐
│ Execute │───▶│ Checkpoint  │───▶│   Next Node  │
│  Node   │    │  (sync)     │    │              │
└─────────┘    └─────────────┘    └──────────────┘

Batch (Opt-In - Higher Performance):
┌─────────┐    ┌─────────────┐    ┌──────────────┐
│ Execute │───▶│  Buffer CP  │───▶│   Next Node  │
│  Node   │    │  (async)    │    │              │
└─────────┘    └─────────────┘    └──────────────┘
       │
       │ (Every N seconds)
       ▼
┌──────────────┐
│  Flush Batch │
│  to Storage  │
└──────────────┘
```

**Warning**: When batch mode is enabled, up to `flush_interval` seconds of work may be lost on unexpected process termination.

### 10. HITL (Human-in-the-Loop) Support - UI Agnostic
**Decision**: Implement UI-agnostic HITL where the workflow enters a "waiting for human" state and resumes when input arrives from any source (pub/sub, POST, webhook, CLI, etc.). The orchestrator does not know or care how the human provided input.
**Rationale**: HITL should not be coupled to any specific UI technology. Input could come from a web form, Slack message, email, CLI prompt, or external system webhook. The workflow simply waits for an event.

**Architecture**:
```rust
/// HITL state - workflow is suspended waiting for human input
pub struct SuspensionPoint {
    pub workflow_id: WorkflowId,
    pub node_id: NodeId,
    pub suspended_at: DateTime<Utc>,
    pub timeout_at: Option<DateTime<Utc>>,
    pub context: Vec<u8>, // Serialized state for UI rendering
    pub expected_input_type: InputType, // What kind of input is expected
    pub status: SuspensionStatus,
}

pub enum SuspensionStatus {
    AwaitingInput,
    TimedOut,
    Cancelled,
    Resumed,
}

/// Input can come from any source - UI agnostic
pub enum HumanInput {
    Text(String),
    Json(Value),
    Approval { approved: bool, comment: Option<String> },
    FormData(HashMap<String, String>),
    Binary(Vec<u8>),
}

/// UI-agnostic HITL trait
pub trait HITLDurability {
    /// Suspend workflow, release execution resources
    async fn suspend(&self, point: SuspensionPoint) -> Result<SuspensionId>;
    
    /// Resume workflow with human input (from any source)
    async fn resume(&self, suspension_id: SuspensionId, input: HumanInput) -> Result<()>;
    
    /// Query suspended workflows (for UI/admin)
    async fn query_suspended(&self, filters: HITLFilters) -> Result<Vec<SuspensionPoint>>;
    
    /// Cancel suspended workflow
    async fn cancel(&self, suspension_id: SuspensionId, reason: &str) -> Result<()>;
}

/// HITL Node - defines what input is needed, not how it's collected
pub struct HITLNode {
    pub prompt_context: String, // Context shown to human
    pub timeout: Option<Duration>,
    pub fallback_action: Option<String>, // Action if timeout
    pub expected_input: InputType,
}

pub enum InputType {
    Text,
    Approval,
    JsonSchema(JsonSchema),
    Form(Vec<FormField>),
}
```

**Input Sources** (UI Agnostic):
```rust
/// Input can arrive via any mechanism
pub enum InputChannel {
    /// Pub/Sub message (e.g., from Slack, email)
    PubSub { topic: String },
    
    /// HTTP POST (e.g., webhook, form submission)
    HttpPost { endpoint: String },
    
    /// CLI input (for local/development use)
    Cli,
    
    /// Custom channel (any other source)
    Custom(Box<dyn InputReceiver>),
}

/// Trait for receiving input from any source
pub trait InputReceiver: Send + Sync {
    async fn receive(&self, suspension_id: SuspensionId) -> Result<HumanInput>;
}
```

**Workflow Flow**:
```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Execute   │────▶│  HITLNode.exec() │────▶│   Suspend       │
│   HITLNode  │     │                  │     │   (checkpoint)  │
└─────────────┘     └──────────────────┘     └────────┬────────┘
                                                      │
                                                      ▼
                                           ┌──────────────────┐
                                           │  Awaiting Input  │
                                           │  (UI agnostic)   │
                                           └────────┬─────────┘
                                                    │
              ┌─────────────────────────────────────┼─────────────────────────────────────┐
              │                                     │                                     │
              ▼                                     ▼                                     ▼
    ┌──────────────────┐              ┌──────────────────┐              ┌──────────────────┐
    │   Pub/Sub        │              │   HTTP POST      │              │   CLI Input      │
    │   (Slack/email)  │              │   (webhook)      │              │   (local dev)    │
    └────────┬─────────┘              └────────┬─────────┘              └────────┬─────────┘
             │                                 │                                  │
             └─────────────────────────────────┼──────────────────────────────────┘
                                               │
                                               ▼
                                    ┌──────────────────┐
                                    │  Resume Workflow │
                                    │  (load from      │
                                    │   checkpoint)    │
                                    └──────────────────┘
```

**Example Implementations**:
```rust
/// Webhook input receiver
pub struct WebhookReceiver {
    server: HttpServer,
}

impl InputReceiver for WebhookReceiver {
    async fn receive(&self, suspension_id: SuspensionId) -> Result<HumanInput> {
        // Wait for HTTP POST to /resume/{suspension_id}
        let body = self.server.wait_for_post(&format!("/resume/{}", suspension_id)).await?;
        Ok(HumanInput::Json(body))
    }
}

/// CLI input receiver (for local development)
pub struct CliReceiver;

impl InputReceiver for CliReceiver {
    async fn receive(&self, suspension_id: SuspensionId) -> Result<HumanInput> {
        println!("Workflow {} needs input:", suspension_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(HumanInput::Text(input.trim().to_string()))
    }
}

/// Pub/Sub input receiver
pub struct PubSubReceiver {
    subscriber: Subscriber,
}

impl InputReceiver for PubSubReceiver {
    async fn receive(&self, suspension_id: SuspensionId) -> Result<HumanInput> {
        let message = self.subscriber.receive(format!("hitl:{}", suspension_id)).await?;
        Ok(HumanInput::Json(message.payload))
    }
}
```

**Three-Tier HITL Support**:
```rust
pub enum HITLMode {
    /// File-based suspension state (dev/local)
    /// Uses local filesystem for storage
    LocalFile,
    
    /// SQLite-based with optional webhook server
    /// Single-user with web UI support
    LocalWeb,
    
    /// Full Dapr Actor with timeout coordination
    /// Multi-tenant, distributed, with Actor reminders
    Distributed,
}
```

**Timeout Handling**:
- If timeout specified, workflow resumes with fallback after expiration
- Timeout checks run via:
  - Local: Background tokio task
  - Distributed: Dapr Actor reminders
- Cancellation possible via API

**Key Principle**: The orchestrator is completely UI-agnostic. It waits for an input event and doesn't care whether it came from a Slack message, web form, email, or CLI prompt.

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

