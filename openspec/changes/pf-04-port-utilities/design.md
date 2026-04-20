## Context

PocketFlow-Rust has basic utility implementations but lacks Dapr integration and enterprise features. This design maps each utility to Dapr building blocks to achieve cached, rate-limited, and observable operations. The utilities build on the core abstractions and design patterns ported in previous changes.

We are taking a **fresh start approach**: use Python PocketFlow utilities as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust utilities should do everything Python can do, but better - with caching, rate limiting, and observability built-in.

## Goals / Non-Goals

**Goals:**
- Map LLM calls to Dapr Conversation API for centralized LLM management
- Map embedding to Dapr Conversation API + local Rust implementations
- Map vector DB operations to Dapr Vector bindings
- Map web search to Dapr bindings
- Implement chunking as pure Rust (no Dapr mapping needed)
- Implement visualization as pure Rust with Dapr State caching
- Map text-to-speech to Dapr bindings
- Add caching, rate limiting, and observability to all utilities
- Prioritize working implementation over line-by-line Python equivalence
- Aim for zero-copy where possible for performance

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Replace all Python utility functionality wholesale
- Support every possible LLM provider or vector database
- Provide real-time streaming for all utilities
- Achieve line-by-line code equivalence with Python (not required)

## Decisions

### 1. LLM Calls: Dapr Conversation API
**Decision**: Use Dapr Conversation API for LLM calls with fallback to direct HTTP.
**Rationale**: Centralized LLM management, caching, rate limiting, observability.
**Alternatives Considered**: Direct HTTP calls only (rejected - lacks enterprise features), custom LLM client (rejected - reinventing wheel).

### 2. Embedding: Hybrid Approach
**Decision**: Use Dapr Conversation API for cloud embeddings, local Rust crates for local models.
**Rationale**: Flexibility for offline use and cloud deployment.
**Alternatives Considered**: Cloud-only (rejected - no offline support), local-only (rejected - limited model variety).

### 3. Vector DB: Dapr Bindings
**Decision**: Use Dapr Vector bindings for vector database operations.
**Rationale**: Pluggable backends, consistent API across providers.
**Alternatives Considered**: Direct vector DB clients (rejected - more code), custom abstraction (rejected - reinventing wheel).

### 4. Web Search: Dapr Bindings
**Decision**: Use Dapr bindings for web search services.
**Rationale**: Centralized search API, caching, rate limiting.
**Alternatives Considered**: Direct API calls (rejected - lacks enterprise features), custom search client (rejected - reinventing wheel).

### 5. Chunking: Pure Rust
**Decision**: Implement chunking as pure Rust functions using existing crates.
**Rationale**: Local operation, no need for distributed runtime.
**Alternatives Considered**: Dapr mapping (rejected - unnecessary complexity), custom implementation (rejected - reinventing wheel).

### 6. Visualization: Pure Rust + Dapr State Caching
**Decision**: Generate visualizations locally, cache results in Dapr State.
**Rationale**: Visualization is local rendering; caching can be distributed.
**Alternatives Considered**: Dapr rendering (rejected - overkill), no caching (rejected - performance).

### 7. Text-to-Speech: Dapr Bindings
**Decision**: Use Dapr bindings for TTS services.
**Rationale**: Pluggable TTS providers, caching, rate limiting.
**Alternatives Considered**: Direct API calls (rejected - lacks enterprise features), custom TTS client (rejected - reinventing wheel).

## Risks / Trade-offs

**Risk**: Dapr Conversation API may not support all LLM providers → Mitigation: Fallback to direct HTTP calls.
**Risk**: Dapr bindings may have limited feature sets → Mitigation: Allow custom implementations for advanced features.
**Risk**: Caching may introduce staleness → Mitigation: Configurable TTL and cache invalidation.
**Risk**: Local embedding models may be large → Mitigation: Provide model downloading and caching.

## Migration Plan

1. Create utility trait definitions
2. Implement Dapr Conversation API LLM client
3. Implement hybrid embedding client
4. Implement Dapr Vector binding client
5. Implement Dapr web search binding client
6. Implement chunking utilities
7. Implement visualization with Dapr State caching
8. Implement Dapr TTS binding client
9. Add caching layer across all utilities
10. Create compatibility test suite

## Open Questions

1. How to handle Dapr component configuration for different environments?
2. What caching strategy to use (LRU, TTL, etc.)?
3. How to handle authentication for Dapr components?
4. Should we support custom utility implementations alongside Dapr?
5. How to test Dapr integration without running actual Dapr sidecar?