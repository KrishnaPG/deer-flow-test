# A/B Storage Contracts And C:L2 View Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define a generator-agnostic backend contract where Hierarchy A and Hierarchy B physically exist in storage across `L0`-`L6`, `C:L0` admits shell-declared presentation inputs from `A:L2+` and `B:L2+`, `C:L1` optionally transforms them, and every app/tool/game reads only from named shell-defined `C:L2` contracts implemented by generator-supplied SQL views or materialized views. DeerFlow is the first generator binding and first consumer proof slice.

**Architecture:** Hierarchy A stores observed execution/orchestration facts. Hierarchy B stores normalized content, operational, and lineage-bearing storage rows derived from A. Hierarchy C is the presentation hierarchy: `C:L0` accepts all `A:L2+` and `B:L2+` rows that a shell declares as admissible presentation inputs, `C:L1` is optional presentation-side transform/reduction/formatting, and `C:L2` is the shell-defined canonical query contract layer implemented per generator through SQL/materialized views or equivalent query projections. Presentation canonicalization remains the authored `C:L2` contract set, not app-local glue and not direct `A/B:L0/L1` consumption.

**Tech Stack:** Rust workspace crates, storage schema contracts, SQL view/materialized view definitions, `serde`, `serde_json`, `chrono`, `thiserror`, `insta`

---

## Architecture Invariants

- Hierarchy A and Hierarchy B are the only physical storage hierarchies in this slice.
- `A/B:L0` and `A/B:L1` are storage evidence and sanitation layers; consumers never query them directly.
- `C:L0` accepts all `A:L2+` and `B:L2+` rows that a shell declares as admissible presentation inputs.
- `C:L1` is optional presentation-side transform/reduction/formatting.
- `C:L2` is the shell-defined minimum consumer query surface for apps/tools/games, implemented by generators.
- Every `C:L2` contract must declare source families, allowed source levels, join keys, row grain, output columns, ABAC/exclusion behavior, and refresh policy.
- DeerFlow binds into the shared A/B contract; it does not redefine the shared model.
- Shells are not separate architectures. Each shell contributes required `C:L2` contract definitions within the same presentation hierarchy, and generators supply the implementations that satisfy them.

## Scope Boundary

- This plan defines the shared A/B storage contract, the `C:L0` admissibility boundary, optional `C:L1` presentation transforms, the shared shell-defined `C:L2` query contract, and the metadata needed to support generator-supplied implementations of those contracts.
- This plan includes DeerFlow as the first generator profile and the first consumer-shell registration only.
- This plan does not allow direct app reads from generator payloads or from raw `A/B:L0/L1` storage.
- This plan does not make client caches, UI state, or world objects the canonical source of truth.
- This plan does not onboard other generators yet; later generators must fit the same A/B/C contract.

## Plan Shape

1. Task 0 defines the shared A/B storage family registry plus the `C:L0` admissibility rules, optional `C:L1` policy, and shared shell-defined `C:L2` contract registry plus generator-implemented query catalog requirements.
2. Task 1 hardens record metadata so A/B rows carry immutable joins, lineage, ABAC/exclusion, and refresh data required by `C:L2` views.
3. Task 2 lands DeerFlow source objects into shared A/B storage families without leaking app-facing canon.
4. Task 3 sanitizes DeerFlow landing rows into shared `A:L2+` and `B:L2+` projection inputs.
5. Task 4 registers shared commander/researcher/thread/core-shell `C:L2` views against those A/B inputs.
6. Task 5 registers the battle-command/world-primary shell views in the same `C:L2` catalog.

## File Structure

### Shared storage and query contracts

- Create: `crates/pipeline/raw_sources/src/storage_contract.rs` - authoritative registry of shared A/B family ids, allowed levels, immutable keys, and lineage anchors.
- Create: `crates/pipeline/raw_sources/src/c_query_contract.rs` - shell registry naming admissible `C:L0` inputs and which shell-defined `C:L2` contracts each consumer requires.
- Modify: `crates/pipeline/raw_sources/src/lib.rs` - export the shared storage and query contracts.
- Create: `crates/pipeline/raw_sources/tests/storage_contract.rs` - prove generator bindings land in explicit A/B families, shell admissibility covers `A:L2+`/`B:L2+`, and consumer guards reject direct `A/B:L0/L1` reads.
- Create: `crates/pipeline/normalizers/src/c_view_catalog.rs` - authoritative generator-supplied `C:L1`/`C:L2` presentation transform and SQL view/materialized-view implementation catalog for shell-defined contracts.
- Modify: `crates/pipeline/normalizers/src/lib.rs` - export the `C:L2` view catalog.
- Create: `crates/pipeline/normalizers/tests/c_view_catalog.rs` - prove every registered `C:L2` view reads only from allowed A/B levels and has a complete contract.

### Foundation metadata

- Modify: `crates/foundation/contracts/src/meta.rs` - add immutable join keys, correlation anchors, ABAC/exclusion metadata, and view-refresh metadata.
- Modify: `crates/foundation/contracts/src/records.rs` - carry the new metadata on stored A/B rows.
- Modify: `crates/foundation/domain/src/common.rs` - preserve that metadata through builders.
- Create: `crates/foundation/contracts/tests/source_metadata_contracts.rs` - verify headers expose the metadata required by `C:L2` SQL definitions.
- Create: `crates/foundation/domain/tests/source_aware_record_builders.rs` - verify builders preserve join, lineage, access, and refresh metadata.

### DeerFlow first binding

- Create: `crates/pipeline/raw_sources/src/deerflow.rs` - bind DeerFlow fixture shapes into shared A/B family ids.
- Create: `crates/pipeline/raw_sources/tests/deerflow_adapter.rs` - verify DeerFlow lands in the shared storage contract without DeerFlow-only canon.
