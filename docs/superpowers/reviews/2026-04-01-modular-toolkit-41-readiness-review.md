# Modular Toolkit 41 Readiness Review

**Date:** 2026-04-01
**Scope:** `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
**Basis:** Cross-check against `20-intent-and-mediated-read-model.md`, `37-shell-mode-support-matrix.md`, `39-linked-interaction-contract.md`, and `40-shell-and-linked-interaction-stress-test.md`

## Verdict

`41-final-integration-tranche.md` is **mostly ready**, but not yet fully planning-ready as-is.

It closes most of the gaps raised by `40-shell-and-linked-interaction-stress-test.md`, but it still needs a narrow reconciliation pass across the surrounding specs before the line can be treated as fully integrated.

## What 41 Successfully Closes

- Adds explicit world-primary shell support through `battle_command`
- Adds `TacticalWorldView`, `MinimapView`, world/minimap linkage, and camera/navigation rules
- Adds tiered shell visibility, collapsible rails, and compound bottom-deck behavior
- Adds temporal primitives and shell-level streaming/failover semantics
- Adds freshness, staleness, late-event, and recovery rules
- Adds policy overlays, tombstones, masking, large-object mediated handoff, and invalidation behavior
- Tightens intent lifecycle boundaries and anti-accidental-execution rules

## Remaining Integration Gaps

### 1. Command-surface ambiguity

`41-final-integration-tranche.md` makes `ActionDeckView` the fast contextual command surface for `battle_command`, but the lifecycle rules still say only `CommandConsoleView` and `IntentComposerView` may enter `draft`, `validated`, or `submitted`.

Result: rapid command issuance is still under-specified.

## 2. `39-linked-interaction-contract.md` is not fully updated

`39-linked-interaction-contract.md` still models actionable-shell routing around `CommandConsoleView` and `IntentComposerView`, and does not yet integrate the new world-primary interaction and temporal primitives introduced by `41`.

Missing integration points include:

- `viewport_navigation`
- `camera_sync`
- `replay_cursor`
- `temporal_range`
- world/minimap panel participation rules
- bottom-deck command-surface participation rules

## 3. `37-shell-mode-support-matrix.md` does not yet absorb 41

`37-shell-mode-support-matrix.md` still lacks:

- `battle_command`
- the new views introduced by `41`
- the new panels introduced by `41`
- the new layout traits introduced by `41`
- a judgeable shell-mode definition for the world-primary mode

Without those updates, `41` exists directionally, but not yet as a fully integrated shell-support contract.

## 4. Broker-map requirements are still not fully published

`40-shell-and-linked-interaction-stress-test.md` called for per-shell-mode broker maps and handoff rules.

`41` improves broker semantics, epochs, and recovery language, but it does not yet publish concrete per-shell-mode broker ownership maps for `battle_command`, `control_center`, or `live_meeting`.

## 5. Filtering minimums still look over-generalized

`40` identified that RTS-style world-primary operation is primarily selection/focus/navigation/command-targeting, not filter-first.

`39` still requires every linked shell mode to define a broker for `filtering`, and `41` does not explicitly reconcile that for world-primary shells.

## 6. Approval vs outcome semantics need one clarification

`20-intent-and-mediated-read-model.md` treats approval/denial as first-class external write families.

`41` can be read as approval surfaces directly moving an intent into `approved` or `rejected`.

That should be clarified so the design stays explicit about:

- operator approval/denial as mediated intent input
- `approved` / `rejected` as observed outcome states on the target flow

## Recommendation

Do **not** start `writing-plans` yet.

Instead, do one narrow reconciliation pass to integrate `41` back into the shell-mode and linked-interaction base specs.

## Recommended Next Doc Changes

1. Update `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`
   - add new world/temporal primitives and state slices
   - generalize actionable command-surface rules beyond `CommandDeckPanel`
   - define participation rules for the new world-primary panels
   - make filtering brokerage conditional where shell semantics justify it

2. Update `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`
   - add `battle_command`
   - add the new views, panels, and layout traits
   - fully populate required linked interactions, mediated reads, intents, degradation, and unsupported conditions

3. Clarify lifecycle wording in `docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md` and optionally mirror it in `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`

4. After the reconciliation pass, update `docs/superpowers/specs/modular-toolkit/40-shell-and-linked-interaction-stress-test.md` to mark the original gap as resolved or superseded by `41`

## Decision Gate

The current user-review decision is:

- **Option A:** treat `41` as accepted in substance, and do a targeted reconciliation patch to `37`, `39`, `20`, and `40`
- **Option B:** require one more explicit integration tranche before any surrounding spec edits

Recommended: **Option A**.
