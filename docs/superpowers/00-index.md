# Superpowers Docs Index

**Date:** 2026-03-31
**Purpose:** Fast entrypoint for future agents working from `docs/superpowers/`

## Start Here

Read these first, in order:

1. `docs/superpowers/01-current-state.md`
2. `docs/superpowers/02-rules-in-force.md`
3. `docs/superpowers/04-status-board.md`

Then read the active workstream docs listed there.

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
- Do not start with `research/` or `plans/` unless the current-state doc points you there.
- Treat superseded docs as historical context only.
