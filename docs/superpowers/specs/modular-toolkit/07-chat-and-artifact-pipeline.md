# Design: Modular Toolkit - Chat And Artifact Pipeline

**Date:** 2026-03-31
**Status:** Accepted for planning

## Why This File Exists

The earlier modular-toolkit draft described data views well, but it
under-specified the live chat pipeline that DeerFlow already supports.

This file stress-tests the architecture against a real use case:

- initiate a research chat
- stream orchestration progress live
- compile outcomes across backend agents
- surface artifacts and clarification steps interactively

## DeerFlow Pipeline Summary

From the DeerFlow backend and frontend study, the UI-facing pipeline is:

1. create or resume a thread
2. upload optional files
3. start a run stream for the lead agent
4. receive live message, tool, and custom task-progress events
5. receive thread-state snapshots containing title, messages, artifacts, and
   optional todos
6. open artifacts through thread-scoped artifact endpoints
7. handle clarification tool messages as pause-and-resume UX, not as failures

Important backend facts:

- the lead agent orchestrates tools and optional subagents
- subagents emit custom progress events
- artifacts only become thread-visible when explicitly presented
- uploads and artifacts live in per-thread storage

## How The Spec Maps To This Pipeline

### Raw Sources Layer

This layer wraps transport-specific integration points such as:

- LangGraph thread creation and run-stream APIs
- embedded Python `DeerFlowClient.stream()` and `chat()`
- uploads gateway
- artifact gateway

Its job is to produce raw stream events and raw thread snapshots without leaking
transport details into views.

### Normalizer Layer

This layer translates raw DeerFlow events into canonical records.

Examples:

- `values` stream events -> thread snapshot updates
- `messages-tuple` -> incremental message or tool-call updates
- custom `task_*` events -> subtask progress records
- artifact lists -> canonical artifact records
- clarification tool messages -> clarification records

### Canonical Record Layer

This layer gives the rest of the toolkit stable concepts across the ontology
families.

For chat-oriented pipelines, the most important families are:

- carrier/orchestration:
  - `SessionRecord`
  - `RunRecord`
  - `MessageRecord`
  - `ToolCallRecord`
  - `ArtifactRecord`
  - `ClarificationRecord`
  - `TaskRecord`
  - `RuntimeStatusRecord`
- semantic spine where applicable:
  - `SourceRecord`
  - `SanitizedRecord`
  - `ViewRecord`
  - `InsightRecord`
  - `PredictionRecord`
  - `PrescriptionRecord`
  - `OutcomeRecord`
- representation/index:
  - `AsIsRepresentationRecord`
  - `ChunkRecord`
  - `EmbeddingRecord`
- governance/operational:
  - `IntentRecord`
  - `TransformRecord`
  - `WriteOperationRecord`
  - `ReplayCheckpointRecord`

That is the key architectural move: chat is not treated as a one-off widget. It
becomes a canonical multi-family state model that multiple apps can inspect.

### Derivation Layer

This layer shapes canonical records into view models for reusable UI.

Expected view models:

- `TranscriptVm`
- `ComposerVm`
- `ArtifactShelfVm`
- `RunStatusVm`
- `TaskProgressVm`
- `ThreadHeaderVm`

### Read-Model Layer

This layer owns transient experience state such as:

- draft prompt text
- attachment queue
- optimistic-send state
- stream connection status
- expanded tool-call cards
- selected artifact
- clarification pending state
- auto-open artifact panel behavior

### Reusable Views And Panels

The reusable UI for this pipeline should be split into lego bricks:

- conversation thread view
- prompt composer
- task-progress rail
- artifact shelf
- artifact file presenter
- detail inspector for messages, artifacts, and tool calls

These can be docked into runtime layouts through panel shells and the layout
runtime.

## What This Enables In `deer_gui`

With the layers above, `deer_gui` can compose the full research-chat flow:

- left or center panel for transcript and composer
- right panel for artifacts and inspectors
- status rail for subtask progress and tool activity
- top-level world view that reacts to the same canonical thread state

That means the user can:

- ask a research question
- watch orchestration live
- inspect intermediate tool activity
- open generated artifacts
- pivot from chat state into RTS or RPG intervention views

## Required Proving Ground

This use case exposes a real gap in the earlier milestone ladder:

- a live chat proving app is needed before `deer_gui`

So the spec now requires:

- `apps/deer_chat_lab` as the narrow proof app for the interactive chat and
  artifact pipeline

## Acceptance Criteria For This Slice

The architecture supports DeerFlow-style chat correctly only if a proof app can:

- create or resume a thread
- submit a prompt with optional uploads
- stream live assistant, tool, and task-progress updates
- render clarification interrupts as interactive pauses
- list and open persisted artifacts
- preview in-flight file outputs when available
- expose the same canonical thread state to other reusable views

## Stress-Test Conclusion

The modular-toolkit architecture can support the whole DeerFlow chat pipeline,
but only if chat is treated as a first-class reusable module family rather than
an app-specific panel.

That is now an explicit part of the spec.
