# Superpowers Current State

**Date:** 2026-04-01
**Purpose:** Answer what is active, what is superseded, and what to read next

## Active Direction

The active architecture direction is toolkit-first.

- `deer_gui` is not the architectural center
- reusable modules and proof apps come before first playable composition
- chat/artifacts/replay/world state are part of one layered design

## Workstream Authority Map

| Workstream           | Decision Status          | Implementation Status   | Canonical Docs                                                                                                                                                                           | Notes                                                                         |
| -------------------- | ------------------------ | ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `modular-toolkit`    | draft revision, active   | review pending          | `docs/superpowers/specs/modular-toolkit/00-index.md`, `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`, `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`, `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md` | final integration tranche and surrounding base specs have been reconciled; acceptance is the remaining planning gate |
| `hybrid-data-game`   | draft input, active      | not started             | `docs/superpowers/specs/2026-03-31-hybrid-data-game-design.md`                                                                                                                           | product and game framing input for toolkit composition                        |
| `scifi-rts-hud`      | active visual reference  | not started             | `docs/superpowers/specs/scifi-rts-hud/2026-03-30-scifi-rts-hud-design.md`                                                                                                                | HUD look-and-feel reference                                                   |
| `multi-scene-themes` | approved, narrower scope | implementation-specific | `docs/superpowers/specs/multi-scene-themes/00-index.md`                                                                                                                                  | valid `deer_gui` scene/theme design, but not the newer top-level architecture |

## In Progress

- The modular-toolkit spec line has been reconciled and is awaiting acceptance.
- `writing-plans` has not started yet.
- The modular-toolkit spec set has been stress-tested for:
  - toolkit-first milestones
  - chat and artifact pipeline support
  - chat-to-world bridging
  - anti-drift planning guardrails
  - shell-mode support from operating modes through panels, layouts, and views
  - generator support against shell-mode requirements and linked dashboard behavior
  - linked interaction as a dedicated shell-level contract rather than an implicit
    view behavior
  - shell and linked-interaction stress testing against RTS HUD and state-server
    edge cases
  - final integration of world-primary shells, temporal/failover semantics,
    policy overlays, and intent boundaries

## Most Recent Decision Direction

- Shell support is now being pinned in `shell mode` terms first.
- The current support stack is:
  - `backend -> canonical mappings -> view support -> panel support -> layout support -> shell mode support`
- The current backend finding is that no studied generator yet proves any
  canonical shell mode under the strict shell contract.
- The linked-interaction contract is now being pinned as its own spec layer.
- The final integration tranche and surrounding base specs now reconcile the
  previously identified world-primary, temporal/failover, policy, and intent-
  boundary gaps.
- The remaining decision is whether to accept the reconciled spec line as
  planning-ready.
- If accepted, the next step is to start `writing-plans` from the reconciled
  modular-toolkit spec set.

## Recommended Reading Order

1. `docs/superpowers/specs/modular-toolkit/00-index.md`
2. `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`
3. `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`
4. `docs/superpowers/specs/modular-toolkit/06-milestones-to-first-playable.md`
5. `docs/superpowers/specs/modular-toolkit/07-chat-and-artifact-pipeline.md`
6. `docs/superpowers/specs/modular-toolkit/08-chat-to-world-bridging.md`
7. `docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md`
8. `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`
9. `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md`
10. `docs/superpowers/specs/modular-toolkit/40-shell-and-linked-interaction-stress-test.md`
11. `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`
12. `docs/superpowers/specs/2026-03-31-hybrid-data-game-design.md`
13. `docs/superpowers/specs/scifi-rts-hud/2026-03-30-scifi-rts-hud-design.md`
14. `docs/superpowers/specs/multi-scene-themes/00-index.md`
