# Design: Modular Toolkit - Implementation Readiness Checklist

**Date:** 2026-03-31
**Status:** Draft revision

## Must Be True Before Any Implementation

- [ ] storage truth vs client truth is explicitly documented
- [ ] mediated read boundary is explicitly documented
- [ ] intent/write boundary is explicitly documented
- [ ] L0-L6 semantics are attached to canonical record design
- [ ] planes (`as_is`, `chunks`, `embeddings`) are attached to canonical record design
- [ ] immutable lineage fields are defined
- [ ] generator-agnostic mapping rules exist
- [ ] view taxonomy and LOD rules exist
- [ ] shared base, mandatory modules, and profile-driven module model are defined
- [ ] level-prefixed semantic extensions are defined as a profile-driven module
- [ ] mandatory carrier/orchestration module families are defined
- [ ] profile-driven representation families are defined
- [ ] profile-driven governance/operational families are defined
- [ ] universal vs conditional metadata envelopes are defined
- [ ] mutable status is modeled append-only, not as in-place updates
- [ ] proof-app-first sequencing remains in force
- [ ] no reusable feature is allowed to debut in `apps/deer_gui`
- [ ] storage-native invariants are documented:
  - storage is sole immutable truth
  - DB/index/cache layers are derived only
  - `UPDATE`/`DELETE` rewriting is forbidden by model
  - `as_is` / `chunks` / `embeddings` identities are hash anchored
  - embeddings do not duplicate payload text as source truth

## Required Source Docs

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

- any required source doc is missing
- any required source doc is still `Status: Draft revision`
- crate-level implementation plans are written before discovery approval
- crate folders, Cargo changes, module skeletons, fixtures, tests, or proof-app code are started early
- a team tries to justify "just foundation work" before discovery approval
- a reusable feature is proposed to debut in `apps/deer_gui`

## Failure Rule

Scaffolding counts as implementation.

If any checklist item is incomplete, do not start foundation, runtime, view,
proof-app, or `deer_gui` implementation work.
