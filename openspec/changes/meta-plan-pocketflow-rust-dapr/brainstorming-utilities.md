# Brainstorming: Utilities Dapr Mapping

## Current State (PocketFlow-Rust)
- **LLM calls**: Basic wrapper (`llm_wrapper.rs`)
- **Chunking**: Basic text chunking (`text_chunking.rs`)
- **Embedding**: Basic embedding (`embedding.rs`)
- **Vector DB**: Basic vector DB client (`vector_db.rs`)
- **Web Search**: Basic web search (`web_search.rs`)
- **Visualization**: Basic debug visualization (`viz_debug.rs`)
- **Text-to-Speech**: Not implemented

## Dapr Building Blocks for Utilities

### 1. LLM Calls → Dapr Conversation API
**Dapr Mapping**: Use Dapr Conversation API for LLM calls (OpenAI, Anthropic, etc.)
**Pros**: Centralized LLM management, caching, rate limiting, observability
**Cons**: Additional Dapr component dependency
**Alternative**: Direct HTTP calls to LLM providers (simpler but less features)

### 2. Chunking → Pure Rust Implementation
**Dapr Mapping**: No Dapr mapping needed; implement as pure Rust functions
**Rationale**: Chunking is a local operation, no need for distributed runtime
**Implementation**: Use existing Rust crates (`text-splitter`, `tokenizers`)

### 3. Embedding → Dapr Conversation API + Local Models
**Dapr Mapping**: Use Dapr Conversation API for cloud embeddings, local Rust crates for local models
**Pros**: Flexible, supports both cloud and local embeddings
**Cons**: Need to handle multiple backends
**Implementation**: Trait-based abstraction with Dapr and local implementations

### 4. Vector DB → Dapr Vector Bindings
**Dapr Mapping**: Use Dapr Vector bindings for vector database operations
**Pros**: Pluggable backends (Pinecone, Weaviate, Milvus, etc.)
**Cons**: Binding limitations, may need custom implementations for advanced features
**Alternative**: Direct vector DB clients (more control but more code)

### 5. Web Search → Dapr Bindings
**Dapr Mapping**: Use Dapr bindings for web search (Google, Bing, etc.)
**Pros**: Centralized search API, caching, rate limiting
**Cons**: Limited search providers via bindings
**Alternative**: Direct HTTP calls to search APIs (more flexibility)

### 6. Visualization → Pure Rust + Dapr State for Caching
**Dapr Mapping**: Generate visualizations locally, cache results in Dapr State
**Rationale**: Visualization is local rendering; caching can be distributed
**Implementation**: Use `plotters`, `mermaid` crates for generation; Dapr State for caching

### 7. Text-to-Speech → Dapr Bindings
**Dapr Mapping**: Use Dapr bindings for TTS services (Google TTS, Amazon Polly, etc.)
**Pros**: Pluggable TTS providers, caching, rate limiting
**Cons**: May need custom implementations for advanced features
**Alternative**: Direct API calls (simpler but less features)

## Architecture Sketch

```
┌─────────────────────────────────────────────────────────┐
│                   Utilities Layer                       │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   LLM       │  │  Embedding  │  │  Vector DB  │    │
│  │  (Dapr      │  │  (Dapr +    │  │  (Dapr      │    │
│  │  Conv API)  │  │  Local)     │  │  Bindings)  │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│         │                 │                 │          │
│         ▼                 ▼                 ▼          │
│  ┌─────────────────────────────────────────────────────┐│
│  │              Dapr Sidecar (daprd)                  ││
│  └─────────────────────────────────────────────────────┘│
│         │                 │                 │          │
│         ▼                 ▼                 ▼          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  LLM        │  │  Embedding  │  │  Vector     │    │
│  │  Providers  │  │  Models     │  │  Databases  │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## Open Questions

1. **Dapr Conversation API maturity**: Is Dapr Conversation API production-ready for LLM calls?
2. **Local vs cloud**: Should we prioritize local implementations for offline use?
3. **Caching strategy**: How to cache utility results (LLM responses, embeddings) effectively?
4. **Error handling**: How to handle Dapr component failures gracefully?
5. **Performance**: What are the latency implications of Dapr sidecar for utility calls?

## Next Steps

1. Create OpenSpec change for utilities porting plan
2. Define detailed specs for each utility mapping
3. Design Rust API with trait abstractions
4. Plan implementation phases