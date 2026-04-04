# DeerFlow Generator Onboarding — Shell + Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the full onboarding path for DeerFlow as the first backend generator under the modular toolkit's New Generator Onboarding Contract (spec 42), covering shared A/B storage families, normalization to L2+, shell-defined C:L2 query contracts, and end-to-end proof.

**Architecture:** Hierarchy A (observed execution facts) and Hierarchy B (normalized content/lineage) are physical storage. Hierarchy C is presentation: C:L0 admits shell-declared A:L2+/B:L2+ rows, C:L1 is optional transform, C:L2 is shell-defined canonical query contracts implemented by generator-supplied SQL/query projections. DeerFlow binds into shared A/B families, normalizes to L2+, and supplies C:L2 implementations for commander, researcher, thread/core, battle-command, and world-primary shells.

**Tech Stack:** Rust workspace crates, serde, serde_json, chrono, thiserror, uuid, insta

---

## Architecture Invariants (from spec 42)

- A and B are the only physical storage hierarchies
- L0/L1 are storage evidence only; consumers never query them directly
- C:L0 = admissible A:L2+/B:L2+ source pool for presentation composition
- C:L1 = optional presentation-side format/selection/reduction
- C:L2 = minimum consumer query surface; shell defines contract, generator supplies implementation
- B:L3/L4/L5/L6 rows are admissible B:L2+ inputs, not a special exception
- A generator does not define app-facing canon
- Shell does not bypass C:L2 with app-local read glue

## File Structure

### New files to create

| File | Responsibility |
|------|---------------|
| `crates/pipeline/raw_sources/src/storage_contract.rs` | Shared A/B family registry, allowed levels, immutable keys, lineage anchors, C:L0 admissibility |
| `crates/pipeline/raw_sources/src/c_query_contract.rs` | Shell registry: admissible C:L0 inputs per shell, required C:L2 view ids per shell |
| `crates/pipeline/raw_sources/src/deerflow.rs` | DeerFlow binding: maps DeerFlow source shapes into shared A/B family ids |
| `crates/pipeline/normalizers/src/c_view_catalog.rs` | C:L1/C:L2 presentation transform and SQL view implementation catalog |
| `crates/pipeline/raw_sources/tests/storage_contract.rs` | Tests: A/B registry, consumer guards, DeerFlow bindings, B:L3+ admissibility |
| `crates/pipeline/raw_sources/tests/deerflow_adapter.rs` | Tests: DeerFlow landing into shared A/B families, no consumer canon leak |
| `crates/pipeline/normalizers/tests/c_l2_projection_catalog.rs` | Tests: commander/researcher/thread/core C:L2 views registered with complete metadata |
| `crates/pipeline/normalizers/tests/c_l2_world_shell_catalog.rs` | Tests: battle-command/world-primary C:L2 views registered |
| `crates/foundation/contracts/tests/source_metadata_contracts.rs` | Tests: headers expose join/lineage/ABAC/refresh metadata |
| `crates/foundation/domain/tests/source_aware_record_builders.rs` | Tests: builders preserve metadata end-to-end |
| `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs` | Tests: normalization produces L2+ rows with lineage intact |

### Modified files

| File | Change |
|------|--------|
| `crates/pipeline/raw_sources/src/lib.rs` | Export new modules |
| `crates/pipeline/raw_sources/Cargo.toml` | Add `deer-foundation-contracts` dependency |
| `crates/pipeline/normalizers/src/lib.rs` | Export c_view_catalog |
| `crates/foundation/contracts/src/meta.rs` | Add JoinKeysMeta, AccessControlMeta, ViewRefreshMeta |
| `crates/foundation/contracts/src/records.rs` | Add new metadata fields to RecordHeader |
| `crates/foundation/contracts/src/ids.rs` | Add new typed ID types if needed |
| `crates/foundation/domain/src/common.rs` | Source-aware builders with full metadata |

## Task Index

| Task | Name | Crate |
|------|------|-------|
| 0 | Shared A/B Storage Contract and C:L2 Query Contract Registry | `raw_sources` |
| 1 | Harden Foundation Metadata | `foundation/contracts`, `foundation/domain` |
| 2 | DeerFlow Raw Landing into Shared A/B Families | `raw_sources` |
| 3 | DeerFlow Normalization to Shared A/B L2+ Rows | `normalizers` |
| 4 | C:L2 Presentation Views — Commander/Researcher/Thread/Core | `normalizers` |
| 5 | C:L2 Presentation Views — Battle-Command/World-Primary | `normalizers` |
| 6 | End-to-End Proof and Readiness Gate | `raw_sources`, `normalizers` |

## Execution Notes

### Test Run Commands (per task)

```bash
# Task 0
cargo test -p deer-pipeline-raw-sources --test storage_contract -v

# Task 1
cargo test -p deer-foundation-contracts --test source_metadata_contracts -v
cargo test -p deer-foundation-domain --test source_aware_record_builders -v

# Task 2
cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v

# Task 3
cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v

# Task 4
cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v

# Task 5
cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v
cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v

# Task 6
cargo test -p deer-pipeline-raw-sources --test e2e_deerflow_onboarding -v
cargo test -p deer-pipeline-normalizers --test e2e_deerflow_c_l2 -v

# Full workspace
cargo test --workspace -v
cargo clippy --workspace
```

### Dependency Changes Summary

| Crate | Change |
|-------|--------|
| `deer-pipeline-raw-sources` | Add `deer-foundation-contracts` dependency |
| `deer-foundation-contracts` | No new deps |
| `deer-foundation-domain` | No new deps |
| `deer-pipeline-normalizers` | No new deps (already depends on raw_sources) |

### Key Design Decisions

1. **StorageContract** uses string-keyed HashMap for family lookup, enabling runtime validation that no generator-specific family ids leak
2. **C:L0 admissibility** is a pure function of level (L2+ = admissible, L0/L1 = not), with B:L3+ using the same rule as B:L2
3. **CQueryContract** and **CViewCatalog** are separate: contracts declare what shells need, catalog holds generator-supplied SQL implementations
4. **DeerFlow binding** is fixture-driven: JSON fixtures bind to shared families through explicit mapping, not inference
5. **Normalization** elevates L0/L1 to L2+ for the shared path, preserving lineage through the `header_mut()` trait method
6. **SQL definitions** are stored as strings in the catalog — actual SQL execution would be handled by a storage engine layer outside this scope
