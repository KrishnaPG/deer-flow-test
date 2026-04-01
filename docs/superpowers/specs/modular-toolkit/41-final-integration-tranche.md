# Design: Modular Toolkit - Final Integration Tranche

**Date:** 2026-04-01
**Status:** Draft revision

## Why This File Exists

`40-shell-and-linked-interaction-stress-test.md` showed that the current design
line is close, but not yet planning-ready.

This file is the final integration tranche that closes the remaining gaps across:

- world-primary shell support
- temporal, streaming, and failover semantics
- policy and access overlays
- intent lifecycle boundaries

## Pinned Rule

- planning readiness requires these concerns to be integrated into the shell
  model, not left as future implementation detail
- world-primary shells are peers to center-workspace shells, not optional visual
  flourishes
- shell-local linked state must be deterministic under failover, exclusion, and
  replay conditions
- linked gestures may prefill intent context, but may not silently execute work

## World-Primary Shell Support

### `battle_command`

Purpose:

- world-primary HOTL/HITL shell for tactical supervision, rapid command issuing,
  spatial awareness, and queue-backed operations

World projection requirement:

- `required`

Required views:

- `TacticalWorldView`
- `MinimapView`
- `GlobalVitalsView`
- `EventFeedView`
- `FleetTreeView`
- `InspectorTabsView`
- `SelectionSummaryView`
- `ActionDeckView`
- `QueueStatusView`

Optional views:

- `AttentionQueueView`
- `PinnedContextStripView`
- `ArtifactAccessView`
- `ReplayActivityView`

Required panels:

- `WorldViewportPanel`
- `MinimapPanel`
- `GlobalVitalsPanel`
- `EventRailPanel`
- `FleetRailPanel`
- `InspectorPanel`
- `BottomDeckPanel`

Required layout traits:

- `world_anchored_shell`
- `persistent_operational_frame`
- `persistent_edges`
- `selection_to_inspection_loop`
- `attention_without_takeover`
- `tiered_shell_visibility`
- `rail_compaction`
- `compound_bottom_deck`

### World / Camera / Minimap Rules

Add required linked-interaction primitives:

- `viewport_navigation`
- `camera_sync`

Rules:

- `TacticalWorldView` is the primary world interaction surface
- `MinimapView` is the authoritative overview and viewport locator
- `camera_sync` must propagate camera position, zoom, and frustum bidirectionally
  between world and minimap surfaces
- `viewport_navigation` may reposition camera only; it must not implicitly select
  targets or arm commands
- world picks may emit `selection` and optional `focus`
- minimap picks may emit `focus` only when explicitly used as locate or follow
- command targeting must remain distinct from navigation and camera movement

### Tiered Shell Visibility

- Tier 1
  - always-visible shell chrome: vitals, world viewport, minimap, compact rails,
    minimal inspector presence
- Tier 2
  - context-revealed operational deck: bottom deck opens on selection, command
    focus, or queue activity
- Tier 3
  - opaque overlays for deep inspection, forensics, settings, or blocking
    intervention; world interaction pauses, but canonical selection and draft
    context persist

### Collapsible Rails

- `EventRailPanel` and `FleetRailPanel` are independently collapsible persistent
  rails
- collapsed state becomes icon or badge strip, but linked participation survives
- collapse changes density, not semantic participation

### Compound Bottom Deck

- `BottomDeckPanel` contains three required sections:
  - `SelectionSummaryView`
  - `ActionDeckView`
  - `QueueStatusView`
- all three share brokered shell context but preserve different interaction
  semantics
- `ActionDeckView` is a fast contextual command surface with stable hotkey slots,
  not a free-form console replacement

### Coexistence With Existing Modes

- `battle_command` is a peer to `control_center` and `live_meeting`
- `control_center` remains broad orchestration-first monitoring
- `live_meeting` remains transcript-first and non-world-primary
- mode switches must preserve canonical selection, pins, and draft intent where
  the target mode supports the same references

## Temporal, Streaming, And Failover Semantics

### `replay_cursor`

Purpose:

- identify the singular active temporal inspection point shared across replay-
  capable surfaces

Required basis:

- `sequence_id`, `event_time`, or `checkpoint_id`
- source stream or canonical record family
- broker epoch
- origin panel and emission sequence
- resolution mode: `live_tail`, `checkpoint`, or `historical_event`

### `temporal_range`

Purpose:

- express a bounded time window for brushing, filtering, compare alignment, or
  replay scope

Required basis:

- lower and upper temporal anchors
- anchor kind
- inclusivity semantics
- granularity
- open-endedness where relevant

### Stream Lifecycle

Every live linked stream must expose one of:

- `connecting`
- `live`
- `catching_up`
- `degraded`
- `stalled`
- `recovered`
- `closed`

Rules:

- the shell must show whether a surface is at live tail or viewing historical
  state
- state transitions into degraded or recovered conditions must be operator-
  visible
- linked interaction must not imply freshness while replaying backlog

### Freshness And Staleness

Required metadata:

- freshness timestamp or last applied `sequence_id`
- staleness budget by shell mode
- supersession semantics for pins, compare members, and focus targets
- explicit stale reason

Rule:

- stale linked state may remain visible only when marked explicitly

