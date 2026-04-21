## Context

PocketFlow-Rust has basic utility implementations but lacks Dapr integration and enterprise features. This design maps each utility to Dapr building blocks to achieve cached, rate-limited, and observable operations. The utilities build on the core abstractions and design patterns ported in previous changes.

We are taking a **fresh start approach**: use Python PocketFlow utilities as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust utilities should do everything Python can do, but better - with caching, rate limiting, and observability built-in.

## Goals / Non-Goals

**Goals:**
- Map LLM calls to Dapr Conversation API for centralized LLM management
- Support streaming LLM responses where providers support it
- Map embedding to Dapr Conversation API + local Rust implementations
- Map vector DB operations to Dapr Vector bindings
- Map web search to Dapr bindings
- Implement chunking as pure Rust (no Dapr mapping needed)
- Implement visualization as pure Rust with Dapr State caching
- Map text-to-speech to Dapr bindings with streaming support
- **Add speech-to-text (STT) utility** - required for voice-chat cookbook
- Add caching, rate limiting, and observability to all utilities
- Prioritize working implementation over line-by-line Python equivalence
- Aim for zero-copy where possible for performance

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Replace all Python utility functionality wholesale
- Support every possible LLM provider or vector database
- Achieve line-by-line code equivalence with Python (not required)

## Decisions

### 1. LLM Calls: Dapr Conversation API
**Decision**: Use Dapr Conversation API for LLM calls with fallback to direct HTTP.
**Rationale**: Centralized LLM management, caching, rate limiting, observability.
**Alternatives Considered**: Direct HTTP calls only (rejected - lacks enterprise features), custom LLM client (rejected - reinventing wheel).

### 2. Embedding: Hybrid Approach & Connection Pooling
**Decision**: Use Dapr Conversation API for cloud embeddings, local Rust crates for local models. For local execution targeting local GPUs (e.g., via `vLLM` or `candle`), mandate a high-performance connection pool (like `deadpool`) to prevent port/connection exhaustion.
**Rationale**: Flexibility for offline use and cloud deployment. Connection pooling ensures reliability under high concurrent throughput.
**Alternatives Considered**: Cloud-only (rejected - no offline support), Unpooled local connections (rejected - crashes under load).

### 3. Vector DB: Pluggable Backends with Pure-Rust Local First
**Decision**: Implement VectorStore trait with pluggable backends. For local production WITHOUT Dapr sidecar, use `embedvec`. For cloud/distributed, use Dapr Vector bindings.
**Rationale**: Enables RAG pattern to work without Docker/Dapr and without C++ heavy dependencies like sqlite-vss. `embedvec` runs locally in pure Rust with SIMD acceleration and `sled` persistence. Pluggable design allows swapping backends.
**Implementation**:
```rust
pub trait VectorStore: Send + Sync {
    /// Insert or update vectors with metadata
    async fn upsert(&self, id: &str, embedding: &[f32], metadata: Value) -> Result<()>;
    
    /// Search for similar vectors
    async fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>>;
    
    /// Delete vectors by ID
    async fn delete(&self, id: &str) -> Result<()>;
}

/// Local vector store - pure Rust, no C++ dependencies
/// Uses embedvec with sled persistence
pub struct EmbedVecStore {
    db: embedvec::EmbedVec,
}

impl VectorStore for EmbedVecStore {
    async fn upsert(&self, id: &str, embedding: &[f32], metadata: Value) -> Result<()> {
        // ... embedvec insert logic ...
        Ok(())
    }
    
    async fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>> {
        // ... embedvec search logic ...
        Ok(results)
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        // ... embedvec delete logic ...
        Ok(())
    }
}

/// Dapr Vector store - for cloud/distributed deployments
pub struct DaprVectorStore {
    component_name: String,
    client: DaprClient,
}

impl VectorStore for DaprVectorStore {
    // Uses Dapr Vector binding API
}
```

