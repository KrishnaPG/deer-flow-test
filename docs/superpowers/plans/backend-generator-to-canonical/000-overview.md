# A/B Storage Contracts And C:L2 View Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define a generator-agnostic backend contract where Hierarchy A and Hierarchy B physically exist in storage across `L0`-`L6`, and every app/tool/game reads only from named `C:L2` SQL views or materialized views. DeerFlow is the first generator binding and first consumer proof slice.

**Architecture:** Hierarchy A stores observed execution/orchestration facts. Hierarchy B stores normalized content, operational, and lineage-bearing storage rows derived from A. Hierarchy C is the presentation hierarchy only: a catalog of storage-native SQL views/materialized views defined over `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6`. Presentation canonicalization is the authored `C:L2` SQL contract set, not app-local glue and not direct `L0/L1` consumption.

**Tech Stack:** Rust workspace crates, storage schema contracts, SQL view/materialized view definitions, `serde`, `serde_json`, `chrono`, `thiserror`, `insta`

---

## Architecture Invariants

- Hierarchy A and Hierarchy B are the only physical storage hierarchies in this slice.
- `L0` and `L1` are storage evidence and sanitation layers; consumers never query them directly.
- `C:L2` is the minimum consumer query surface for apps/tools/games.
- Every `C:L2` contract must declare source families, allowed source levels, join keys, row grain, output columns, ABAC/exclusion behavior, and refresh policy.
- DeerFlow binds into the shared A/B contract; it does not redefine the shared model.
- Shells are not separate architectures. Each shell contributes required `C:L2` view definitions within the same presentation hierarchy.

## Scope Boundary

- This plan defines the shared A/B storage contract, the shared `C:L2` query contract, and the metadata needed to support both.
- This plan includes DeerFlow as the first generator profile and the first consumer-shell registration only.
- This plan does not allow direct app reads from generator payloads or from raw `L0/L1` storage.
- This plan does not make client caches, UI state, or world objects the canonical source of truth.
- This plan does not onboard other generators yet; later generators must fit the same A/B/C contract.

## Plan Shape

1. Task 0 defines the shared A/B storage family registry plus the shared `C:L2` SQL-view contract registry.
2. Task 1 hardens record metadata so A/B rows carry immutable joins, lineage, ABAC/exclusion, and refresh data required by `C:L2` views.
3. Task 2 lands DeerFlow source objects into shared A/B storage families without leaking app-facing canon.
4. Task 3 sanitizes DeerFlow landing rows into shared `A:L2+` and `B:L2+` projection inputs.
5. Task 4 registers shared commander/researcher/thread/core-shell `C:L2` views against those A/B inputs.
6. Task 5 registers the battle-command/world-primary shell views in the same `C:L2` catalog.

## File Structure

### Shared storage and query contracts

- Create: `crates/pipeline/raw_sources/src/storage_contract.rs` - authoritative registry of shared A/B family ids, allowed levels, immutable keys, and lineage anchors.
- Create: `crates/pipeline/raw_sources/src/c_query_contract.rs` - shell registry naming which `C:L2` views each consumer requires.
- Modify: `crates/pipeline/raw_sources/src/lib.rs` - export the shared storage and query contracts.
- Create: `crates/pipeline/raw_sources/tests/storage_contract.rs` - prove generator bindings land in explicit A/B families and consumer guards reject direct `L0/L1` reads.
- Create: `crates/pipeline/normalizers/src/c_view_catalog.rs` - authoritative `C:L2` SQL view/materialized-view catalog.
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
