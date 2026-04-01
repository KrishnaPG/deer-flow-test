# Design: Modular Toolkit - Planning Guardrails

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The architecture is strong enough to guide planning, but not strong enough to
prevent drift unless the planning phase carries explicit gates.

This file defines those gates.

## Questions Every Plan Item Must Answer

Every planned task must state:

1. what layer or crate owns this work?
2. what contract is being defined or extended?
3. what failing test or fixture comes first?
4. which proof app validates it before `deer_gui` uses it?
5. what inputs, events, and commands does it expose?
6. what dependencies are forbidden for this task?
7. if this is world mapping, why does it belong in `world_projection`?
8. what milestone gate does completion unlock?

If a plan item cannot answer these, it is not ready.

## Reusable Module Definition Of Done

A reusable module is not done until it has:

- named owning crate/layer
- typed contracts
- representative fixtures
- passing automated tests
- a proof-app path
- declared forbidden dependencies
- explicit input, output, command, persistence, and runtime assumptions

## Proof-App Rule

"Proven in a tool app" means:

- the slice is wired into its narrow proof app
- at least one scripted, fixture-backed scenario exists
- the scenario is repeatable, not a one-off manual demo
- the proof happens before `deer_gui` depends on the slice

## Milestone Gates

### Gate A - Foundation Ready

- raw inputs flow to canonical domain and typed view models
- transient client state is testable outside apps

### Gate B - Module Ready

- no raw-schema or app-specific type leakage
- contract and tests exist
- reusable boundary is explicit

### Gate C - Tool Proof Complete

- the slice works end-to-end in its proof app
- proof scenario is fixture-backed

### Gate D - Projection Ready

- projected world objects are generated from canonical state only
- stable IDs map world objects back to source records
- drill-down from world object to generic detail works

### Gate E - Composition Allowed

`deer_gui` may compose a slice only after Gates B, C, and D are satisfied where
applicable.

## No-Skip Rule

Milestones in `06-milestones-to-first-playable.md` are blocking gates, not loose
suggestions.

Specifically:

- `deer_gui` must not become the first complete version of chat, replay, graph,
  layout runtime, or artifact browsing
- reusable slices must clear their proof-app milestone before being treated as
  available building blocks

## Review Checklist For Plans And PRs

Reject or re-scope work if any answer is yes:

- is this feature debuting in `deer_gui` even though it should be reusable?
- does this API expose raw backend or transport-specific types?
- is normalization happening outside `normalizers`?
- is app-specific metaphor leaking into reusable crates?
- is `world_projection` absorbing renderer or UI logic?
- is the proof app missing or purely manual?
- are stable cross-view IDs absent?

## Planning Consequence

When `writing-plans` begins, plans should be organized by reusable slices and
proof milestones, not by one giant app-first feature list.

## Discovery Prerequisite Rule

Do not write or execute crate-level implementation plans until the reconciled
discovery-first spec line through
`docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md` is
accepted.

This block includes:

- crate creation
- Cargo/workspace edits
- module skeletons
- foundation/runtime/view implementation
- proof-app implementation
- `deer_gui` composition work

Acceptance means:

- the discovery-first tranche through
  `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` remains
  approved in substance
- the shell, linked-interaction, temporal, policy, and intent-boundary
  reconciliation added through
  `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`,
  `docs/superpowers/specs/modular-toolkit/38-generator-shell-mode-support-evaluation.md`,
  `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`,
  `docs/superpowers/specs/modular-toolkit/40-shell-and-linked-interaction-stress-test.md`,
  and `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
  is also accepted as planning-ready
