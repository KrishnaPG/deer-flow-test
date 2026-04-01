# Root Doc Housekeeping Plan

**Date:** 2026-04-01
**Scope:** `docs/superpowers/00-index.md`, `docs/superpowers/01-current-state.md`, `docs/superpowers/04-status-board.md`

## Current State

The root docs still correctly show modular-toolkit as pre-planning, but `docs/superpowers/01-current-state.md` is slightly stale because it says the final integration tranche is still being drafted even though `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md` now exists.

## Update Now

Update `docs/superpowers/01-current-state.md` to reflect that:

- the tranche has been drafted
- it is awaiting review
- the next decision is whether planning can start

Optional: tighten wording in `docs/superpowers/04-status-board.md` from generic drafted language to explicit review-pending language.

## Wait Until After Approval

Do not flip root docs to planning-ready until the tranche review decision is made.

That means these changes should wait:

- changing modular-toolkit from `pre-planning` to `planning-ready`
- changing the next step from `review the tranche` to `start writing-plans`
- changing the root index role text to planning-ready wording

## Minimal Pre-Approval Wording

### `docs/superpowers/01-current-state.md`

Suggested row update:

`| \`modular-toolkit\` | draft revision, review pending | pre-planning | \`docs/superpowers/specs/modular-toolkit/00-index.md\`, \`docs/superpowers/specs/modular-toolkit/09-non-negotiables.md\`, \`docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md\`, \`docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md\` | this is the main architecture line to follow; tranche approval is the remaining planning gate |`

Suggested bullet updates:

- `- The final integration tranche has now been drafted in \`docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md\` and is awaiting review.`
- `- The next decision after review is whether the spec line is finally ready for \`writing-plans\`.`

### `docs/superpowers/04-status-board.md`

Suggested row update:

`| \`modular-toolkit\` | design refined, tranche review pending | review \`41-final-integration-tranche.md\`, then decide whether planning can start | \`docs/superpowers/specs/modular-toolkit/00-index.md\` |`

Suggested priority list:

1. `review docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md for planning readiness`
2. `finish root-doc housekeeping after the review decision`
3. `start writing-plans for modular-toolkit only if the tranche is approved`

## Minimal Post-Approval Wording

If the tranche is accepted after reconciliation, then update:

### `docs/superpowers/01-current-state.md`

- modular-toolkit decision status -> `approved for planning, active`
- implementation status -> `planning-ready`
- note -> `start writing-plans from this spec set`

### `docs/superpowers/04-status-board.md`

- stage -> `planning-ready`
- next step -> `start writing-plans for modular-toolkit`

### `docs/superpowers/00-index.md`

Optional row wording:

`| \`modular-toolkit\` | current architecture direction; planning-ready | \`docs/superpowers/specs/modular-toolkit/00-index.md\` |`

## Recommendation

Make the small pre-approval current-state cleanup now, but do not update the root docs to planning-ready until the reconciliation work around `41` is complete and accepted.
