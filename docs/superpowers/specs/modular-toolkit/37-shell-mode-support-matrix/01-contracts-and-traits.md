## Why This File Exists

`36-view-contract-support-model.md` pins `view contract` as the primary support
unit.

That still leaves one important gap: the cinematic shell is experienced as an
operating mode, not as an isolated view.

This file defines the missing shell-level layer so support can be judged as:

`backend -> canonical mappings -> view support -> panel support -> layout support -> shell mode support`

## Pinned Rule

- `shell mode` is the primary unit of operator experience
- `view contract` remains the primary atomic unit of backend support
- `panel` support is derived from hosted view support
- `layout` support is derived from required panel coexistence and capabilities
- `shell mode` support is derived from required views, required panels, required
  layout traits, linked dashboard interactions, mediated reads, and required
  intents

## Shell Mode Contract

Every `ShellModeContract` must define enough detail, directly or by normative
inheritance, to make shell-support judgment possible:

- `purpose`
- `HOTL/HITL affinity`
- `required views`
- `optional views`
- `required panels`
- `required layout traits`
- `required linked dashboard interactions`
- `required canonical record families`
- `required metadata`
- `required mediated reads`
- `required intents`, where actionable
- `required broker ownership map` for each required linked interaction type
- `required temporal semantics`, where the shell mode is replay-capable or
  live-streamed
- `required policy/access overlay behavior`, where exclusions, masking, or
  tombstones may affect linked state
- `required shell-state durability and rehydration rules`
- `required command-target behavior`, where the shell mode is actionable
- `world projection requirement`: `required`, `optional`, or `not_applicable`
- `degradation behavior`
- `unsupported conditions`

Shared inherited requirements:

- `Required Metadata Baseline` in this file applies to every shell mode unless a
  mode tightens it
- `39-linked-interaction-contract.md` provides the default linked-interaction,
  broker, metadata, degradation, and support-judgment rules for any shell mode
  that declares those interaction paths
- `41-final-integration-tranche.md` provides the default temporal, failover,
  policy-overlay, and intent-boundary rules for any shell mode that exposes
  those capabilities
- mode sections only need to restate inherited requirements when they narrow,
  extend, or override them

## Canonical Panel Contracts

Panels are role-based hosts, not pixel positions.

| Panel | Purpose | Required hosted views | Optional hosted views |
| --- | --- | --- | --- |
| `GlobalVitalsPanel` | persistent global health and burn framing | `GlobalVitalsView` | none |
| `MissionRailPanel` | mission/workflow/swarm navigation | `MissionRailView` | `ActivityRailView` |
| `AttentionPanel` | approvals, blockers, actionable interrupts | `AttentionQueueView` | `InterventionGateView` |
| `CommandDeckPanel` | command entry and intervention issuing | `CommandConsoleView` | `IntentComposerView` |
| `InspectorPanel` | current selection detail and actions | `InspectorTabsView` | `ArtifactAccessView` |
| `CenterCanvasPanel` | central transcluded working surface | at least one center-bearing hosted view appropriate to the shell mode; `CenterCanvasModeSwitcherView` is additionally required where `mode_switchable_center` is declared | `PinnedContextStripView`, `SearchRetrievalResultsView` |
| `ArtifactShelfPanel` | artifact browsing and attachment surfacing | `ArtifactShelfView` | `ArtifactAccessView` |
| `ReplayPanel` | replay and event-sequence inspection | `ReplayActivityView` | `ForensicsLogView` |
| `ModalDetailPanel` | fullscreen deep inspection | one of `ForensicsLogView`, `ArtifactAccessView`, `ReplayActivityView` | `SearchRetrievalResultsView` |
| `WorldCanvasPanel` | cinematic background or world-bearing projection host | no required hosted view for first-wave non-blocking shell support | future world-bearing views |
| `WorldViewportPanel` | primary world-bearing operational surface | `TacticalWorldView` | `ReplayActivityView`, `PinnedContextStripView` |
| `MinimapPanel` | authoritative overview and viewport locator | `MinimapView` | none |
| `EventRailPanel` | persistent temporal/event rail for live tail and backlog awareness | `EventFeedView` | `AttentionQueueView` |
| `FleetRailPanel` | persistent hierarchy/asset/mission rail | `FleetTreeView` | none |
| `BottomDeckPanel` | contextual command, selection summary, and queue state deck | `SelectionSummaryView`, `ActionDeckView`, `QueueStatusView` | `ArtifactAccessView` |

## Canonical Layout Traits

Layouts are judged by capability, not fixed geometry.

- `persistent_operational_frame`
  - global status and command entry remain available without entering a modal
- `persistent_edges`
  - mission/navigation and inspection can coexist while center work continues
- `dominant_center_canvas`
  - the center can host one primary work mode without collapsing the shell
- `selection_to_inspection_loop`
  - mission, artifact, replay, or world selections can open immediate detail
- `cross_view_brushing_and_filtering`
  - side-by-side views can participate in the shell mode's declared shared
    selection, brushing, filtering, and focus paths without becoming semantic
    silos
- `attention_without_takeover`
  - approvals and failures surface persistently without forcing every event into
    a modal
- `mode_switchable_center`
  - the center can switch among meeting, artifact, replay, and forensics modes
- `deep_inspection_escalation`
  - logs, lineage, and raw detail can move into fullscreen inspection when needed
- `world_anchored_shell`
  - the shell can coexist with a cinematic background or world projection layer
- `tiered_shell_visibility`
  - the shell supports always-visible world chrome, context-revealed
    operational decks, and blocking overlays without losing canonical selection
    or draft context
- `rail_compaction`
  - persistent rails may collapse to dense icon or badge strips without dropping
    linked participation
- `compound_bottom_deck`
  - a single panel can host distinct summary, command, and queue sections with
    shared brokered context but separate interaction semantics

