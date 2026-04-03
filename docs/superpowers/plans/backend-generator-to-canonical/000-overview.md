# Shared Pipeline And DeerFlow Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing reusable pipeline so DeerFlow can flow from generator capture through storage-aware raw envelopes, state-server-mediated reads, lineage-safe canonical records, and L2 game-facing projections that later generators can follow.

**Architecture:** Reuse the current foundation, raw-source, normalizer, derivation, and world-projection crates instead of inventing app-local glue. Start by hardening foundation metadata for source level/plane and correlation needs, then add a DeerFlow-specific raw/mediated adapter, normalize that into L0/L1/L2 canonical records, and finish with reusable game-facing derivations plus world objects backed by lineage and backlinks.

**Tech Stack:** Rust workspace crates, `serde`, `serde_json`, `chrono`, `thiserror`, `insta`, existing DeerFlow fixture/bridge shapes as reference only

---

## Scope Boundary

- This plan covers the shared reusable spine plus DeerFlow onboarding only.
- It modifies existing crates: `foundation/contracts`, `foundation/domain`, `pipeline/raw_sources`, `pipeline/normalizers`, `pipeline/derivations`, and `runtime/world_projection`.
- It does **not** modify `apps/deer_gui` or any proof app.
- It does **not** onboard Hermes, Rowboat, or PocketFlow yet; the resulting pipeline must make those follow-on adapters straightforward.
- It does **not** implement a production state server. It introduces state-server-aligned mediated read DTOs and fixture-backed DeerFlow adapters that preserve the boundary.
- It does **not** allow direct backend payloads, direct artifact paths, or app-local mutable truth to bypass storage/lineage semantics.

## File Structure

### Foundation metadata hardening

- Modify: `crates/foundation/contracts/src/meta.rs` - extend correlation metadata with `agent_id` and source object anchors needed by generator onboarding.
- Modify: `crates/foundation/contracts/src/records.rs` - add source level/plane and storage-disposition convenience flags to `RecordHeader`.
- Modify: `crates/foundation/domain/src/common.rs` - add header builders that preserve correlations and explicit source level/plane metadata.
- Modify: `crates/foundation/domain/src/common.rs` - extend the `define_record!` macro with `new_with_meta(...)` while keeping existing `new(...)` intact.
- Create: `crates/foundation/contracts/tests/source_metadata_contracts.rs` - verify required source/plane fields and correlation metadata.
- Create: `crates/foundation/domain/tests/source_aware_record_builders.rs` - verify records can preserve correlation and source metadata without breaking existing constructors.

### DeerFlow raw source and mediated-read layer

- Create: `crates/pipeline/raw_sources/src/envelopes.rs` - raw envelope families aligned to the spec catalog.
- Create: `crates/pipeline/raw_sources/src/deerflow.rs` - DeerFlow-specific fixture/parser adapter that emits raw envelope batches.
- Create: `crates/pipeline/raw_sources/src/mediated_reads.rs` - state-server-aligned live stream, snapshot, replay window, and artifact preview DTOs.
- Modify: `crates/pipeline/raw_sources/src/lib.rs` - export the new envelope, DeerFlow, and mediated-read APIs.
- Modify: `crates/pipeline/raw_sources/Cargo.toml` - add any crate dependencies needed by the new modules.
- Create: `crates/pipeline/raw_sources/tests/deerflow_adapter.rs` - end-to-end fixture test for DeerFlow raw capture and mediated read shaping.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json` - DeerFlow thread/run snapshot fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json` - DeerFlow multi-agent live activity fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json` - DeerFlow replay/history fixture.

### Canonical normalization

- Modify: `crates/pipeline/normalizers/src/lib.rs` - export the new DeerFlow normalization entrypoint.
- Modify: `crates/pipeline/normalizers/src/carrier.rs` - normalize DeerFlow raw envelopes into L0/L1/L2 carrier records with correlations and lineage.
- Modify: `crates/pipeline/normalizers/src/representation.rs` - emit representation records for DeerFlow `as_is` and selective `chunks` payloads.
- Modify: `crates/pipeline/normalizers/src/governance.rs` - emit transform records for DeerFlow promotion steps.
- Create: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs` - verify L0/L1/L2 occupancy, lineage backlinks, and correlation preservation.

### L2 game-facing derivations

- Create: `crates/pipeline/derivations/src/game_face.rs` - derive agent units, task-force links, queue rows, history rows, and telemetry summaries from canonical records.
- Modify: `crates/pipeline/derivations/src/lib.rs` - export the new game-facing VM entrypoints.
- Create: `crates/pipeline/derivations/tests/deerflow_game_face.rs` - verify DeerFlow canonical records derive into stable L2 game-facing views with backlinks.

### World projection

- Modify: `crates/runtime/world_projection/src/world_object.rs` - add world object constructors for unit actors, task-force links, queue beacons, and history anchors with level/plane/source metadata.
- Modify: `crates/runtime/world_projection/src/projection_rules.rs` - project DeerFlow-backed canonical records into world objects without inventing app-local truth.
- Create: `crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs` - end-to-end pipeline proof from DeerFlow raw fixture to world projection.
