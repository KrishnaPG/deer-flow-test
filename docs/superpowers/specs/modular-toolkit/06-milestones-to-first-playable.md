# Design: Modular Toolkit - Milestones To First Playable

**Date:** 2026-03-31
**Status:** Draft revision

## Core Sequence

The path to first playable is not game-first.

It is:

- build reusable module by contract
- test it in isolation
- prove it in a narrow tool app
- repeat until a small toolkit exists
- compose `deer_gui` from proven bricks

## Milestone Ladder

### M0 - Foundation Spine

Build and test:

- `foundation/contracts`
- `foundation/domain`
- `foundation/replay`
- `pipeline/normalizers`
- `pipeline/derivations`
- `runtime/read_models`

Exit criteria:

- fixtures can flow cleanly from raw input to canonical domain to view models
- transient client state is testable without any app shell

### M1 - Live Chat Toolkit Proof

Build and test:

- conversation thread module
- prompt composer module
- artifact shelf and file presenter module
- stream and task-progress normalization

Prove in:

- `apps/deer_chat_lab`

Exit criteria:

- a thread can be created or resumed
- a live run stream renders message, tool, and task progress updates
- uploads work before submit
- artifacts can be opened after `present_files`
- clarification interrupts pause and resume cleanly

### M2 - Replay Toolkit Proof

Build and test:

- timeline/replay module
- event feed module
- detail inspector module

Prove in:

- `apps/deer_replay`

Exit criteria:

- replay fixtures load
- timeline scrubs
- events and details stay in sync

### M3 - Graph Toolkit Proof

Build and test:

- graph view module
- graph interaction state
- graph detail integration

Prove in:

- `apps/deer_graph_lab`

Exit criteria:

- graph data renders from generic `GraphVm`
- clustering, selection, and inspection work outside `deer_gui`

### M4 - Layout Toolkit Proof

Build and test:

- panel shells
- panel registry
- layout runtime
- view host bridge

Prove in:

- `apps/deer_design`

Exit criteria:

- panels register by contract
- layouts save and restore
- chat, graph, replay, and inspector views embed cleanly in one runtime

### M5 - Spatial Toolkit Proof

Build and test:

- actor cloud view
- telemetry map view
- scene host integration points

Prove in:

- `apps/deer_design` or a narrow spatial sandbox

Exit criteria:

- spatial views render typed view models
- selection and focus events flow back through reusable contracts

### M6 - First Playable Composition

Compose in:

- `apps/deer_gui`

Using already-proven bricks:

- live chat and artifact access
- replay and timeline
- event feed and detail inspector
- graph/minimap slice
- layout runtime
- actor cloud and telemetry slice

Exit criteria:

- one closed loop exists:
  send research request -> watch live orchestration -> inspect artifacts/results
  -> select units/agents -> react to alert/failure -> drill into intervention
  -> return to macro state

## What Counts As "First Playable"

The first playable version is not full content breadth.

It is the first composed experience where:

- live chat and artifact retrieval are usable
- the RTS shell is navigable
- at least one intervention path changes macro state
- replay and inspection explain what happened
- the experience is built mostly from previously proven toolkit modules

## Stress-Test Conclusion

If we follow this spec strictly:

- `deer_chat_lab` proves the live chat and artifact slice
- `deer_replay` proves the first toolkit slice
- `deer_graph_lab` proves the second toolkit slice
- `deer_design` proves composition and authoring
- `deer_gui` becomes the first playable app after those proofs, not before
