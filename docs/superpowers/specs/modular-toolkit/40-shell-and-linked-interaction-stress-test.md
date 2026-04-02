# Design: Modular Toolkit - Shell And Linked Interaction Stress Test

**Date:** 2026-04-01
**Status:** Accepted for planning

## Why This File Exists

`36` through `39` now define:

- view support
- shell-mode support
- backend shell evaluation
- linked interaction contracts

Before writing plans, those contracts must survive two harder tests:

- the world-first sci-fi RTS HUD shell
- the storage-native state-server stress scenarios

This file records that stress-test pass, the gaps it exposed, and the
reconciliation targets now integrated in `41-final-integration-tranche.md`.

## Overall Finding

The shell and linked-interaction model was directionally correct but needed one
final integration tranche before planning.

Those required corrections are now captured in
`41-final-integration-tranche.md`; this document should be read as the
rationale for that tranche, not as a standing blocker once `41` is accepted.

It is strongest on:

- linked selection and inspection loops
- shell composition by views, panels, and layout traits
- mediated action intent as a first-class idea

It is weakest on:

- world-primary shells
- minimap and camera linkage
- tiered shell visibility
- streaming and replay continuity
- policy and exclusion overlays
- failover and rehydration behavior
- deterministic broker ownership

## Stress Test A: Sci-Fi RTS HUD

### Confirmed Fits

- the `view -> panel -> layout -> shell mode` ladder remains the right
  evaluation model
- role-based panels still fit the RTS shell better than fixed geometry
- `selection`, `focus`, `drill_down`, and `action_intent_emission` already map
  well to world picks, fleet selection, inspection, and orders
- `attention_without_takeover` and `selection_to_inspection_loop` match the RTS
  pattern where alerts stay ambient and detail blooms only when needed

### Mismatches

- current shell framing is still center-workspace oriented, while the RTS HUD is
  world-primary
- `control_center` currently treats world projection as optional, but the RTS HUD
  requires both a central world viewport and a persistent minimap
- the current panel set does not cleanly separate chronological event feed from
  hierarchical fleet tree
- `CommandDeckPanel` is still modeled too much like command composition and not
  enough like a fast contextual action deck with stable hotkey slots
- current linked minimums over-emphasize filtering; RTS operation is primarily
  selection, navigation, focus, and command targeting

### Missing Contracts

- `TacticalWorldView` contract
- `MinimapView` contract
- `viewport_navigation` primitive
- `camera_sync` primitive
- `rail_compaction` contract for collapsible persistent rails
- `tiered_shell_visibility` contract for persistent chrome, contextual lower deck, and
  opaque tier-3 overlays
- compound bottom-deck contract for selection, command, and queue sections that
  share context but differ in behavior

### Design Consequence

The shell model must support world-primary modes, not only center-workspace
transclusion modes.

## Stress Test B: State-Server Scenarios

### Confirmed Fits

- HOTL pipeline monitoring remains a good conceptual fit for `control_center`
- HITL live interaction remains a good conceptual fit for `live_meeting`
- mediated reads already reject direct storage or database access as shell support
- intent-based action flow already aligns with conflicting-intent scenarios in
  principle

### Mismatches

- burst, lag, and backpressure behavior are not yet modeled at the shell layer
- exclusion and compliance overlays are not first-class linked-interaction rules
- large-object access is not yet modeled as a special mediated handoff contract
- streaming, failover, and shell-state rehydration are not defined
- deterministic operator-visible conflict handling is still under-specified
- shell modes claim fields like required linked interactions and mediated reads,
  but most mode sections do not yet fill them exhaustively

### Missing Contracts

These concern labels are now absorbed into the integrated shell-level rules in
`41-final-integration-tranche.md`; they do not require separate follow-on
documents if `41` is accepted.

- `StreamContract`
- `OperationalDegradationContract`
- `PolicyOverlayContract`
- `LargeObjectAccessContract`
- `BrokerRecoveryContract`
- `IntentConflictContract`

### Design Consequence

These scenarios established the missing contracts that had to be added for
storage-native readiness.

Those contracts are now the acceptance checklist for
`41-final-integration-tranche.md`, rather than a separate unresolved blocker.

## Adversarial Findings Against Linked Interaction

