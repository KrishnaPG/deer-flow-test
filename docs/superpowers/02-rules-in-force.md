# Superpowers Rules In Force

**Date:** 2026-03-31
**Purpose:** Short normative checklist for future planning and implementation

## Global Rules

- Save this line of work under `docs/superpowers/` only.
- Prefer small focused docs over monoliths.
- Research is input, not authority.
- Plans are execution guides, not the source of truth for architecture.

## Architecture Rules In Force

Follow `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`.

The most important rules are:

- toolkit first; `deer_gui` is a client, not the center
- no layer jumping
- reusable modules stay generic
- tool apps prove slices before `deer_gui`
- TDD happens at module boundaries
- chat is domain state, not a widget
- world meaning enters only through `world_projection`
- stable cross-view identity must be preserved
- backend truth and client transient state must stay separate

## Planning Rules In Force

Follow `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`.

Every plan item must answer:

1. owning layer or crate
2. contract being defined
3. failing test or fixture first
4. proof app before `deer_gui`
5. forbidden dependencies
6. milestone gate unlocked

## Current Do-Not-Start-With List

Do not start from these unless explicitly needed:

- `docs/superpowers/plans/`
- `docs/superpowers/research/`
- superseded spec files

Start from `docs/superpowers/01-current-state.md` instead.