**Local vs Remote Vector DB Decision Tree**:
```
┌─────────────────────────────────────────────────────────┐
│            Vector DB Backend Selection                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Is DAPR_HTTP_ENDPOINT set?                            │
│         │                                             │
│    YES──┼──NO                                          │
│         │                                              │
│         ▼                                              ▼
│  ┌──────────────┐      ┌──────────────┐              │
│  │ Dapr Vector  │      │   embedvec   │              │
│  │  Bindings    │      │  (Local)     │              │
│  └──────────────┘      └──────────────┘              │
│         │                     │                      │
│         └──────────┬──────────┘                      │
│                    ▼                                  │
│         ┌─────────────────────┐                     │
│         │  Same VectorStore   │                     │
│         │  Trait Interface   │                     │
│         └─────────────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

**Alternatives Considered**: Direct vector DB clients (rejected - more code), custom abstraction (rejected - reinventing wheel).

### 4. Web Search: Dapr Bindings
**Decision**: Use Dapr bindings for web search services.
**Rationale**: Centralized search API, caching, rate limiting.
**Alternatives Considered**: Direct API calls (rejected - lacks enterprise features), custom search client (rejected - reinventing wheel).

### 5. Chunking: SIMD-Accelerated Pure Rust
**Decision**: Implement chunking as pure Rust functions, heavily relying on **SIMD-optimized crates** (like `memchr` or `aho-corasick`) for text splitting.
**Rationale**: Local operation without distributed runtime overhead. SIMD reduces gigabyte-scale document chunking from seconds to milliseconds.
**Alternatives Considered**: Dapr mapping (rejected - unnecessary complexity), Standard string splitting (rejected - leaves massive performance on the table).

### 6. Visualization: Pure Rust + Dapr State Caching
**Decision**: Generate visualizations locally, cache results in Dapr State.
**Rationale**: Visualization is local rendering; caching can be distributed.
**Alternatives Considered**: Dapr rendering (rejected - overkill), no caching (rejected - performance).

### 7. Streaming with Built-in Backpressure Handling
**Decision**: Implement streaming support with built-in backpressure handling for improved DX. No durable streaming - clients handle their own durability with resend/retry logic.
**Rationale**: Built-in backpressure prevents overwhelming consumers. No durability guarantees during streaming (out of scope). Clients are responsible for their own reliability.
**Implementation**:
```rust
pub trait StreamingClient {
    type Item;
    
    /// Stream with backpressure handling
    fn stream_with_backpressure(
        &self,
        request: Request,
        config: StreamConfig,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Item>> + Send>>;
}

pub struct StreamConfig {
    /// Buffer size before backpressure kicks in
    pub buffer_size: usize,
    /// Timeout for consumer to process item
    pub consumer_timeout: Duration,
    /// Backpressure strategy
    pub backpressure_strategy: BackpressureStrategy,
    /// Failure policy when buffer overflows
    pub failure_policy: StreamFailurePolicy,
}

pub enum BackpressureStrategy {
    /// Block producer until consumer catches up
    Block,
    /// Drop oldest items in buffer
    DropOldest,
    /// Drop newest items (reject new)
    DropNewest,
}

pub enum StreamFailurePolicy {
    /// Fail the entire stream
    Fail,
    /// Continue streaming, log errors
    Continue,
    /// Custom handler
    Custom(Box<dyn Fn(StreamError) -> StreamAction>),
}

pub enum StreamAction {
    Continue,
    Abort,
    Retry,
}
```

**Key Principles**:
- Built-in backpressure handling (not optional)
- Configurable buffer sizes and strategies
- NO durable streaming - clients handle retries
- Failure policies are configurable per stream

### 8. Tiered Caching Strategy (L1 + L2)
**Decision**: Build a tiered cache architecture. Use an in-memory Rust `moka` cache for ultra-low latency L1 caching of frequent utility calls. On an L1 miss, fall back to Dapr State Management (L2 cache).
**Rationale**: Achieves microsecond latency for hot data (like frequently used embeddings or prompt templates) while maintaining distributed consistency and fault tolerance via the sidecar. Emit OpenTelemetry spans for hit/miss ratios.
**Alternatives Considered**: Dapr state cache only (rejected - adds sidecar latency to hot-path data), In-memory only (rejected - lacks distributed persistence).

### 9. Battle-Tested OSS & Centralized Secret Management
**Decision**: Use **no custom code** where battle-tested open-source packages exist. For example, use `reqwest` for HTTP, `moka` for L1 caching, and `deadpool` for connection pools. Additionally, all API keys and environment parameters must be resolved through a single centralized `config` module backed by the Dapr Secret Store API.
**Rationale**: Reinforces enterprise stability. Utilities must never manually parse `.env` files or attempt to re-implement complex low-level mechanics like LRU caches or connection pools.

### 12. Connection Pooling for External Services
**Decision**: Implement connection pooling for database and external service integrations to handle high concurrent load.
**Rationale**: Tool integrations (pocketflow-tool-database, web crawling) need efficient connection reuse. Prevents connection exhaustion and improves performance.
**Architecture**:
```rust
pub struct ConnectionPoolConfig {
    pub max_connections: usize,           // Maximum pool size
    pub min_connections: usize,           // Minimum idle connections
    pub connection_timeout: Duration,     // Timeout for acquiring connection
    pub idle_timeout: Duration,           // How long to keep idle connections
    pub max_lifetime: Duration,           // Maximum connection lifetime
    pub health_check_interval: Duration,  // How often to check connection health
}

pub trait PooledConnectionManager {
    type Connection;
    
