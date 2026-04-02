# Design: Modular Toolkit - Generator Shell Mode Support Evaluation

**Date:** 2026-04-01
**Status:** Accepted for planning

## Why This File Exists

`37-shell-mode-support-matrix.md` defines what the shell requires.

This file evaluates the currently studied generators against that stricter shell
contract so future agents do not confuse strong raw backend capability with
actual shell support.

## Evaluation Rule

Support is judged as:

`generator -> canonical mappings -> view support -> panel support -> layout support -> shell mode support`

This evaluation also treats cross-view brushing and filtering as a required shell
capability where the shell mode declares it.

## Current Overall Finding

Under the current strict shell-mode contract, none of the four studied
generators currently reach `full` support for any of the canonical shell modes.

Most failures are not because the generators are useless.

Most failures happen because the shell contract now requires:

- co-present hosted views rather than isolated screens
- shared selection, brushing, and filtering across panels
- stable canonical mappings with drill-down-safe metadata
- explicit mediated reads and intent-backed actions
- dashboard-grade inspection and escalation loops

So the current generators are best understood as partial view providers and data
sources for shell composition, not as direct proofs of shell support.

## Coverage Snapshot

| Generator | `control_center` | `live_meeting` | `artifact_analysis` | `replay` | `forensics` |
| --- | --- | --- | --- | --- | --- |
| DeerFlow | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `unsupported` |
| Hermes | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `unsupported` |
| PocketFlow | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `unsupported` |
| Rowboat | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `unsupported` |

This does not mean all generators fail equally.

It means each one stops short on different required contracts.

## Generator Findings

### DeerFlow

Strongest canonical fit:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`

Closest shell matches:

- `live_meeting`
- `artifact_analysis`

Why it still fails shell support:

- conversation and artifact flows are strong, but shell-grade inspector, mission
  rail, attention queue, and center-mode coordination are not yet first-class
- replay and forensics remain embedded in chat/tool traces rather than promoted
  into replay-native or forensic-native contracts
- linked brushing/filtering across co-present views is weak

Practical reading:

- DeerFlow is a strong source for transcript-centric and artifact-adjacent view
  contracts
- DeerFlow should not yet be treated as proving `control_center`, `replay`, or
  `forensics`

### Hermes

Strongest canonical fit:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

Closest shell matches:

- `control_center`
- `live_meeting`

Why it still fails shell support:

- runtime, delivery, and carrier status are stronger than DeerFlow, but artifact
  representation and intent-composer evidence remain weaker
- shell-wide linked dashboard behavior across attention, command, transcript,
  and inspector surfaces is not established
- replay and forensics still lack explicit checkpointed or transform-linked
  contracts

Practical reading:

- Hermes is a strong source for vitals, mission/run state, and runtime-oriented
  status views
- Hermes should not yet be treated as proving artifact-heavy, replay-heavy, or
  forensics-heavy shell modes

### PocketFlow

Strongest canonical fit:

- `TaskRecord`
- `RuntimeStatusRecord`
- structural workflow topology
- retries and transitions

Closest shell matches:

- `replay`
- `forensics`

Why it still fails shell support:

- graph and execution structure are strong, but messages, clarifications,
  artifact lineage, and intent-backed operator surfaces are sparse
- replay and debugging patterns exist more as externalized patterns than as a
  stable replay-checkpoint contract
- linked brushing/filtering remains weak because the required shell joins are not
  canonically anchored across transcript, artifact, and inspection views

Practical reading:

- PocketFlow is a strong source for execution-topology and activity-rail-style
  views
- PocketFlow should not yet be treated as proving meeting-grade or
  artifact-analysis-grade shell support

### Rowboat

Strongest canonical fit:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- provenance-heavy file/version knowledge

Closest shell matches:

- `live_meeting`
- `artifact_analysis`

Why it still fails shell support:

- surfaced UI patterns are closer to chat plus resource browser plus detail pane
  than to a full shell with mission rail, attention queue, global vitals, and
  linked inspector loops
- representation and provenance fit is promising, but the shell contract needs
  side-by-side linked analysis and structured dashboard interaction, not only
  shelf-to-detail navigation
- replay and forensics still lack explicit canonical checkpoint/log contracts at
  the shell point of use

Practical reading:

- Rowboat is a strong source for artifact, provenance, and knowledge-adjacent
  view contracts
- Rowboat should not yet be treated as proving runtime-ops or replay-forensics
  shell support

## Shell-By-Shell Findings

### `control_center`

Closest generator:

- Hermes

Why the category still fails:

- the mode requires `GlobalVitalsView`, `MissionRailView`, `AttentionQueueView`,
  `CommandConsoleView`, `InspectorTabsView`, and `CenterCanvasModeSwitcherView`
  to coexist with linked brushing/filtering
- no current generator proves that full operational frame as a supported shell
- Hermes comes closest because runtime, delivery, and mission-like status are
  richer, but attention and shell-wide linked command/inspect loops are still not
  pinned

### `live_meeting`

Closest generators:

- DeerFlow
- Rowboat
- Hermes

Why the category still fails:

- strong transcript support exists, but the shell contract also requires
  `SessionRunHeaderView`, `IntentComposerView`, `InspectorTabsView`, and linked
  coordination among them
- current backends are closer to chat-plus-sidecar than to a fully linked meeting
  shell

### `artifact_analysis`

Closest generators:

- Rowboat
- DeerFlow

Why the category still fails:

- artifact browse/open behavior exists, but the shell contract requires
  inspector-led comparison, representation-aware access, and linked filtering
  across artifact, retrieval, and semantic views
- current generators stop at strong artifact access rather than dashboard-grade
  analysis composition

### `replay`

Closest generator:

- PocketFlow

Why the category still fails:

- replay needs checkpointed sequence reconstruction with stable drill-down-safe
  metadata
- execution traces and ordered events are not enough unless they are promoted to
  `ReplayCheckpointRecord`-backed views and coordinated inspector behavior

### `forensics`

Closest generator:

- PocketFlow

Secondary candidate:

- Hermes

Why the category still fails:

- forensics requires explicit log-source backlinks, deep inspection escalation,
  and transform/replay-safe provenance at the point of use
- current generators expose enough raw material to inspire the mode, but not to
  prove canonical forensic shell support

## Linked Brushing And Filtering Readout

| Generator | Likely current strength |
| --- | --- |
| DeerFlow | `weak` |
| Hermes | `medium` within carrier/runtime views, but not shell-wide |
| PocketFlow | `weak` overall, `weak-medium` for execution topology slices |
| Rowboat | `weak` to `weak-moderate` for artifact/detail adjacency |

Current implication:

- no studied generator should be treated as proving dashboard-grade linked
  brushing/filtering under the shell contract
- linked interaction support will need to be designed at the canonical toolkit
  composition layer, not assumed from backend-native UI fragments

## Design Consequences

- backend evaluation should continue to name strongest candidate shell modes per
  generator, but still mark them `unsupported` when required contracts are absent
- the toolkit must own the composition logic that upgrades isolated view support
  into linked shell support
- future backend adapters should be judged partly by whether they improve
  drill-down-safe metadata, canonical replay/forensics evidence, and cross-view
  joinability
- implementation planning should not start from the assumption that one existing
  backend UI already proves the target shell

## Recommended Next Step

The next design pass should define the canonical linked-interaction layer that
bridges:

- shared shell selection
- brushing and filtering state
- cross-panel correlation keys
- drill-down-safe focus propagation
- intent emission from any qualifying hosted view

That layer is now the main missing bridge between strong isolated views and true
dashboard-grade shell support.
