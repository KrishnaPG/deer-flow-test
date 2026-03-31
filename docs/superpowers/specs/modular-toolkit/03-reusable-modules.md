# Design: Modular Toolkit - Reusable Modules

**Date:** 2026-03-31
**Status:** Draft revision

## Reusable Lego Bricks

The first reusable module families should be:

1. conversation thread and prompt composer
2. artifact shelf and file presenter
3. timeline and replay
4. event feed and detail inspector
5. graph view
6. panel registry and layout runtime
7. actor cloud and telemetry map

These are broad enough to support multiple apps and concrete enough to prove the
toolkit quickly.

## Module Boundaries

### Conversation Thread And Prompt Composer

Purpose:

- render live transcript state for human, assistant, and tool activity
- collect prompt text, attachments, and clarification responses

Consumes:

- `TranscriptVm`
- `ComposerVm`
- run status and stream state

Emits:

- send message command
- stop run command
- resume clarification command
- attachment add or remove commands

### Artifact Shelf And File Presenter

Purpose:

- show thread-visible artifacts, in-flight file outputs, and file previews

Consumes:

- `ArtifactShelfVm`
- selected artifact state

Emits:

- open artifact
- download artifact
- open-in-panel command
- install or import command where supported

### Timeline And Replay

Purpose:

- show traces, histories, meetings, branches, and playback cursors

Consumes:

- `TimelineVm`
- `ReplayRecord`
- interaction state for scrub, bookmark, and selection

Emits:

- scrub events
- selection events
- branch or anomaly focus events

### Event Feed And Detail Inspector

Purpose:

- show structured events and inspect selected records deeply

Consumes:

- `EventFeedVm`
- `DetailCardVm`
- `InspectorVm`

Emits:

- selection and focus commands
- open-in-panel or drill-down commands

### Graph View

Purpose:

- render knowledge graphs, dependencies, lineage, and sector maps

Consumes:

- `GraphVm`
- graph interaction state

Emits:

- node selection
- cluster collapse and expand
- path highlight or focus commands

### Panel Registry And Layout Runtime

Purpose:

- register reusable panels and compose them into layouts

Consumes:

- panel descriptors
- shell contracts
- layout serialization state

Emits:

- layout mutations
- panel lifecycle events
- modal and dock commands

### Actor Cloud And Telemetry Map

Purpose:

- render dense entity fields and spatial summaries for the RTS shell

Consumes:

- `ActorCloudVm`
- `TelemetryMapVm`

Emits:

- selection
- camera/focus hints
- hover and hotspot events

## Contract Style

Recommended shape:

- raw-source traits live in `foundation/contracts`
- canonical records live in `foundation/domain`
- typed view models live in `pipeline/derivations`
- views consume view models, not raw responses or app structs

Good:

- `GraphView::render(&GraphVm, &GraphInteractionState)`

Bad:

- `GraphView::render(&RawKnowledgeGraphResponse)`

## Rule For Reuse

Each module must clearly specify:

- accepted input type
- emitted events
- allowed commands
- persistence expectations
- runtime or threading assumptions

If those are unclear, the module is not ready to be reused.

## Chat-Pipeline Rule

For DeerFlow-style chat, the reusable UI must preserve message and tool meaning,
not just final assistant text.

That means reusable modules must support:

- assistant message deltas
- tool-call and tool-result rendering
- clarification interrupts
- subtask progress events
- thread-level artifact inventory and live file previews
