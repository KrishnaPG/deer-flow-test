# Design: Modular Toolkit - Interactive Chat Lifecycle

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The toolkit needs a generic lifecycle for interactive chat-oriented clients.

DeerFlow is one example, but not the only one.

This lifecycle should also fit other chat-capable systems that support some mix
of:

- session or thread context
- prompt submission
- streaming responses
- tool activity
- human clarification or approval
- artifact generation or presentation

## Lifecycle Stages

### 1. Session Establishment

The client creates, resumes, or attaches to a conversational execution context.

Examples:

- thread creation
- session resume
- delegated child conversation

### 2. Optional Input Preparation

The client prepares user-visible inputs before execution.

Examples:

- prompt draft
- attachment queue
- upload staging
- tool or mode selection

### 3. Intent Submission

The client submits a user intent into the mediated execution boundary.

Examples:

- send prompt
- respond to clarification
- approve or deny action
- request intervention

### 4. Execution Streaming

The system emits live updates while work is in progress.

Examples:

- assistant text deltas
- reasoning or progress deltas
- tool-call lifecycle updates
- task or subtask progress
- runtime status changes

### 5. Interactive Pause / Clarification

The execution can pause for a human decision or missing information.

Examples:

- clarification question
- approval gate
- permission request
- human feedback request

### 6. Artifact And Output Presentation

The execution may produce outputs that become visible to the user only after
presentation or authorization.

Examples:

- generated file
- previewable result
- retrieval result
- summarized output bundle

### 7. Completion Or Continuation

The interaction either completes, errors, stops, or continues into another turn.

Examples:

- successful run completion
- partial completion with artifacts
- interruption
- reconnect/resume
- next-turn continuation

## Lifecycle Rule

These stages are generic and should not depend on DeerFlow naming.

Each generator/client may implement the stages differently, but the toolkit
should map them into the same canonical lifecycle.

## Canonical Records Involved

Interactive chat-oriented clients will usually populate some subset of:

- `SessionRecord`
- `RunRecord`
- `MessageRecord`
- `TaskRecord`
- `ToolCallRecord`
- `ClarificationRecord`
- `ArtifactRecord`
- `RuntimeStatusRecord`
- optional `DeliveryRecord`

## View Implication

At minimum, a generic chat-oriented toolkit should be able to support:

- session/thread header
- transcript and prompt composer
- tool/progress rail
- clarification surface
- artifact shelf and artifact presenter

## Anti-Drift Rule

Do not design the chat lifecycle around a single generator's event names or API
surface.