### Operational Degradation

The shell must define operator-visible degradation behavior for:

- ingest lag
- backpressure
- partial stream loss
- unavailable enrichments
- storage slowdown with event-bus continuity

### Broker Recovery

Every shell mode with linked temporal behavior must define:

- broker owner by interaction type
- broker epoch semantics
- origin tracking and cycle suppression
- handoff rules for replica recovery or leadership change

Rule:

- recovered brokers must resume only under a new epoch

### Shell-State Durability And Rehydration

Shell-local state must be classified as:

- `discardable`
- `rehydratable`
- `recoverable_draft`
- `non_rehydratable`

Required recovery basis:

- shell-instance ID
- recovery fence or session epoch
- last acknowledged temporal anchor per mission, session, or run
- invalidation rules for stale or excluded references

Rule:

- rehydration must resolve through canonical references, never mutable cached
  payloads

### Late-Event Behavior

Rules:

- late events must be inserted by canonical temporal anchor, not append order
- late events may not silently move the active `replay_cursor` unless the
  operator is in `live_tail` mode
- if late events invalidate current focus, selection, compare, or pin context,
  the shell must surface explicit supersession or tombstone state

## Policy And Access Overlays

### Pinned Rules

- policy overlays are shell-level constraints applied on top of canonical truth
- every policy evaluation must carry a `policy_epoch` and `policy_reason`
- no linked interaction path may preserve access to an object the current policy
  excludes or masks beyond joinability

### Exclusions And Tombstones

- exclusions are authoritative deny overlays
- when an object becomes excluded, the shell must either:
  - replace direct detail with a tombstone state, where allowed, or
  - remove it entirely if even tombstone visibility is forbidden
- tombstones must not leak hidden content, derivative previews, or reopen paths

### ABAC Masking

- masked objects may participate in linked interaction only when remaining fields
  still preserve joinability and drill-down safety
- if masking removes required join keys, compare axes, or drill-down targets, the
  affected linked interaction path becomes unsupported for that object

### Large-Object Mediated Handoff

Large objects must use a `LargeObjectAccessContract` with:

- canonical target reference
- ABAC validation at request time
- representation or version identity
- handoff form such as pre-signed URL or streaming session token
- expiry, revocation, and audit metadata

Rule:

- large objects are never proxied inline through linked interaction brokers

### Policy-Driven State Invalidation

On relevant `policy_epoch` changes, shell brokers must recompute:

- `selection_state`
- `pin_set`
- `compare_set`
- `drill_down_targets`
- `focus_state`

Rules:

- excluded references must be dropped or tombstoned immediately
- no shell may preserve ghost selection, stale compare alignment, or reopen paths
  to pre-policy detail

### Intent Rejection Behavior

`action_intent_emission` must revalidate policy at `validated` and `submitted`
stages.

Rejected intents must return:

- rejected target references
- rejection class
- operator-safe reason text
- resulting draft or context transition

Do not auto-retarget intents to fallback objects.

## Intent Lifecycle Boundaries And Broker Ownership

Intent state remains shell-local and non-canonical through:

- `prefill`
- `draft`
- `validated`

Only `submitted` may append an `IntentRecord`.

`approved`, `executed`, and `rejected` are later observed mediated outcomes.

### Lifecycle Entry Rules

| Stage | May be entered by | Boundary |
| --- | --- | --- |
| `prefill` | any declared `action_intent_emission` source | non-submittable seed only |
| `draft` | `CommandConsoleView`, `IntentComposerView` | first editable operator-owned state |
| `validated` | `CommandConsoleView`, `IntentComposerView` | explicit validation against targets, parameters, and policy |
| `submitted` | `CommandConsoleView`, `IntentComposerView` | explicit submit action only |
| `approved` / `rejected` | approval surfaces or backend authority | adjudication of a submitted intent only |
| `executed` | mediated operation or result stream only | never entered directly by local gesture |

### Command-Target Brokerage

- every actionable shell mode must declare exactly one `command_target` broker
- sources may suggest command targets from selection, focus, pins, compare sets,
  replay anchors, or world picks
- only the broker may publish the normalized command-target bundle
- broker output must include actor or session context, canonical target refs,
  scope, provenance, policy visibility, and broker sequence or epoch

### Conflict Presentation

If current selection, focus, pinning, policy overlays, or approval scope produce
incompatible command targets, the broker must surface explicit conflict state
before `validated`.

Allowed resolutions:

- keep current draft target
- replace with latest broker target
- split into separate drafts

Unresolved conflicts block submission.

### Guardrails Against Accidental Execution

- linked gestures, brushing, replay scrubbing, world picks, camera moves, and
  attention interrupts may create or refresh `prefill` only
- no linked gesture may directly enter `validated`, `submitted`, `approved`, or
  `executed`
- `prefill -> draft`, `draft -> validated`, and `validated -> submitted` each
  require distinct explicit operator action
- target changes, exclusions, policy changes, or broker epoch changes after
  `validated` must demote the intent to `draft` or invalidate it visibly

## Planning Readiness Gate

If this tranche is accepted, the discovery line should be considered planning-
ready.

Planning should still remain blocked unless future review finds another missing
integration seam.
