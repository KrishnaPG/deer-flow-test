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

### 3. Vector DB: Dapr Bindings
**Decision**: Use Dapr Vector bindings for vector database operations.
**Rationale**: Pluggable backends, consistent API across providers.
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

### 7. Text-to-Speech: Dapr Bindings with Streaming
**Decision**: Use Dapr bindings for TTS services. Support streaming audio chunks for real-time playback.
**Rationale**: Pluggable TTS providers, caching, rate limiting. Streaming enables responsive voice interfaces.
**Implementation**:
```rust
pub trait TTSClient {
    /// Generate complete audio
    async fn synthesize(&self, text: &str) -> Result<AudioData>;
    
    /// Stream audio chunks for real-time playback
    fn synthesize_stream(&self, text: &str) -> Pin<Box<dyn Stream<Item = Result<AudioChunk>> + Send>>;
}
```

### 10. Speech-to-Text: Dapr Bindings with Streaming
**Decision**: Use Dapr bindings for STT services. Support streaming transcription for real-time voice input.
**Rationale**: Required for `pocketflow-voice-chat` cookbook. Real-time transcription enables conversational voice interfaces.
**Implementation**:
```rust
pub trait STTClient {
    /// Transcribe complete audio
    async fn transcribe(&self, audio: &AudioData) -> Result<String>;
    
    /// Stream transcription results for real-time voice input
    fn transcribe_stream(&self) -> Pin<Box<dyn Stream<Item = Result<TranscriptionChunk>> + Send>>;
}
```

### 8. Tiered Caching Strategy (L1 + L2)
**Decision**: Build a tiered cache architecture. Use an in-memory Rust `moka` cache for ultra-low latency L1 caching of frequent utility calls. On an L1 miss, fall back to Dapr State Management (L2 cache).
**Rationale**: Achieves microsecond latency for hot data (like frequently used embeddings or prompt templates) while maintaining distributed consistency and fault tolerance via the sidecar. Emit OpenTelemetry spans for hit/miss ratios.
**Alternatives Considered**: Dapr state cache only (rejected - adds sidecar latency to hot-path data), In-memory only (rejected - lacks distributed persistence).

### 9. Battle-Tested OSS & Centralized Secret Management
**Decision**: Use **no custom code** where battle-tested open-source packages exist. For example, use `reqwest` for HTTP, `moka` for L1 caching, and `deadpool` for connection pools. Additionally, all API keys and environment parameters must be resolved through a single centralized `config` module backed by the Dapr Secret Store API.
**Rationale**: Reinforces enterprise stability. Utilities must never manually parse `.env` files or attempt to re-implement complex low-level mechanics like LRU caches or connection pools.

### 11. Streaming Support Matrix
**Decision**: Support streaming for utilities where LLM providers support it.
**Streaming Matrix**:
| Utility | Streaming Support | Strategy | Notes |
|---------|------------------|----------|-------|
| OpenAI Chat | ✅ | Direct stream | Native streaming API |
| Anthropic Claude | ✅ | Direct stream | Native streaming API |
| Local LLMs | ✅ | Direct stream | Via generation callbacks |
| TTS | ✅ | Audio chunk stream | Real-time playback |
| STT | ✅ | Real-time transcription | Voice input |
| Embeddings | ❌ | Complete only | Usually not streamed |
| Web Search | ❌ | Complete only | Atomic results |

**Rationale**: Streaming reduces time-to-first-byte (TTFB) for conversational interfaces. Not all utilities benefit from streaming.

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
8. Implement Dapr TTS binding client with streaming
9. **Implement Dapr STT binding client with streaming (NEW)**
10. Add caching layer across all utilities
11. Create compatibility test suite
12. Test streaming functionality with voice-chat cookbook

