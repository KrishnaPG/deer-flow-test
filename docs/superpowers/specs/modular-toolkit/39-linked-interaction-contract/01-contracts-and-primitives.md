## Why This File Exists

The current spec line now explicitly rejects view silos.

That creates one more missing layer between isolated view support and actual shell
support: a canonical linked-interaction contract.

This file defines the shared interaction layer that lets co-present views,
panels, and shell modes behave as one dashboard.

## Pinned Rule

- linked interaction is shell-level behavior, not a private feature of one view
- linked interaction state is local, shared, and non-canonical
- canonical records remain immutable storage-backed truth
- linked interaction must use declared canonical references, join keys, and
  mediated reads
- side-by-side hosted views do not count as linked unless they participate in the
  same declared interaction contract
- navigational world gestures and camera movement are not command-target changes
- linked state must remain deterministic across broker failover and policy
  invalidation
- `filtering` is required only for shell modes that declare shared filter
  semantics
- `prefill_seed` is broader than `action_intent_emission`; `command_target` and
  explicitly allowed linked-gesture paths may seed or refresh shell-local
  `prefill` without becoming editable or submittable intent state

## Linked Interaction Contract

Every linked interaction contract must define:

- participating views and panels
- required interaction paths
- canonical anchor record families for each path
- required stable join keys and filter dimensions
- required metadata at the point of use
- broker responsibilities
- broker owner by interaction type
- state durability and rehydration classification
- temporal/failover semantics, where relevant
- policy invalidation behavior, where relevant
- command-surface lifecycle entry boundaries, where the shell mode is
  actionable
- degradation behavior
- unsupported conditions

## Canonical Linked-Interaction Primitives

