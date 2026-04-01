# Design: Modular Toolkit - Implementation Readiness Checklist

**Date:** 2026-04-01
**Status:** Draft revision

Current read after the modular-toolkit reconciliation and prefill-seed-source follow-up: the contract-definition tranche is substantially filled in, but implementation remains blocked because the required source docs still sit at `Status: Draft revision` and the discovery line has not yet been accepted for planning.

## Must Be True Before Any Implementation

- [x] storage truth vs client truth is explicitly documented
- [x] mediated read boundary is explicitly documented
- [x] intent/write boundary is explicitly documented
- [x] L0-L6 semantics are attached to canonical record design
- [x] planes (`as_is`, `chunks`, `embeddings`) are attached to canonical record design
- [x] immutable lineage fields are defined
- [x] generator-agnostic mapping rules exist
- [x] view taxonomy and LOD rules exist
- [x] shared base, mandatory modules, and profile-driven module model are defined
- [x] level-prefixed semantic extensions are defined as a profile-driven module
- [x] mandatory carrier/orchestration module families are defined
- [x] profile-driven representation families are defined
- [x] profile-driven governance/operational families are defined
- [x] universal vs conditional metadata envelopes are defined
- [x] mutable status is modeled append-only, not as in-place updates
- [x] proof-app-first sequencing remains in force
- [x] no reusable feature is allowed to debut in `apps/deer_gui`
- [ ] storage-native invariants are fully documented end to end:
  - [x] storage is sole immutable truth
  - [x] DB/index/cache layers are derived only
  - [ ] `UPDATE`/`DELETE` rewriting is forbidden by model
  - [ ] `as_is` / `chunks` / `embeddings` identities are hash anchored
  - [x] embeddings do not duplicate payload text as source truth

## Required Source Docs

All listed source docs exist. The remaining readiness blocker in this section is status, not file absence: the required docs are still marked `Status: Draft revision`, so this checklist remains blocked.

- `docs/superpowers/specs/modular-toolkit/11-state-server-alignment.md`
- `docs/superpowers/specs/modular-toolkit/12-levels-and-planes-contract.md`
- `docs/superpowers/specs/modular-toolkit/13-lineage-and-immutability-contract.md`
- `docs/superpowers/specs/modular-toolkit/14-data-trajectory-model.md`
- `docs/superpowers/specs/modular-toolkit/15-discovery-object-taxonomy.md`
- `docs/superpowers/specs/modular-toolkit/16-retrieval-and-indexing-contract.md`
- `docs/superpowers/specs/modular-toolkit/17-view-taxonomy-and-lod.md`
- `docs/superpowers/specs/modular-toolkit/18-generator-mapping-matrix.md`
- `docs/superpowers/specs/modular-toolkit/19-ui-state-server-boundary.md`
- `docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md`
- `docs/superpowers/specs/modular-toolkit/22-canonical-record-families.md`
- `docs/superpowers/specs/modular-toolkit/23-correlation-and-identity-contract.md`
- `docs/superpowers/specs/modular-toolkit/24-level-plane-lineage-matrix.md`
- `docs/superpowers/specs/modular-toolkit/25-generator-capability-matrix.md`
- `docs/superpowers/specs/modular-toolkit/26-normalizer-promotion-rules.md`
- `docs/superpowers/specs/modular-toolkit/27-raw-envelope-family-catalog.md`
- `docs/superpowers/specs/modular-toolkit/28-entity-view-matrix.md`
- `docs/superpowers/specs/modular-toolkit/29-world-projection-object-contract.md`
- `docs/superpowers/specs/modular-toolkit/30-library-decision-matrix.md`
- `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md`
- `docs/superpowers/specs/modular-toolkit/35-carrier-core-and-typed-extensions-contract.md`
- `docs/superpowers/specs/modular-toolkit/36-view-contract-support-model.md`

## No-Go Conditions

Implementation is blocked if any of the following are true:

- any required source doc is missing (currently false)
- any required source doc is still `Status: Draft revision` (currently true)
- crate-level implementation plans are written before discovery approval
- crate folders, Cargo changes, module skeletons, fixtures, tests, or proof-app code are started early
- a team tries to justify "just foundation work" before discovery approval
- a reusable feature is proposed to debut in `apps/deer_gui`

## Failure Rule

Scaffolding counts as implementation.

If any checklist item is incomplete, do not start foundation, runtime, view,
proof-app, or `deer_gui` implementation work.
