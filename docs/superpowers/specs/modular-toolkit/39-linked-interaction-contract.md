# Design: Modular Toolkit - Linked Interaction Contract

**Date:** 2026-04-01
**Status:** Accepted for planning


- [Contracts and Primitives](./39-linked-interaction-contract/01-contracts-and-primitives.md)
- [Selection to Focus](./39-linked-interaction-contract/02-selection-to-focus.md)
- [Pinning to Compare](./39-linked-interaction-contract/03-pinning-to-compare.md)
- [Action to Command](./39-linked-interaction-contract/04-action-to-command.md)
- [Viewport to Temporal](./39-linked-interaction-contract/05-viewport-to-temporal.md)
- [Shared Shell State](./39-linked-interaction-contract/06-shared-shell-state.md)
- [Panel Participation](./39-linked-interaction-contract/07-panel-participation.md)
- [Shell Mode Minimums](./39-linked-interaction-contract/08-shell-mode-minimums.md)
- [Joinability Metadata](./39-linked-interaction-contract/09-joinability-metadata.md)
- [Support Judgment](./39-linked-interaction-contract/10-support-judgment.md)

## Reconciled Linked-Interaction Boundary

Linked interactions may widen selection, replay, world, or comparison context
into command entry only as shell-local `prefill_seed`.

They do not directly assert editable `prefill`, operator-owned `draft`,
`validated`, or canonical `submitted`; those promotions occur only within the
receiving shell lifecycle defined in `20-intent-and-mediated-read-model.md`.
