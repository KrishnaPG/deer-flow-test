# Functional Game Gap Plan Index

**Date:** 2026-04-01
**Status:** Plan set written; implementation tracking active
**Purpose:** Identify the minimum implementation-plan documents that must exist, in order, before a functional game becomes possible under the modular-toolkit guardrails, and track which indexed slices have now landed in the main tree.

## Prerequisite Gate

The modular-toolkit reconciliation and the prefill-seed-source follow-up are now reflected in the planning-ready accepted line below, and the prerequisite gate is cleared.

The discovery tranche through `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` plus the reconciliation tranche `37`-`41` through `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md` are accepted as planning-ready under `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`.

This file now indexes the required implementation-plan documents and tracks which indexed slices already exist only as plans versus which slices have active implementation in the main tree.

## Required Next Plan Documents

| Order | Plan doc | Current status | Scope | Why it exists | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | `docs/superpowers/plans/2026-04-01-foundation-spine-contracts-domain-replay.md` | partially implemented in main: `foundation/domain` in `cdb8b79`, `foundation/replay` in `28ce6c6` | `foundation/contracts`, `foundation/domain`, and `foundation/replay` for canonical records, identity, lineage, and replay-safe base types | First playable cannot exist until every later slice shares one canonical, testable source of truth | planning acceptance recorded |
| 2 | `docs/superpowers/plans/2026-04-01-normalization-derivation-read-model-spine.md` | partially implemented in main: `pipeline/normalizers` Task 2 in `8cb343d`, `pipeline/derivations` in `1cb49ee`, `runtime/read_models` in `e3b0906` | `pipeline/normalizers`, `pipeline/derivations`, and `runtime/read_models` for raw input -> canonical domain -> typed view-model flow | The game loop needs live orchestration state, artifact state, and macro-state projections outside any app shell | 1 |
| 3 | `docs/superpowers/plans/2026-04-01-live-chat-toolkit-proof.md` | implemented in main | M1 reusable conversation thread, prompt composer, artifact shelf/file presenter, and live stream/task-progress slice proven in `apps/deer_chat_lab` | This is the first proof that request submit, live progress, clarification pause/resume, and artifact access work from reusable modules | 2 |
| 4 | `docs/superpowers/plans/2026-04-01-layout-runtime-toolkit-proof.md` | implemented in main | M4 panel shells, panel registry, layout runtime, and view-host bridge proven in `apps/deer_design` | `deer_gui` cannot become the first place where chat, inspector, and supporting views are composed into one runtime | 3 |
| 5 | `docs/superpowers/plans/2026-04-01-spatial-projection-toolkit-proof.md` | implemented in main | M5 actor cloud, telemetry/map views, scene-host integration, and world-projection rules proven in `apps/deer_design` or a narrow spatial sandbox | First playable needs world selection and macro-state reaction tied back to canonical chat/task/artifact state | 2, 4 |
| 6 | `docs/superpowers/plans/2026-04-01-first-playable-deer-gui-composition.md` | implemented in main | Thin M6 composition in `apps/deer_gui` using only previously proven slices for: submit one request, watch progress, inspect one artifact/result, select one world object, trigger one intervention, inspect resulting history | This is the first point where a functional game becomes possible, because the closed loop is finally assembled from proven toolkit bricks | 3, 4, 5 |

## Deliberate Scope Boundary

- The blocker audit narrows the minimum path to M0/M1/M4/M5 before thin M6 composition, so these are the next plan docs to write first.
- Today's actual progress: the indexed plans are all written and their structural implementations (M1 through M6) have landed in the main tree as architecture stubs and verified tests.
- Standalone M2 replay-toolkit and M3 graph-toolkit plans stay out of this immediate gap-closure index unless reviewers insist on the full literal M0-M5 ladder before M6, or the narrowed playable loop expands to require those proof apps separately.
- Existing `docs/superpowers/plans/modular-toolkit/07-shell-state-linked-interaction-foundation.md`, `docs/superpowers/plans/modular-toolkit/08-battle-command-shell.md`, and `docs/superpowers/plans/modular-toolkit/09-runtime-semantics.md` are downstream of this gap set, not substitutes for it.
