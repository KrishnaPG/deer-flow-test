## Context

PocketFlow-Rust requires systematic validation to ensure it matches Python PocketFlow's behavior, meets performance expectations, integrates correctly with Dapr, and satisfies enterprise security and compliance requirements. This design creates a validation framework that automates these checks across multiple dimensions.

We are taking a **fresh start approach**: validate functional equivalence and performance improvement, not line-by-line equality. Python PocketFlow is the functional reference, but the Rust implementation is allowed (encouraged) to differ internally where it achieves better outcomes.

## Goals / Non-Goals

**Goals:**
- Validate functional equivalence between Python and Rust implementations
- Benchmark performance improvements (Rust+Dapr should be faster than Python)
- Test integration with Dapr components and external dependencies
- Validate security features (authentication, authorization, encryption)
- Ensure compliance with enterprise standards (SOC2, GDPR)
- Provide automated CI/CD integration and reporting
- Focus on catching regressions, not on achieving identical internal implementation

**Non-Goals:**
- Implement the actual validation framework (this is a plan only)
- Validate every possible edge case (focus on critical paths)
- Replace manual validation entirely (some manual checks needed)
- Require line-by-line code equivalence (not needed, functional equivalence is sufficient)

## Decisions

### 1. Framework Architecture: Multi-Category Testing
**Decision**: Organize validation into categories (unit, integration, performance, security, compliance) with shared infrastructure.
**Rationale**: Separation of concerns, allows specialized tools per category.
**Alternatives Considered**: Single monolithic test suite (rejected - complex), separate frameworks per category (rejected - duplication).

### 2. Feature Parity & Stability: Tolerances + Property-Based Fuzzing
**Decision**: Compare Python and Rust outputs with **explicit ToleranceConfig** for non-deterministic operations. Integrate `proptest` for **Property-Based Fuzzing**.
**ToleranceConfig Structure**:
```rust
pub struct ToleranceConfig {
    /// Semantic similarity threshold for text (0.0 - 1.0)
    /// Use case: LLM output comparison
    pub text_similarity: f64,  // Default: 0.85
    
    /// Acceptable timing variance percentage
    /// Use case: Performance comparison
    pub timing_variance_percent: f64,  // Default: 20.0
    
    /// Floating-point comparison tolerance
    /// Use case: Numeric embedding comparison
    pub numeric_precision: f64,  // Default: 0.001
    
    /// Statistical confidence for property-based tests
    /// Use case: Fuzzing success rate
    pub statistical_confidence: f64,  // Default: 0.99
    
    /// BLEU score threshold for generated text
    /// Use case: Code generation comparison
    pub bleu_threshold: f64,  // Default: 0.7
}
```
**Comparison Strategies**:
| Output Type | Strategy | Threshold |
|-------------|----------|-----------|
| Free text | Semantic similarity | > 0.85 |
| JSON | Structural equality | 100% |
| JSON (numeric) | ± tolerance | ±0.001 |
| Embeddings | Cosine similarity | > 0.95 |
| Code | BLEU score | > 0.7 |
**Rationale**: Explicit tolerances prevent flaky tests. Fuzzing proves crash-resilience.
**Alternatives Considered**: Fixed input tests only (rejected - misses obscure edge cases), manual validation only (rejected - not scalable).

### 3. Performance Benchmarking: Criterion + Custom Metrics
**Decision**: Use Rust's `criterion` for micro-benchmarks, custom metrics for workflow-level performance.
**Rationale**: `criterion` provides statistical rigor; custom metrics capture Dapr-specific overhead.
**Alternatives Considered**: Ad-hoc timing (rejected - unreliable), external benchmarking tools (rejected - complex).

