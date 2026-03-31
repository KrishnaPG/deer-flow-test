# Design: Modular Toolkit - Index

**Date:** 2026-03-31
**Status:** Draft revision
**Scope:** `apps/deer_gui` and supporting toolkit/apps

## Overview

This spec set replaces the earlier monolithic modular-toolkit design with a
smaller, toolkit-first set of documents.

Core position:

- the primary product is a reusable toolkit, not a single app
- authoring and inspection tools are first-class proving grounds
- TDD happens at module boundaries before app composition
- `deer_gui` becomes the first playable composition target, not the place where
  core modules are invented

## File Map

| File | Purpose |
| --- | --- |
| `00-index.md` | entry point, scope, file map |
| `01-architecture-position.md` | architectural stance and core principles |
| `02-workspace-and-layering.md` | workspace shape and strict data layering |
| `03-reusable-modules.md` | lego-brick modules, contracts, and boundaries |
| `04-authoring-tools-as-proving-grounds.md` | tool apps that prove each module slice |
| `05-tdd-by-contract.md` | test strategy and module-first TDD workflow |
| `06-milestones-to-first-playable.md` | milestone ladder from toolkit to playable app |
| `07-chat-and-artifact-pipeline.md` | live chat, streaming, uploads, and artifact-access mapping |
| `08-chat-to-world-bridging.md` | how generic chat state projects into RTS/RPG world state |
| `09-non-negotiables.md` | top architectural principles that must survive planning and implementation |
| `10-planning-guardrails.md` | mandatory planning and milestone guardrails to prevent drift |

## Stable Decisions

- Save this line of work under `docs/superpowers/` only.
- Prefer multiple small files over one large design file.
- Follow strict layering:
  `raw -> canonical domain -> derived VM -> reusable view/panel -> app`.
- Reusable modules must stay app-agnostic.
- Tool apps must consume the same reusable modules as runtime apps.
- `deer_gui` is one composition target, not the monolithic center.

## Reading Order

Read in this order:

1. `01-architecture-position.md`
2. `02-workspace-and-layering.md`
3. `03-reusable-modules.md`
4. `04-authoring-tools-as-proving-grounds.md`
5. `05-tdd-by-contract.md`
6. `06-milestones-to-first-playable.md`
7. `07-chat-and-artifact-pipeline.md`
8. `08-chat-to-world-bridging.md`
9. `09-non-negotiables.md`
10. `10-planning-guardrails.md`

## Superseded Document

The old single-file version is superseded by this directory:

- `docs/superpowers/specs/2026-03-31-modular-toolkit-architecture-design.md`
