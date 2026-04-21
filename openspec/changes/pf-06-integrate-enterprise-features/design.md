## Context

PocketFlow-Rust currently lacks enterprise features needed for production deployment. This design integrates Dapr's enterprise building blocks to add observability, durability, scalability, fail-safety, idempotency, tracability, and security. The integration builds on the core abstractions, design patterns, utilities, and cookbooks ported in previous changes.

We are taking a **fresh start approach**: build enterprise-grade features that Python PocketFlow cannot easily provide. Rust+Dapr enables features that would require external libraries in Python - retry/fallback built-in, native OpenTelemetry, Dapr-sidecar resilience patterns.

## Goals / Non-Goals

**Goals:**
- Integrate Dapr OpenTelemetry for distributed tracing and metrics
- Integrate Dapr State Management for durable workflow state
- Leverage Dapr sidecar model for horizontal scaling and service discovery
- Implement Dapr Resiliency policies for fail-safety (retry, circuit breaker, timeout)
- Ensure idempotency through Dapr Workflow guarantees and deduplication
- Add distributed tracing and audit logging for tracability
- Integrate Dapr Secret Management and access control for security
- Provide enterprise features that Python requires external libraries for
- Aim for zero-copy, zero-allocation where possible for performance

**Non-Goals:**
- Implement the actual integration (this is a plan only)
- Replace all existing PocketFlow-Rust code wholesale
- Support non-Dapr deployment scenarios (focus on Dapr-first)
- Provide GUI management interfaces
- Duplicate Python's limitations (we build better, not just different)

## Decisions

### 1. Observability: Adaptive Telemetry Sampling via OpenTelemetry
**Decision**: Use the battle-tested `tracing` crate in the core engine to emit standard spans. The specific execution driver (e.g., `dapr-driver`) is responsible for W3C TraceContext injection. Enforce **Adaptive Head-Based Sampling** (low normal rate, 100% on error/fallback).
**Rationale**: Leverages automatic trace propagation without coupling the core engine to OpenTelemetry SDKs, while preventing observability overhead from destroying performance at enterprise scale.
**Alternatives Considered**: 100% tracing (rejected - kills performance), Custom tracing macros (rejected - reinventing wheel).

### 2. Durability: Dapr State Management + Workflows
**Decision**: Store all persistent state in Dapr State Management, use Dapr Workflows for durable execution.
**Rationale**: Provides pluggable backends, transactional updates, and automatic recovery.
**Alternatives Considered**: Custom persistence layer (rejected - reinventing wheel), file-based persistence (rejected - not scalable).

### 3. Scalability: Dapr Sidecar + Kubernetes
**Decision**: Deploy PocketFlow workflows as Dapr-enabled services on Kubernetes.
**Rationale**: Leverages Dapr's service discovery, load balancing, and Kubernetes orchestration.
**Alternatives Considered**: Custom scaling logic (rejected - complex), single-node deployment (rejected - not scalable).

### 4. Fail-safety: Technical vs. Semantic Resilience
**Decision**: Define Dapr resiliency policies (retry, circuit breaker, timeout) exclusively for *technical* failures (e.g., sidecar networking issues, temporary 500s). For *semantic* resilience (e.g., fallback logic, switching LLM providers), handle routing via PocketFlow's dynamic Node Action returns (`Action::Fallback`).
**Rationale**: Separates infrastructure flakiness from business logic fallbacks. Relying on Dapr Compensation handlers for business logic alternatives is an anti-pattern. Dapr handles the network layer; Rust handles the application logic layer.
**Alternatives Considered**: Conflating technical and semantic fallbacks (rejected - makes code hard to reason about and misuses Saga compensations), Custom retry logic (rejected - scattered).

### 5. Idempotency: Deterministic Keys & Deduplication
**Decision**: Leverage Dapr Workflow's idempotency, but the Rust core must automatically generate a deterministic UUID (hashed from `Workflow_ID + Node_ID + Attempt_Count`) and inject it into external headers as an Idempotency-Key.
**Rationale**: Dapr replays activities if they fail. To protect external systems (e.g., charging APIs, database inserts) from double-execution, a cryptographic idempotency key is mathematically required.
**Alternatives Considered**: Relying solely on Dapr Activity at-least-once guarantees without keys (rejected - causes dangerous side effects).

### 6. Tracability: Distributed Tracing + Audit Logs
**Decision**: Use Dapr's distributed tracing with custom audit logs for compliance.
**Rationale**: Automatic trace propagation, custom spans for PocketFlow operations, audit logs for lineage.
**Alternatives Considered**: Custom tracing (rejected - complex), no tracability (rejected - compliance risk).

### 7. Security: Dapr Security Features & Centralized Config
**Decision**: Use Dapr secret stores for credentials, ACLs for access control, mTLS for encryption. Enforce that *all* secret retrieval routes exclusively through a centralized `config` module, preventing random `env::var` usage across nodes.
**Rationale**: Leverages Dapr's security features without custom implementation, while ensuring the execution engine remains completely agnostic of how secrets are fetched.
**Alternatives Considered**: Custom security (rejected - reinventing wheel), Direct `.env` parsing (rejected - security risk).

## Risks / Trade-offs

**Risk**: Dapr sidecar adds latency → Mitigation: Optimize Dapr configuration, use local mode for development.
**Risk**: Dapr configuration complexity → Mitigation: Provide templates and automation.
**Risk**: Kubernetes operational overhead → Mitigation: Use managed Kubernetes services.
**Risk**: Vendor lock-in to Dapr → Mitigation: Abstract Dapr interfaces, allow alternative implementations.

## Migration Plan

1. Create Dapr component configurations for each building block
2. Integrate OpenTelemetry instrumentation in Rust code
3. Migrate state storage to Dapr State Management
4. Define and apply Dapr resiliency policies
5. Implement idempotency and deduplication
6. Set up distributed tracing and audit logging
7. Configure Dapr security features
8. Create deployment templates for Kubernetes
9. Establish monitoring and alerting dashboards