### 4. Integration Testing: Automated Chaos Testing & OSS
**Decision**: Use `testcontainers-rs` (battle-tested OSS) for real Dapr components, and build a "Chaos Mock" Dapr component that automatically injects latency, HTTP 500s, and sidecar crashes during integration tests.
**Rationale**: This mathematically proves that the resiliency policies work under extreme stress. Happy-path testing is insufficient for an enterprise-grade system. `testcontainers-rs` avoids writing custom orchestration bash scripts for tests.
**Alternatives Considered**: Real Dapr without fault injection (rejected - doesn't prove resilience mechanisms work), Manual chaos testing (rejected - unscalable).

### 5. Security Testing: OWASP ZAP + Custom Checks
**Decision**: Use OWASP ZAP for web security, custom checks for Dapr security features.
**Rationale**: Industry-standard tool for web security; custom checks for Dapr-specific security.
**Alternatives Considered**: Manual security review only (rejected - error-prone), no security testing (rejected - enterprise requirement).

### 6. Compliance Testing: Policy as Code (OPA)
**Decision**: Use Open Policy Agent (OPA) for compliance policy checks.
**Rationale**: Declarative policies, reusable across environments, audit trail.
**Alternatives Considered**: Manual compliance checks (rejected - not scalable), custom compliance engine (rejected - reinventing wheel).

### 9. Checkpoint and Recovery Testing
**Decision**: Explicitly test checkpoint creation, persistence, and recovery scenarios.
**Test Cases**:
```rust
#[tokio::test]
async fn test_checkpoint_recovery() {
    // 1. Start workflow
    let workflow = create_long_running_workflow();
    
    // 2. Run for N steps
    let partial = run_until_checkpoint(workflow.clone(), 5).await;
    
    // 3. Simulate crash (drop workflow)
    drop(workflow);
    
    // 4. Recover from checkpoint
    let recovered = WorkflowInstance::recover(partial.checkpoint).await;
    
    // 5. Complete workflow
    let result = recovered.run().await;
    
    // 6. Verify matches non-interrupted execution
    assert_outputs_equal(result, uninterrupted_result);
}
```
**Recovery Scenarios**:
- Crash during node execution
- Crash between nodes
- Dapr sidecar restart
- Network partition during checkpoint save
**Rationale**: Dapr durability guarantees must be validated.

### 10. State Management Validation
**Decision**: Test SharedStore trait implementations (Dapr State, Local/SQLite, InMemory).
**Test Coverage**:
- State consistency after node failures
- ETag optimistic concurrency handling
- Transactional updates (all-or-nothing)
- State migration between workflow versions
- Large state handling (>1MB)
**Test Matrix**:
| Backend | Persistence | Concurrency | Recovery |
|---------|-------------|-------------|----------|
| Dapr State | ✅ | ✅ | ✅ |
| SQLite | ✅ | ✅ | ✅ |
| InMemory | ❌ | ✅ | ❌ |
**Rationale**: State management is critical for durability guarantees.

### 7. CI/CD Integration: GitHub Actions + Custom Reporters
**Decision**: Integrate with GitHub Actions, generate validation reports in multiple formats.
**Rationale**: Popular CI/CD platform, flexible reporting for different stakeholders.
**Alternatives Considered**: Custom CI/CD (rejected - complex), no automation (rejected - not scalable).

### 8. CI/CD Integration: GitHub Actions & Ownership
**Decision**: Integrate with GitHub Actions for zero-cost continuous validation. Micro-benchmarks and fuzzing must run on every PR. Heavy integration and chaos tests run Nightly. The core framework maintainers own the harness infrastructure, while cookbook-specific tests are community-maintained.
**Rationale**: Popular CI/CD platform, flexible reporting, negligible infrastructure cost. Separating PR checks from Nightly checks preserves developer velocity.

### 11. Action-Based Routing Validation
**Decision**: Exhaustively test action-based routing (core PocketFlow mechanic).
**Test Coverage**:
- Default action routing
- Custom action routing  
- Fallback after retry exhaustion
- Loop termination (back-edges)
- Missing action handling (flow end)
**Rationale**: Action-based routing is PocketFlow's defining feature; must be fully validated.

**Risk**: Validation framework complexity → Mitigation: Start with critical paths, expand incrementally.
**Risk**: Test flakiness due to timing/non-determinism → Mitigation: Use explicit ToleranceConfig, retries, deterministic mocks.
**Risk**: Performance overhead of validation → Mitigation: Run validation in parallel, optimize test execution.
**Risk**: Maintenance burden of test suite → Mitigation: Use reusable components, clear documentation.
**Risk**: Checkpoint testing adds complexity → Mitigation: Start with simple crash scenarios, expand to complex failures.

## Migration Plan

1. Create validation framework core infrastructure
2. Define ToleranceConfig with explicit values
3. Implement feature parity validation for core abstractions
4. Add performance benchmarking for critical paths
5. Implement checkpoint/recovery testing scenarios
6. Add state management validation (Dapr, SQLite, InMemory)
7. Implement action-based routing validation
8. Implement integration testing with Dapr components
9. Add security testing for Dapr features
10. Implement compliance policy checks
11. Integrate with CI/CD pipeline
12. Create validation dashboards and reports

