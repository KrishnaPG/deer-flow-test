## 1. Setup and Utility Traits

- [ ] 1.1 Define utility traits (LLM, Embedding, VectorDB, etc.)
- [ ] 1.2 Create utility configuration types
- [ ] 1.3 Define utility error types
- [ ] 1.4 Add utility observability hooks

## 2. LLM Utility Implementation

- [ ] 2.1 Implement Dapr Conversation API LLM client
- [ ] 2.2 Implement direct HTTP fallback client
- [ ] 2.3 Add response caching with Dapr State
- [ ] 2.4 Add rate limiting with Dapr components
- [ ] 2.5 Implement streaming support

## 3. Chunking Utility Implementation

- [ ] 3.1 Implement text chunking strategies
- [ ] 3.2 Add token-aware chunking using tokenizer crates
- [ ] 3.3 Implement chunking statistics
- [ ] 3.4 Add compatibility layer with Python chunking

## 4. Embedding Utility Implementation

- [ ] 4.1 Implement Dapr Conversation API embedding client
- [ ] 4.2 Implement local Rust embedding models
- [ ] 4.3 Add embedding caching with Dapr State
- [ ] 4.4 Implement batch embedding support
- [ ] 4.5 Add model downloading and caching

## 5. Vector DB Utility Implementation

- [ ] 5.1 Implement Dapr Vector binding client
- [ ] 5.2 Add vector CRUD operations
- [ ] 5.3 Implement similarity search with multiple metrics
- [ ] 5.4 Add search result caching
- [ ] 5.5 Implement batch vector operations

## 6. Web Search Utility Implementation

- [ ] 6.1 Implement Dapr web search binding client
- [ ] 6.2 Add search result parsing and formatting
- [ ] 6.3 Implement search result caching
- [ ] 6.4 Add rate limiting for search providers
- [ ] 6.5 Implement provider-specific optimizations

## 7. Visualization Utility Implementation

- [ ] 7.1 Implement Mermaid diagram generation
- [ ] 7.2 Add diagram rendering to images (SVG, PNG)
- [ ] 7.3 Implement diagram caching with Dapr State
- [ ] 7.4 Add interactive visualization support
- [ ] 7.5 Implement layout and styling options

## 8. Text-to-Speech Utility Implementation

- [ ] 8.1 Implement Dapr TTS binding client
- [ ] 8.2 Add audio formatting and playback
- [ ] 8.3 Implement audio caching with Dapr State
- [ ] 8.4 Add voice and quality options
- [ ] 8.5 Implement provider-specific optimizations

## 9. Caching and Rate Limiting Framework

- [ ] 9.1 Implement distributed caching layer with Dapr State
- [ ] 9.2 Add configurable TTL per utility
- [ ] 9.3 Implement cache invalidation mechanisms
- [ ] 9.4 Add rate limiting framework with Dapr components
- [ ] 9.5 Implement rate limit monitoring and metrics

## 10. Compatibility and Testing

- [ ] 10.1 Create Python PocketFlow compatibility API
- [ ] 10.2 Create output compatibility tests
- [ ] 10.3 Create utility-specific test suites
- [ ] 10.4 Create performance benchmarks
- [ ] 10.5 Add integration tests with Dapr sidecar (mock)