# Design: Modular Toolkit - Authoring Tools As Proving Grounds

**Date:** 2026-03-31
**Status:** Draft revision

## Why Tool Apps Exist

Tool apps are the mechanism that proves the toolkit is real.

Without them, the project risks building app-specific code inside `deer_gui` and
calling it reusable later.

## Tool App Roles

### `apps/deer_replay`

Purpose:

- prove replay loading, timeline rendering, event feed, and detail inspection

What it validates:

- replay fixtures can drive the domain cleanly
- timeline interactions work without game framing
- detail panels are useful outside the main app

### `apps/deer_chat_lab`

Purpose:

- prove live interactive chat, thread streaming, uploads, clarification, and
  artifact access

What it validates:

- a prompt can create or resume a thread and start a run stream
- message deltas, tool calls, and subtask progress render correctly
- uploads and artifacts work through reusable contracts
- clarification pauses and resumes are handled outside `deer_gui`

### `apps/deer_graph_lab`

Purpose:

- prove graph rendering, clustering, selection, and inspection

What it validates:

- graph view is generic enough for multiple data sets
- interaction patterns are not tied to one RTS screen
- layout and inspector integration stays reusable

### `apps/deer_design`

Purpose:

- prove panel composition, layout editing, view hosting, and theme experiments

What it validates:

- panel registry is data-driven
- layout serialization is stable
- views can be arranged, embedded, and reused without `deer_gui`

### `apps/deer_gui`

Purpose:

- compose proven modules into the first playable hybrid RTS/RPG shell

What it should add:

- fantasy framing
- cross-view orchestration
- spatial/gameplay composition
- intervention flow between macro and micro views

What it should not own first:

- first-ever live chat transcript implementation
- first-ever uploads or artifact browser implementation
- first-ever graph implementation
- first-ever replay implementation
- first-ever layout runtime

## Proof Rule

Each major toolkit slice should pass this ladder:

1. contract and tests exist
2. module works in isolation
3. one narrow tool app proves it end-to-end
4. only then can `deer_gui` depend on it

## Architectural Smell Tests

The architecture is drifting if:

- `deer_gui` contains the most complete version of a reusable view
- tool apps fork runtime code instead of sharing it
- a tool app needs app-specific enums to function
- a layout authored in `deer_design` cannot be reused in `deer_gui`
