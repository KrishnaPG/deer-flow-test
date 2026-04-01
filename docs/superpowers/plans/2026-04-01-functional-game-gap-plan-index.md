# Functional Game Gap Plan Index

**Date:** 2026-04-01
**Status:** Blocked pending discovery acceptance and source-doc status lift
**Purpose:** Identify the minimum implementation-plan documents that must exist, in order, before a functional game becomes possible under the modular-toolkit guardrails.

## Prerequisite Gate

The modular-toolkit reconciliation and the prefill-seed-source follow-up are now reflected in the spec line, but the gate below is still not cleared.

No implementation plan below is valid to execute until the discovery tranche through `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` is approved and the readiness checklist is no longer blocked by `Status: Draft revision` source docs.

This file indexes what must be planned next once that gate is cleared. None of the indexed implementation-plan documents exist yet.

## Required Next Plan Documents

| Order | Plan doc | Current status | Scope | Why it exists | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | `docs/superpowers/plans/2026-04-01-foundation-spine-contracts-domain-replay.md` | not written; blocked by prerequisite gate | `foundation/contracts`, `foundation/domain`, and `foundation/replay` for canonical records, identity, lineage, and replay-safe base types | First playable cannot exist until every later slice shares one canonical, testable source of truth | discovery approval |
| 2 | `docs/superpowers/plans/2026-04-01-normalization-derivation-read-model-spine.md` | not written; blocked by prerequisite gate | `pipeline/normalizers`, `pipeline/derivations`, and `runtime/read_models` for raw input -> canonical domain -> typed view-model flow | The game loop needs live orchestration state, artifact state, and macro-state projections outside any app shell | 1 |
| 3 | `docs/superpowers/plans/2026-04-01-live-chat-toolkit-proof.md` | not written; blocked by prerequisite gate | M1 reusable conversation thread, prompt composer, artifact shelf/file presenter, and live stream/task-progress slice proven in `apps/deer_chat_lab` | This is the first proof that request submit, live progress, clarification pause/resume, and artifact access work from reusable modules | 2 |
| 4 | `docs/superpowers/plans/2026-04-01-layout-runtime-toolkit-proof.md` | not written; blocked by prerequisite gate | M4 panel shells, panel registry, layout runtime, and view-host bridge proven in `apps/deer_design` | `deer_gui` cannot become the first place where chat, inspector, and supporting views are composed into one runtime | 3 |
| 5 | `docs/superpowers/plans/2026-04-01-spatial-projection-toolkit-proof.md` | not written; blocked by prerequisite gate | M5 actor cloud, telemetry/map views, scene-host integration, and world-projection rules proven in `apps/deer_design` or a narrow spatial sandbox | First playable needs world selection and macro-state reaction tied back to canonical chat/task/artifact state | 2, 4 |
| 6 | `docs/superpowers/plans/2026-04-01-first-playable-deer-gui-composition.md` | not written; blocked by prerequisite gate | Thin M6 composition in `apps/deer_gui` using only previously proven slices for: submit one request, watch progress, inspect one artifact/result, select one world object, trigger one intervention, inspect resulting history | This is the first point where a functional game becomes possible, because the closed loop is finally assembled from proven toolkit bricks | 3, 4, 5 |

## Deliberate Scope Boundary

- The blocker audit narrows the minimum path to M0/M1/M4/M5 before thin M6 composition, so these are the next plan docs to write first.
- Today's actual progress is still pre-implementation: the spec line is reconciled enough to make this sequence concrete, but not accepted enough to start writing or executing the indexed plan docs.
- Standalone M2 replay-toolkit and M3 graph-toolkit plans stay out of this immediate gap-closure index unless reviewers insist on the full literal M0-M5 ladder before M6, or the narrowed playable loop expands to require those proof apps separately.
- Existing `docs/superpowers/plans/modular-toolkit/07-shell-state-linked-interaction-foundation.md`, `docs/superpowers/plans/modular-toolkit/08-battle-command-shell.md`, and `docs/superpowers/plans/modular-toolkit/09-runtime-semantics.md` are downstream of this gap set, not substitutes for it.
