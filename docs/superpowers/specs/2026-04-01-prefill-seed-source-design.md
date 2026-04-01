# Prefill Seed Source Design

**Date:** 2026-04-01
**Status:** Draft revision
**Scope:** Resolve the `prefill` entry-rule inconsistency across `39-linked-interaction-contract.md`, `41-final-integration-tranche.md`, and shell modes that allow world-primary or replay-linked command seeding.

## Why This Exists

The current spec line contains an internal contradiction:

- `41-final-integration-tranche.md` says `prefill` may be entered by any
  declared `action_intent_emission` source
- the same file also says linked gestures, replay scrubbing, world picks,
  camera moves, and attention interrupts may create or refresh `prefill`
- `39-linked-interaction-contract.md` does not treat all of those as
  `action_intent_emission` sources
- `39-linked-interaction-contract.md` does treat some of them as
  `command_target` sources instead

That means the lifecycle table is narrower than the rest of the shell model.

## Decision

Widen `prefill` entry to a broader **seed source** concept.

`prefill` may be entered or refreshed by any declared `prefill_seed` source.

`prefill_seed` is a shell-level category, not a new canonical record family.
It is satisfied by one of:

- any declared `command_target` source
- any declared `action_intent_emission` source
- any explicitly allowed linked-gesture source declared by the shell mode

## Prefill Seed Rules

### Allowed Effect

`prefill_seed` may only:

- create shell-local `prefill`
- refresh existing `prefill`
- update suggested target, scope, parameter, or provenance context

It may not:

- create `draft` automatically
- enter `validated`
- enter `submitted`
- append any canonical `IntentRecord`
- imply approval, execution, or side effects

### Required Payload Semantics

Every `prefill_seed` must preserve enough context to explain where the seed came
from.

Required basis:

- seed origin kind
- provenance source
- suggested target refs, if any
- suggested scope, if any
- policy visibility at the time of seeding
- broker epoch or interaction sequence where relevant

### Source Classes

#### 1. `action_intent_emission` sources

These remain valid `prefill` sources.

They are the strongest direct action-oriented seed path because they already
carry intent-facing semantics.

#### 2. `command_target` sources

These are also valid `prefill` sources.

This covers sources such as:

- selection-capable views
- world picks
- replay-driven target suggestions
- command-surface retargeting

`command_target` may therefore seed intent context even when the originating
interaction is not itself a declared `action_intent_emission` source.

#### 3. Explicit linked-gesture seed sources

Shell modes may also declare linked gestures that are allowed to seed or refresh
`prefill` even when they are neither `action_intent_emission` nor
`command_target` sources.

Examples may include:

- replay scrubbing to a candidate intervention point
- attention interrupts surfacing a candidate intervention context
- explicit locate/follow gestures that suggest a command context

Plain navigation-only gestures must remain intent-neutral unless the shell mode
explicitly declares them as seed-capable.

## Camera-Movement Rule

Ordinary camera pan, zoom, or viewport repositioning should not create
`prefill` by default.

Only explicit camera-linked gestures that the shell mode declares as
seed-capable may create or refresh `prefill`.

This keeps navigation distinct from action seeding.

## Required Spec Changes

### `41-final-integration-tranche.md`

Update the lifecycle table so `prefill` may be entered by any declared
`prefill_seed` source rather than only by `action_intent_emission` sources.

Update the accidental-execution guardrails so linked gestures remain allowed to
create or refresh `prefill`, but only when they are declared seed-capable.

### `39-linked-interaction-contract.md`

Add a short shell-level rule that:

- `command_target` sources may seed or refresh `prefill`
- declared linked gestures may seed or refresh `prefill`
- none of those paths may advance beyond `prefill` without explicit operator
  promotion

### `37-shell-mode-support-matrix.md`

Where shell modes rely on world-primary or replay-linked intent seeding, they
should declare the allowed seed-capable interaction paths explicitly when those
paths are stricter than the inherited defaults.

## Why This Is Preferred

This keeps the current intent model intact:

- `prefill` stays cheap and shell-local
- explicit operator action is still required for `draft`, `validated`, and
  `submitted`
- world-primary and replay-driven shells keep their intended fast seeding
  behavior
- the contract now matches the broader shell model instead of silently banning
  allowed sources

## Non-Goals

- no new canonical persistence stage
- no silent execution path
- no requirement that every linked gesture become seed-capable
- no requirement that every `viewport_navigation` or camera update imply command
  intent
