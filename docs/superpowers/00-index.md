# Superpowers Docs Index

**Date:** 2026-04-01
**Purpose:** Fast entrypoint for future agents working from `docs/superpowers/`

## Start Here

Read these first, in order:

1. `docs/superpowers/01-current-state.md`
2. `docs/superpowers/02-rules-in-force.md`
3. `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`
4. `docs/superpowers/plans/2026-04-01-functional-game-gap-plan-index.md`

Then read the active workstream docs listed there.

## Progress Tracking

Use the existing tracker files as the canonical progress state for resumed work.

- `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md` is the main readiness-gate checklist
- `docs/superpowers/plans/2026-04-01-functional-game-gap-plan-index.md` is the ordered next-steps tracker for the functional-game path
- future agents should read both before resuming work
- when work starts, finishes, or is re-scoped, update the relevant tracker/checklist files instead of relying only on session-local todo state
- the current tracker state already reflects active main-tree implementation for the foundation/pipeline/runtime spine; use those files for exact commit references
- do not create or depend on a separate root status-board doc when those tracker files already cover the current state

- Recently written/planned implementation docs (present in `docs/superpowers/plans/`):
  - `2026-04-01-foundation-spine-contracts-domain-replay.md`
  - `2026-04-01-normalization-derivation-read-model-spine.md`
  - `2026-04-01-live-chat-toolkit-proof.md`
  - `2026-04-01-layout-runtime-toolkit-proof.md`
  - `2026-04-01-spatial-projection-toolkit-proof.md`
  - `2026-04-01-first-playable-deer-gui-composition.md`

## What Lives Here

- `docs/superpowers/specs/` - design decisions and intended architecture
- `docs/superpowers/plans/` - execution plans, not the source of truth for rules
- `docs/superpowers/research/` - inputs and exploration, never normative by themselves

## Current Workstreams

| Workstream | Role | Canonical Entry |
| --- | --- | --- |
| `modular-toolkit` | current architecture direction | `docs/superpowers/specs/modular-toolkit/00-index.md` |
| `hybrid-data-game` | product/game framing | `docs/superpowers/specs/2026-03-31-hybrid-data-game-design.md` |
| `scifi-rts-hud` | HUD visual direction | `docs/superpowers/specs/scifi-rts-hud/2026-03-30-scifi-rts-hud-design.md` |
| `multi-scene-themes` | approved `deer_gui` scene/theme design | `docs/superpowers/specs/multi-scene-themes/00-index.md` |

## Navigation Rules

- If a root doc and a deep doc disagree, follow the root doc's authority map.
- If a spec and a plan disagree, follow the spec unless the root status docs say otherwise.
- Treat the readiness checklist and gap-plan index as the main TODO/progress source for resumed work.
- Do not start with `research/` or `plans/` unless the current-state doc points you there.
- Treat superseded docs as historical context only.
