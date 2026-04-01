# Superpowers Decision Log

**Date:** 2026-03-31
**Purpose:** Short log of decisions that matter to future agents

| Date | Decision | Status | Source |
| --- | --- | --- | --- |
| 2026-03-31 | The active architecture is toolkit-first, not `deer_gui`-first | active | `docs/superpowers/specs/modular-toolkit/01-architecture-position.md` |
| 2026-03-31 | `deer_gui` should compose proven modules, not invent them first | active | `docs/superpowers/specs/modular-toolkit/04-authoring-tools-as-proving-grounds.md` |
| 2026-03-31 | TDD happens at reusable module boundaries and is proved through tool apps | active | `docs/superpowers/specs/modular-toolkit/05-tdd-by-contract.md` |
| 2026-03-31 | Live chat, uploads, artifacts, and clarification are first-class toolkit concerns | active | `docs/superpowers/specs/modular-toolkit/07-chat-and-artifact-pipeline.md` |
| 2026-03-31 | RTS/RPG meaning enters through `world_projection`, not reusable chat/view crates | active | `docs/superpowers/specs/modular-toolkit/08-chat-to-world-bridging.md` |
| 2026-03-31 | Modular-toolkit planning must follow explicit non-negotiables and milestone gates | active | `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`, `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md` |
| 2026-03-31 | The old monolithic modular-toolkit spec is superseded by the split spec set | superseded | `docs/superpowers/specs/2026-03-31-modular-toolkit-architecture-design.md` |
| 2026-03-31 | Product shape remains hybrid: RTS shell first, RPG puncture points second | active | `docs/superpowers/specs/2026-03-31-hybrid-data-game-design.md` |
| 2026-04-01 | Shell support should be judged from shell modes downward while keeping view contracts as the atomic backend support unit | active | `docs/superpowers/specs/modular-toolkit/37-shell-mode-support-matrix.md`, `docs/superpowers/specs/modular-toolkit/36-view-contract-support-model.md` |
| 2026-04-01 | No studied generator currently proves any canonical shell mode under the strict shell contract; linked shell composition must be owned at the toolkit layer | active | `docs/superpowers/specs/modular-toolkit/38-generator-shell-mode-support-evaluation.md` |
| 2026-04-01 | Linked dashboard behavior must be specified as a shell-level contract with explicit primitives, joinability rules, brokers, and shared non-canonical shell state | active | `docs/superpowers/specs/modular-toolkit/39-linked-interaction-contract.md` |
| 2026-04-01 | Stress testing shows the current shell and linked-interaction contracts are not yet planning-ready; a final integration tranche is required for world-primary shells, temporal/failover behavior, policy overlays, and intent lifecycle boundaries | active | `docs/superpowers/specs/modular-toolkit/40-shell-and-linked-interaction-stress-test.md` |
| 2026-04-01 | The final integration tranche should unify world-primary shells, temporal/failover semantics, policy overlays, and strict intent lifecycle boundaries before planning starts | active | `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md` |
