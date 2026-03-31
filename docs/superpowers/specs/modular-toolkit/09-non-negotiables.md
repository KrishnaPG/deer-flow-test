# Design: Modular Toolkit - Non-Negotiables

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The spec set now has enough moving parts that the core principles need one short,
front-door statement.

These are the architectural rules that planning and implementation must preserve.

## Non-Negotiable Principles

### 1. Toolkit First

The product is a reusable toolkit.

`deer_gui` is one composed client of that toolkit, not the architectural center.

### 2. No Layer Jumping

Every feature must follow a declared chain such as:

- `raw -> normalizer -> canonical domain -> derivation -> reusable view -> panel -> app`
- `raw -> normalizer -> canonical domain -> world projection -> scene/hud VM -> app`

No raw backend payloads in reusable views, panels, or world projection.

### 3. Reusable Modules Stay Generic

Reusable crates must not depend on:

- `deer_gui`
- transport-specific types
- RTS/RPG-specific enums or fantasy names
- layout preset names

### 4. Tool Apps Prove The Toolkit

Major slices must be proven in narrow apps before `deer_gui` composes them.

At minimum:

- `apps/deer_chat_lab`
- `apps/deer_replay`
- `apps/deer_graph_lab`
- `apps/deer_design`

### 5. TDD Happens At Module Boundaries

The required order is:

1. define contract
2. write failing fixture or golden test
3. implement minimal behavior
4. prove in a tool app
5. only then compose into `deer_gui`

### 6. Chat Is Domain State, Not A Widget

Chat must preserve:

- message deltas
- tool calls and tool results
- clarification interrupts
- task progress events
- artifact lifecycle

Reducing chat to final assistant text is architectural drift.

### 7. World Meaning Enters Only Through Projection

RTS/RPG metaphor mapping belongs in `world_projection` near app composition.

Reusable chat and view modules must stay generic.

### 8. Cross-View Identity Must Be Stable

Transcript, artifacts, replay, HUD, and world markers must stay traceable to the
same underlying records.

If a world object cannot drill back into its source chat/task/artifact state,
the architecture is incomplete.

### 9. Authority Split Must Stay Clean

- backend owns persisted truth
- client owns transient experience state

Do not mix draft prompts, selection, layout state, or optimistic UI state into
canonical persisted records.

## Forbidden Shortcuts

Do not allow:

- first complete implementations of reusable features inside `deer_gui`
- raw backend or LangGraph/DeerFlow types above `raw_sources` and `normalizers`
- panels or views performing normalization
- world projection containing renderer or UI framework logic
- giant dumping-ground crates such as `ui_core` or `shared_app`
- tool apps forking reusable runtime code
