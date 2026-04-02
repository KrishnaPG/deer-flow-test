# Design: Modular Toolkit - Architecture Position

**Date:** 2026-03-31
**Status:** Accepted for planning

## Product Position

The primary product is not a single game app.

The primary product is a reusable toolkit for:

- data ingestion and normalization
- canonical simulation and read models
- reusable 2D and 3D views
- reusable panel and layout composition
- authoring and inspection tools

`apps/deer_gui` is then one composed client of that toolkit.

## Priority Order

Priority order stays:

1. reusable view toolkit
2. foundational data pipeline
3. authoring tools
4. game/app composition
5. vertical slice proving ground

This is agile, but toolkit-first agile.

## Core Principles

### 1. Strict Layering

Every feature must pass through:

`raw backend data -> canonical domain model -> derived view model -> reusable view/panel -> app composition`

Rules:

- no reusable view consumes raw backend schema directly
- no panel owns normalization logic
- no app skips the canonical domain to feed one special-case widget

### 2. Reusable Modules Stay App-Agnostic

Reusable modules must not know about:

- `deer_gui`
- specific transport choices such as NATS or LiveKit
- RTS or RPG fantasy naming
- specific screen or layout preset names

Reusable modules may know only:

- typed contracts
- generic commands and events
- rendering and interaction responsibilities

### 3. Authoring Tools Are First-Class Clients

Tool apps are not sidecars. They are proof that the architecture is working.

That means:

- the layout editor uses the same panel registry/runtime as the game
- the replay browser uses the same replay and timeline modules as the game
- the graph lab uses the same graph renderer and inspector as the game

### 4. Composition Over Invention

`deer_gui` should mostly compose already-proven modules.

If a major feature is born inside `deer_gui` first, that is a smell unless it is
truly app-specific framing.

## Architectural Success Criteria

The design is succeeding when:

- a module can be tested in isolation from any app
- a tool app can prove one module family end-to-end
- `deer_gui` adds mostly orchestration, fantasy framing, and layout composition
- new views can be reused without importing backend adapters or app enums
