# Design: Modular Toolkit - Chat Event And State Mapping

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

Interactive chat clients need a generic event and state model before they can be
normalized into canonical records.

This file defines that generic middle layer.

## Generic Chat Event Families

### Session Events

Examples:

- session created
- session resumed
- session titled or updated
- session ended or stopped

### Intent Events

Examples:

- prompt submitted
- clarification response submitted
- approval or denial submitted
- intervention requested

### Message Events

Examples:

- message started
- text delta
- reasoning delta
- message completed

### Tool Events

Examples:

- tool call started
- tool progress updated
- tool result available
- tool error

### Task / Progress Events

Examples:

- task created
- task running
- task blocked
- task completed
- task failed

### Clarification Events

Examples:

- clarification requested
- clarification answered
- approval requested
- approval resolved

### Artifact Events

Examples:

- upload staged
- artifact created
- artifact presented
- artifact preview ready
- artifact downloadable

### Runtime Events

Examples:

- connection ready
- stream connected
- reconnecting
- runtime degraded
- runtime exited

## Mapping Rule

Every chat-capable generator/client should map into these generic event families
before canonical normalization.

That means:

- generator-specific event names stop at the raw envelope layer
- generic chat event families bridge raw envelopes and canonical records
- canonical records stay generator-agnostic

## Canonical Mapping Expectations

Typical mappings:

- session events -> `SessionRecord`, `RunRecord`
- intent events -> append-only intent and follow-up run/task records
- message events -> `MessageRecord`
- tool events -> `ToolCallRecord`
- task/progress events -> `TaskRecord`
- clarification events -> `ClarificationRecord`
- artifact events -> `ArtifactRecord`
- runtime events -> `RuntimeStatusRecord`

## State Machine Requirement

For interactive chat clients, read-model state should explicitly support:

- draft prompt state
- staged attachment state
- optimistic send state
- stream connected/disconnected state
- clarification pending state
- selected artifact state
- expanded tool/progress detail state

## Anti-Drift Rule

Do not map raw generator events directly into reusable views.

They must pass through generic event families and canonical records first.
