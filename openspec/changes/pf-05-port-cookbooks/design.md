## Context

PocketFlow-Rust lacks cookbook examples, while Python PocketFlow has 59 examples covering various patterns. This design creates a porting plan that prioritizes examples, designs a test harness for validation, and establishes a framework for maintaining compatibility. The cookbooks build on the core abstractions, design patterns, and utilities ported in previous changes.

We are taking a **fresh start approach**: use Python PocketFlow examples as inspiration ("in spirit" and "logic"), not as strict line-by-line templates. The Rust implementation should do everything Python examples can do, but better - with Dapr integration, faster execution, and zero-copy where possible. Time to working state is more important than achieving 100% line equality.

## Goals / Non-Goals

**Goals:**
- Port all 59 PocketFlow cookbooks to Rust with Dapr integration
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

### 2. Test Harness: Python-Rust Comparator
**Decision**: Build test harness that runs both Python and Rust implementations and compares outputs.
**Rationale**: Ensures functional equivalence, catches regressions.
**Alternatives Considered**: Manual testing only (rejected - error-prone), unit tests only (rejected - misses integration).

### 3. Dapr Integration: Mock for Testing
**Decision**: Use Dapr CLI in standalone mode for testing, mock components for unit tests.
**Rationale**: Enables testing without full Dapr deployment, maintains test speed.
**Alternatives Considered**: Real Dapr deployment for all tests (rejected - slow, complex), no Dapr in tests (rejected - misses integration issues).

### 4. Example Structure: One Crate per Category
**Decision**: Organize examples in crate by category (basic, patterns, integrations, advanced).
**Rationale**: Logical grouping, easier navigation, matches Python structure.
**Alternatives Considered**: Single flat directory (rejected - messy), one crate per example (rejected - too many crates).

### 5. Documentation: Side-by-Side Comparison
**Decision**: Provide documentation showing Python and Rust code side-by-side.
**Rationale**: Helps developers migrate, highlights differences.
**Alternatives Considered**: Separate documentation (rejected - harder to compare), inline comments only (rejected - insufficient).

## Risks / Trade-offs

**Risk**: Some Python features may not have direct Rust equivalents → Mitigation: Document differences, provide alternatives.
**Risk**: Test harness may be complex to maintain → Mitigation: Keep harness simple, use existing testing frameworks.
**Risk**: Dapr mocking may not catch all integration issues → Mitigation: Include integration tests with real Dapr.
**Risk**: Porting all 59 examples is time-consuming → Mitigation: Prioritize, allow community contributions.

## Migration Plan

1. Create test harness framework
2. Port Tier 1 examples (hello-world, flow, batch, chat)
3. Validate with test harness
4. Port Tier 2 examples (agent, RAG, coding-agent)
5. Port Tier 3 examples (fastapi, integrations)
6. Port Tier 4 examples (advanced patterns)
7. Create documentation and migration guides
8. Establish maintenance process for updates

## Open Questions

1. How to handle Python-specific dependencies (e.g., specific libraries)?
2. What level of documentation is needed for each example?
3. How to measure success of porting (coverage, performance, etc.)?
4. How to involve community in porting efforts?
5. How to keep examples updated as Python versions evolve?