    /// Acquire connection from pool
    async fn acquire(&self) -> Result<PooledConnection<Self::Connection>>;
    
    /// Return connection to pool
    fn release(&self, conn: PooledConnection<Self::Connection>);
    
    /// Health check all connections
    async fn health_check(&self) -> Result<PoolHealth>;
    
    /// Get pool statistics
    fn stats(&self) -> PoolStats;
}

pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub wait_queue_length: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
}

/// Database connection pooling (SQLx)
pub struct DatabasePool {
    pool: sqlx::Pool<Postgres>, // or MySql, Sqlite
}

/// HTTP connection pooling (reqwest)
pub struct HttpPool {
    client: reqwest::Client, // Has built-in pooling
}

/// Generic connection pooling (deadpool)
pub struct GenericPool<T> {
    pool: deadpool::Pool<T>,
}
```

**Integration Points**:
- **Database**: SQLx with connection pooling for pocketflow-tool-database
- **HTTP/REST**: reqwest client with keep-alive for web crawling
- **WebSocket**: Dedicated pool for persistent connections
- **gRPC**: tonic client with channel pooling
- **LLM APIs**: Connection pool for OpenAI, Anthropic APIs

**Configuration**:
```yaml
connection_pools:
  database:
    max_connections: 20
    min_connections: 5
    connection_timeout: 30s
    idle_timeout: 600s
  
  http:
    max_connections: 100
    connection_timeout: 10s
    idle_timeout: 90s
    
  websocket:
    max_connections: 50
    connection_timeout: 5s
    idle_timeout: 300s
```

**Required For**: 
- pocketflow-tool-database (DB connection pooling)
- pocketflow-tool-crawler (HTTP connection reuse)
- pocketflow-fastapi-websocket (WebSocket connection management)
- All LLM utility clients (API connection pooling)

### 11. Streaming Support Matrix (with Backpressure)
**Decision**: Support streaming for utilities where LLM providers support it. All streaming includes built-in backpressure handling.
**Streaming Matrix**:
| Utility | Streaming Support | Backpressure | Notes |
|---------|------------------|--------------|-------|
| OpenAI Chat | ✅ | Built-in | Native streaming API |
| Anthropic Claude | ✅ | Built-in | Native streaming API |
| Local LLMs | ✅ | Built-in | Via generation callbacks |
| Embeddings | ❌ | N/A | Complete only |
| Web Search | ❌ | N/A | Atomic results |

**Backpressure Configuration**:
```rust
// Default backpressure config
let stream_config = StreamConfig {
    buffer_size: 100,           // Items
    consumer_timeout: 30s,      // Per item
    backpressure_strategy: BackpressureStrategy::Block,
    failure_policy: StreamFailurePolicy::Fail,
};

// Custom per-utility
let llm_stream = llm_client
    .stream_with_backpressure(request, stream_config)
    .await?;
```

**Rationale**: Streaming reduces time-to-first-byte (TTFB). Built-in backpressure improves DX. No durability during streaming (out of scope).

## Risks / Trade-offs

**Risk**: Dapr Conversation API may not support all LLM providers → Mitigation: Fallback to direct HTTP calls.
**Risk**: Dapr bindings may have limited feature sets → Mitigation: Allow custom implementations for advanced features.
**Risk**: Caching may introduce staleness → Mitigation: Configurable TTL and cache invalidation.
**Risk**: Local embedding models may be large → Mitigation: Provide model downloading and caching.
**Risk**: Streaming increases complexity → Mitigation: Make streaming opt-in via `StreamingPolicy`.
**Risk**: STT integration may vary by provider → Mitigation: Abstract with trait, support multiple backends.

## Migration Plan

1. Create utility trait definitions
2. Implement Dapr Conversation API LLM client with streaming support
3. Implement hybrid embedding client
4. Implement Dapr Vector binding client
5. Implement Dapr web search binding client
6. Implement chunking utilities
7. Implement visualization with Dapr State caching
8. Implement streaming with built-in backpressure handling
9. Implement connection pooling for databases and HTTP services
10. Add caching layer across all utilities
11. Create compatibility test suite
12. Test streaming functionality with backpressure
13. Test connection pooling under high concurrent load

