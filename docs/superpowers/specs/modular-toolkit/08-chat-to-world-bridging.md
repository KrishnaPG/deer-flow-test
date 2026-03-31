# Design: Modular Toolkit - Chat To World Bridging

**Date:** 2026-03-31
**Status:** Draft revision

## Problem

`deer_gui` needs a world that reacts to research chats, agent orchestration,
artifacts, and clarification steps.

The reusable chat toolkit must stay generic, so it cannot directly speak in RTS
or RPG terms.

This creates a required bridge:

- generic chat and artifact state in the toolkit
- app-specific world semantics in `deer_gui`

## Bridge Rule

The bridge happens through a dedicated world-projection layer.

It consumes canonical records such as:

- `ThreadSession`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `RunProgressRecord`
- `ClarificationRequest`
- agent, queue, and knowledge records from the same canonical domain

It produces `deer_gui`-facing world objects and view models.

The reusable chat toolkit does not know these world objects exist.

## Layer Placement

The bridge sits after canonical domain, before app-specific scene and HUD
composition:

`raw -> normalizer -> canonical domain -> world projection -> scene/hud VMs -> deer_gui`

This is not a bypass of the architecture.

It is the place where app-specific metaphor mapping is allowed.

## What The Bridge Owns

### 1. Semantic Projection

It decides how generic backend truth becomes game meaning.

Examples:

- thread session -> command channel, council scene, or dialogue anchor
- subtask progress -> task-force status or specialist action in flight
- clarification request -> intervention prompt or decision beacon
- artifact record -> world item, upgrade, codex entry, or mission reward
- tool activity -> operational event, queue action, or tactical ping

### 2. Stable Cross-View Identity

The same underlying truth must be traceable across views.

Examples:

- clicking a world marker opens the matching transcript/tool/artifact detail
- selecting a chat message can focus the matching unit, event, or sector marker
- a replay event can point back to the exact chat turn or artifact that caused it

### 3. Macro/Micro Continuity

The bridge guarantees that RTS and RPG views are two readings of the same
projected state, not two unrelated modes.

Examples:

- macro: a task force is stalled
- micro: the same stall appears as a clarification scene or council exchange

## What The Bridge Must Not Own

The bridge must not:

- parse raw transport payloads
- render transcript UI directly
- embed Bevy scene logic inside canonical records
- fork the chat toolkit into game-specific variants

## Mapping Examples

| Canonical Record | World Projection | Macro Use | Micro Use |
| --- | --- | --- | --- |
| `ThreadSession` | `WorldConversationAnchor` | command channel node | dialogue scene entry |
| `RunProgressRecord` | `WorldTaskForceState` | fleet activity/status | squad action beat |
| `ClarificationRequest` | `WorldInterventionPrompt` | alert beacon | decision dialogue |
| `ArtifactRecord` | `WorldArtifactUnlock` | upgrade/cargo output | codex/loot detail |
| `ToolCallRecord` | `WorldOperationEvent` | queue/event feed | inspected action step |

## Interaction Pattern

The bridge should support this loop:

1. user submits a research chat
2. canonical chat/task/artifact state updates from DeerFlow streams
3. world projection emits updated world objects
4. macro HUD and scene react with alerts, unit states, and markers
5. user drills into a world object
6. app orchestrator opens transcript, artifact, or intervention detail using the
   same stable IDs

This keeps world state reactive without making reusable chat modules game-aware.

## TDD Strategy For The Bridge

The bridge is app-specific, but it still follows contract-first testing.

Required tests:

- projection fixtures: canonical thread/task/artifact state -> projected world state
- linkage tests: world object IDs map back to source records
- drill-down tests: selecting projected objects opens the correct generic detail
- consequence tests: clarification/artifact outcomes update the right macro state

These projection fixtures should be previewable in `apps/deer_design` before
full gameplay composition in `apps/deer_gui`.

## Design Consequence

This stress test changes one important reading of the architecture:

- chat toolkit modules are reusable and generic
- world metaphor mapping belongs to a projection layer near app composition

That is the cleanest way to preserve both:

- toolkit reuse across non-game tools
- rich RTS/RPG meaning inside `deer_gui`

## Stress-Test Conclusion

The architecture supports chat-driven world state cleanly if and only if the
world projection layer is explicit.

That layer is now part of the spec.
