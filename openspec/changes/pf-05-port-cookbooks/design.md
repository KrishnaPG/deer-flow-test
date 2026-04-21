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

### 2. Test Harness: Python-Rust Comparator
**Decision**: Build test harness that runs both Python and Rust implementations and compares outputs.
**Rationale**: Ensures functional equivalence, catches regressions.
**Alternatives Considered**: Manual testing only (rejected - error-prone), unit tests only (rejected - misses integration).

### 3. Execution Integration: "No-Dapr" Dev Mode
**Decision**: Provide a zero-dependency "Dev Mode" trait implementation where an `in-memory-driver` thread-local runtime completely takes over the execution engine traits. For integration tests, use Dapr CLI in standalone mode or mocks.
**Rationale**: An `in-memory-driver` guarantees sub-second compile-to-run iteration cycles for developer experience (DX) without requiring container startup, while mocks maintain unit test speed.
**Alternatives Considered**: Real Dapr deployment for all local dev/tests (rejected - slow, complex, high developer friction).

### 4. Example Structure: One Crate per Category
**Decision**: Organize examples in crate by category (basic, patterns, integrations, advanced). For Python-specific dependencies used in original examples (e.g., `requests`, `BeautifulSoup`), replace them with battle-tested Rust equivalents (e.g., `reqwest`, `scraper`). Where no Rust port exists, wrap the logic in a Dapr Binding.
**Rationale**: Logical grouping, easier navigation, matches Python structure. Enforcing Rust-native equivalents ensures pure performance without dragging in external Python runtimes unless absolutely necessary via sidecar bindings.
**Alternatives Considered**: Single flat directory (rejected - messy), one crate per example (rejected - too many crates).

### 5. Documentation and Community Involvement
**Decision**: Provide documentation showing Python and Rust code side-by-side. Additionally, designate specific cookbooks as "Good First Issue" leveraging the new `berg10-execution-engine` generic traits to encourage community contribution.
**Rationale**: Side-by-side docs help developers migrate and highlight differences. Open-sourcing porting efforts accelerates completion while stress-testing the generic APIs with external developers.
**Alternatives Considered**: Separate documentation (rejected - harder to compare), inline comments only (rejected - insufficient).

## Risks / Trade-offs

**Risk**: Some Python features may not have direct Rust equivalents → Mitigation: Document differences, provide alternatives.
**Risk**: Test harness may be complex to maintain → Mitigation: Keep harness simple, use existing testing frameworks.
**Risk**: Dapr mocking may not catch all integration issues → Mitigation: Include integration tests with real Dapr.
**Risk**: Porting all 56 examples is time-consuming → Mitigation: Prioritize, allow community contributions.

## Migration Plan

1. Create test harness framework
2. Port Tier 1 examples (hello-world, flow, batch, chat)
3. Validate with test harness
4. Port Tier 2 examples (agent, RAG, coding-agent)
5. Port Tier 3 examples (fastapi, integrations)
6. Port Tier 4 examples (advanced patterns)
7. Create documentation and migration guides
8. Establish maintenance process for updates

