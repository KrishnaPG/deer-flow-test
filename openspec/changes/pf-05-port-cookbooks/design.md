## Context

PocketFlow-Rust lacks cookbook examples, while Python PocketFlow has 56 examples covering various patterns. This design creates a porting plan that prioritizes examples, designs a test harness for validation, and establishes a framework for maintaining compatibility. The cookbooks build on the core abstractions, design patterns, and utilities ported in previous changes.

We are taking a **fresh start approach**: use Python PocketFlow examples as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust implementation should do everything Python examples can do, but better - with Dapr integration, faster execution, and zero-copy where possible. Time to working state is more important than achieving 100% line equality.

## Goals / Non-Goals

**Goals:**
- Port all 56 PocketFlow cookbooks to Rust with Dapr integration
- Ensure functional equivalence between Python and Rust implementations
- Design comprehensive test harness for automated validation
- Create integration testing framework with Dapr components
- Provide clear documentation and migration guides
- Prioritize working implementation over line-by-line Python equivalence
- Achieve performance improvements over Python examples where possible

**Non-Goals:**
- Implement the actual porting (this is a plan only)
- Port every Python feature exactly (some may be simplified for Rust)
- Create new cookbooks not in Python version
- Support all Python dependencies in Rust
- Achieve line-by-line code equivalence (not required)

## Decisions

### 1. Prioritization: Tiered Approach
**Decision**: Port cookbooks in four tiers based on complexity and importance.
**Rationale**: Focuses effort on core examples first, provides incremental value.
**Alternatives Considered**: Alphabetical order (rejected - no prioritization), random selection (rejected - inefficient).

### 2. Test Harness: Multi-Mode Validation
**Decision**: Build test harness that validates:
1. **Functional Equivalence**: Run Python and Rust, compare outputs
2. **Checkpoint/Recovery**: Simulate crashes, verify resumption
3. **Streaming**: Test real-time output for streaming cookbooks
4. **Durability Modes**: Test with InMemory, Local (SQLite), and Dapr backends
**Rationale**: Comprehensive validation of custom orchestrator, not just Dapr integration.
**Test Strategy**:
```rust
#[tokio::test]
async fn test_chat_with_checkpoint_recovery() {
    // Run for 3 turns, simulate crash
    let partial = run_workflow(chat_flow, 3, InMemoryDurability).await;
    
    // Recover and resume
    let recovered = recover_from_checkpoint(partial.checkpoint).await;
    
    // Complete workflow
    let result = continue_workflow(recovered).await;
    
    // Verify against Python reference
    assert_outputs_equal(result, python_reference_output);
}
```

### 3. Execution Integration: Multiple Durability Modes
**Decision**: Support three durability modes for cookbooks:
- **Dev Mode**: `InMemoryDurability` - no persistence, fastest iteration
- **Local Mode**: `LocalDurability` (SQLite) - single-user production, no Dapr sidecar
- **Distributed Mode**: `DaprDurability` - full enterprise features with sidecar
**Rationale**: Matches architecture from PF-02. Enables testing at all levels without forcing Dapr for simple examples.
**Test Strategy**:
- Unit tests: Use InMemoryDurability
- Integration tests: Use LocalDurability (SQLite) or Dapr in standalone mode
- Production tests: Use full Dapr deployment

### 4. Example Structure: One Crate per Category
**Decision**: Organize examples in crate by category (basic, patterns, integrations, advanced). For Python-specific dependencies used in original examples (e.g., `requests`, `BeautifulSoup`), replace them with battle-tested Rust equivalents (e.g., `reqwest`, `scraper`). Where no Rust port exists, wrap the logic in a Dapr Binding.
**Rationale**: Logical grouping, easier navigation, matches Python structure. Enforcing Rust-native equivalents ensures pure performance without dragging in external Python runtimes unless absolutely necessary via sidecar bindings.
**Alternatives Considered**: Single flat directory (rejected - messy), one crate per example (rejected - too many crates).

### 5. Documentation and Community Involvement
**Decision**: Provide documentation showing Python and Rust code side-by-side. Additionally, designate specific cookbooks as "Good First Issue" leveraging the new `berg10-execution-engine` generic traits to encourage community contribution.
**Rationale**: Side-by-side docs help developers migrate and highlight differences. Open-sourcing porting efforts accelerates completion while stress-testing the generic APIs with external developers.
**Alternatives Considered**: Separate documentation (rejected - harder to compare), inline comments only (rejected - insufficient).

### 6. Streaming Cookbook Support
**Decision**: Include streaming test cases for cookbooks that use streaming (voice-chat, llm-streaming).
**Rationale**: Streaming is first-class feature. Test harness must validate streaming behavior.
**Test Approach**:
- Measure time-to-first-byte (TTFB)
- Verify chunk ordering
- Test backpressure handling
- Compare against Python streaming behavior

## Risks / Trade-offs

**Risk**: Some Python features may not have direct Rust equivalents → Mitigation: Document differences, provide alternatives.
**Risk**: Test harness may be complex to maintain → Mitigation: Keep harness simple, use existing testing frameworks.
**Risk**: Dapr mocking may not catch all integration issues → Mitigation: Include integration tests with real Dapr.
**Risk**: Porting all 56 examples is time-consuming → Mitigation: Prioritize, allow community contributions.
**Risk**: Checkpoint testing adds complexity → Mitigation: Start with simple cases, add recovery scenarios incrementally.
**Risk**: Streaming test validation may be flaky → Mitigation: Use tolerance-based comparison, not exact matching.

## Migration Plan

1. Create test harness framework with Python-Rust comparator
2. Create checkpoint/recovery test utilities
3. Port Tier 1 examples (hello-world, flow, batch, chat)
4. Validate functional equivalence with test harness
5. Port Tier 2 examples (agent, RAG, coding-agent)
6. Test checkpoint/recovery for long-running examples
7. Port Tier 3 examples (fastapi, integrations, voice-chat)
8. Test streaming functionality (voice-chat, llm-streaming)
9. Port Tier 4 examples (advanced patterns)
10. Create documentation and migration guides
11. Establish maintenance process for updates

