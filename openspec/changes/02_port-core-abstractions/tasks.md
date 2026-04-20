## 1. Setup and Crate Structure

- [ ] 1.1 Create crate `crates/berg10/execution-engine` with Cargo.toml
- [ ] 1.2 Add dependencies: dapr, tokio, serde, serde_json, async-trait, thiserror
- [ ] 1.3 Define module structure: node, flow, context, params, retry, error, dapr/
- [ ] 1.4 Create AGENTS.md with coding standards

## 2. Core Trait Definitions

- [ ] 2.1 Define Node trait with prep, exec, post methods
- [ ] 2.2 Define Flow trait with orchestration methods
- [ ] 2.3 Define SharedStore trait with key-value operations
- [ ] 2.4 Define Params type as HashMap<String, serde_json::Value>
- [ ] 2.5 Define error types (NodeError, FlowError, StoreError)

## 3. Dapr Activity Node Implementation

- [ ] 3.1 Implement DaprActivityNode wrapper for Node trait
- [ ] 3.2 Map prep→exec→post to Dapr Activity lifecycle
- [ ] 3.3 Integrate Dapr retry policies with node retry logic
- [ ] 3.4 Implement exec_fallback as Dapr compensation handler
- [ ] 3.5 Add serialization/deserialization for activity inputs/outputs

## 4. Dapr Actor Node Implementation (Optional)

- [ ] 4.1 Implement DaprActorNode wrapper for stateful nodes
- [ ] 4.2 Define actor state persistence via Dapr State Management
- [ ] 4.3 Implement actor turn-based concurrency
- [ ] 4.4 Map node identity to actor ID

## 5. Dapr Workflow Flow Implementation

- [ ] 5.1 Implement DaprWorkflowFlow wrapper for Flow trait
- [ ] 5.2 Map flow orchestration to Dapr Workflow definition
- [ ] 5.3 Implement sequential execution via workflow activities
- [ ] 5.4 Implement conditional transitions via workflow branching
- [ ] 5.5 Implement loops via workflow continue signals
- [ ] 5.6 Implement nested flows via sub-workflows

## 6. Dapr State Management SharedStore

- [ ] 6.1 Implement DaprStateStore wrapping Dapr State Management API
- [ ] 6.2 Implement key-value operations with pluggable backends
- [ ] 6.3 Implement transactional state updates
- [ ] 6.4 Implement state recovery after failures
- [ ] 6.5 Add caching layer for performance

## 7. Retry and Fallback Features

- [ ] 7.1 Implement retry policies with exponential backoff
- [ ] 7.2 Integrate Dapr retry policies with node retries
- [ ] 7.3 Implement fallback execution as compensation
- [ ] 7.4 Add fallback-to-alternative-path support in flows

## 8. Parallel Execution

- [ ] 8.1 Implement fan-out/fan-in using Dapr Workflow when_all
- [ ] 8.2 Implement batch processing with parallelism limits
- [ ] 8.3 Add AsyncParallelBatchNode equivalent
- [ ] 8.4 Add AsyncParallelBatchFlow equivalent

## 9. Compatibility Layer

- [ ] 9.1 Create Python PocketFlow compatibility API
- [ ] 9.2 Implement behavioral compatibility tests
- [ ] 9.3 Document differences due to Rust type system
- [ ] 9.4 Provide migration guide from Python PocketFlow

## 10. Testing and Validation

- [ ] 10.1 Create unit tests for each trait implementation
- [ ] 10.2 Create integration tests with Dapr sidecar (mock)
- [ ] 10.3 Create compatibility test suite vs Python PocketFlow
- [ ] 10.4 Create performance benchmarks
- [ ] 10.5 Add observability hooks (tracing, metrics)