### Broker Authority Is Under-Specified

Current problem:

- the spec requires one broker per interaction type, but does not assign brokers
  per shell mode or define broker handoff rules

Failure risk:

- split-brain selection
- stale command targets
- disagreeing center and command surfaces

Required fix:

- add per-shell-mode broker maps
- add broker epoch or sequence semantics
- add cycle-suppression and origin-tracking rules

### Shell-Local State Has No Failover Contract

Current problem:

- shell state is defined as local and non-canonical, but there is no durability,
  rehydration, or invalidation matrix

Failure risk:

- stale pins
- resurrected compare sets
- lost drafts
- diverging state after replica recovery

Required fix:

- define which state is client-local, server-rehydratable, or discardable
- add shell-instance identity and recovery fence semantics
- define invalidation rules for stale local state

### Temporal And Replay Linkage Is Incomplete

Current problem:

- shell minimums require replay cursor brokerage, but the interaction spec does
  not yet define first-class replay cursor or temporal range state

Failure risk:

- unstable replay scrubbing
- inconsistent time-linked filtering
- broken late-event handling

Required fix:

- add `replay_cursor` primitive
- add `temporal_range` primitive
- require anchors such as `sequence_id`, `event_time`, and `checkpoint_id`

### World And Minimap Contracts Are Too Implicit

Current problem:

- world picks are allowed, but minimap, camera, frustum, and region-brush
  semantics are not yet pinned

Failure risk:

- world click meaning is ambiguous
- minimap and world drift apart
- camera movement and command targeting blur together

Required fix:

- add world/minimap appendix with strict world interaction semantics
- separate camera movement from selection and command targeting

### Compliance Overlays Are Missing

Current problem:

- exclusions can hide objects immediately, but the linked-interaction spec does
  not define how selected, pinned, compared, or replayed objects behave when they
  become excluded

Failure risk:

- ghost selections
- leaked counts
- pinned-context backdoors

Required fix:

- add exclusion and tombstone semantics
- recompute selection, pin, compare, and drill-down state under policy overlays
- reject intent emission against excluded targets with explicit policy reasons

### Intent Prefill Vs Submission Is Too Blurry

Original gap:

- `action_intent_emission` currently mixes prefill and submission too loosely

Failure risk:

- accidental escalation from selection to executable action

Integrated resolution in `41-final-integration-tranche.md`:

- widen the entry model so linked or shell carry-in lands first as
  `prefill_seed`
- require promotion through `prefill`, `draft`, `validated`, and explicit
  `submitted`
- keep `approved` / `executed` / `rejected` as later mediated outcomes
- limit linked-interaction surfaces to seeding entry, not direct submission

This stress-test gap is therefore resolved if `41` is accepted and propagated
back into `20`, `37`, and `39`; it is not a separate open blocker after that
integration.

## Resolved Adjustment Checklist

`41-final-integration-tranche.md` should be accepted only if it closes all of
the following stress-test corrections:

1. world-primary shell support
   - `battle_command` or equivalent world-primary shell mode
   - `TacticalWorldView`
   - `MinimapView`
   - `viewport_navigation`
   - `camera_sync`

2. tiered shell visibility
   - persistent shell chrome
   - context-revealed decks
   - opaque modal overlays
   - compaction and collapse rules for rails

3. broker and failover semantics
   - broker map by shell mode
   - broker handoff rules
   - durability and rehydration matrix for shell-local state

4. temporal and operational semantics
   - `replay_cursor`
   - `temporal_range`
   - stream and freshness rules
   - lag and degraded-state signaling

5. policy and access overlays
   - exclusion and tombstone behavior
   - large-object mediated handoff contract
   - intent rejection rules under policy constraints

6. intent lifecycle separation
   - shell-local `prefill_seed`, `prefill`, `draft`, and `validated`
   - canonical `IntentRecord` append only at explicit `submitted`
   - `approved` / `executed` / `rejected` observed later as mediated outcomes

## Readiness Decision

Do not use this file by itself as the planning gate.

The planning gate is whether `41-final-integration-tranche.md` is accepted as
closing the six correction areas above.

If `41` is accepted, `writing-plans` may begin. If `41` is rejected or
reopened, planning remains blocked.
