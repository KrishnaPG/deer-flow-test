# Functional Game Blockers Audit

**Date:** 2026-04-01
**Scope:** `docs/superpowers/01-current-state.md`, `docs/superpowers/04-status-board.md`, `docs/superpowers/specs/modular-toolkit/00-index.md`, `docs/superpowers/specs/modular-toolkit/06-milestones-to-first-playable.md`, `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`, `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`, `docs/superpowers/specs/modular-toolkit/41-final-integration-tranche.md`, and current `apps/deer_gui`

## 1) Top blocker

The top blocker is **missing implementation of the first required vertical slice**, not vague architecture indecision alone.

More specifically: the repo has committed to a toolkit-first path where `deer_gui` is only allowed to become playable **after** reusable modules and proof apps exist, but those crates/apps do not exist in the actual workspace yet.

That means there is currently no path in-repo to produce the required first-playable loop, because the mandated building blocks are still mostly specified rather than built.

## 2) Secondary blockers

### A. Planning is still formally gated by draft-only discovery docs

The docs explicitly say planning and implementation are blocked until the discovery tranche through `31-custom-code-boundaries.md` is approved, and the readiness checklist says implementation is blocked if any required source doc is still `Status: Draft revision`.

That is a real blocker, but it is secondary to missing implementation because even if the gate were lifted today, the needed toolkit/proof-app work still has not been started.

### B. No proof apps or reusable toolkit crates are present in the workspace

The milestone path depends on `foundation/contracts`, `pipeline/normalizers`, `runtime/read_models`, plus proof apps `apps/deer_chat_lab`, `apps/deer_replay`, `apps/deer_graph_lab`, and `apps/deer_design`.

In the live repo, the workspace only includes `apps/deer_gui`, and there is no `crates/` tree or those proof apps. So the required ladder to first playable is absent as code, not just unfinished as docs.

### C. `deer_gui` is shell/UI-heavy and does not contain the required closed gameplay loop

`deer_gui` has substantial Bevy shell work, but key game-loop behavior is still stubbed or non-composed:

- command dispatch is explicitly TODO in `apps/deer_gui/src/hud/state_systems.rs`
- world event handling mostly logs bridge events in `apps/deer_gui/src/world/systems.rs`
- the app talks to a chat bridge, but the first-playable loop from the milestone doc (research request -> live orchestration -> inspect artifacts/results -> select units/agents -> react to alert/failure -> drill into intervention -> return to macro state) is not implemented end to end

### D. The current app is not actually the architecture the docs say to finish

`docs/superpowers/01-current-state.md` says `deer_gui` is not the architectural center and that the active `modular-toolkit` line is still `pre-planning`.

So the existing `deer_gui` work may be useful shell/reference work, but by the repo's own active rules it is not yet the sanctioned path to a functional playable game.

## 3) Evidence from repo/docs

- `docs/superpowers/01-current-state.md`
  - says the active direction is toolkit-first
  - says `deer_gui` is not the architectural center
  - says `modular-toolkit` implementation status is `pre-planning`
  - says `writing-plans` has not started yet
- `docs/superpowers/04-status-board.md`
  - says current priority is reviewing the final integration tranche, then deciding whether planning can start
  - says to start `writing-plans` only if that tranche is approved
- `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`
  - says milestone gates are blocking, not suggestions
  - says `deer_gui` must not become the first complete version of core reusable slices
  - says do not write or execute crate-level implementation plans until the discovery-first tranche through `31-custom-code-boundaries.md` is approved
- `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`
  - says implementation is blocked if any required source doc is still `Status: Draft revision`
  - says no reusable feature may debut in `apps/deer_gui`
  - says scaffolding counts as implementation
- `docs/superpowers/specs/modular-toolkit/06-milestones-to-first-playable.md`
  - defines M0-M5 as required toolkit/proof milestones before M6 first playable composition in `apps/deer_gui`
  - names required crates/apps that are not present in the current workspace
- `Cargo.toml`
  - workspace members are only `apps/deer_gui`
- repo structure
  - no `crates/` directory for `foundation/contracts`, `pipeline/normalizers`, or `runtime/read_models`
  - no `apps/deer_chat_lab`, `apps/deer_replay`, `apps/deer_graph_lab`, or `apps/deer_design`
- `apps/deer_gui/src/hud/state_systems.rs`
  - `command_dispatch_system` still says `TODO: dispatch to OrchestratorClient or emit a Bevy event`
- `apps/deer_gui/src/world/systems.rs`
  - bridge events are mostly logged, not turned into a real intervention/macro-state loop

## 4) Minimum path to a functional game

The minimum path is **not** to keep elaborating `deer_gui` in place.

It is:

1. Accept or narrowly reconcile the remaining planning-readiness docs so the guardrails stop blocking plan creation.
2. Write the first implementation plans for the smallest possible reusable slice that supports the eventual closed loop.
3. Build only the minimum M0/M1/M4/M5 path needed for one real loop:
   - canonical foundation contracts
   - normalization/read-model path for live orchestration state
   - one proof app for chat/artifact flow
   - one proof app or narrow sandbox for layout/spatial projection
4. Compose `deer_gui` only after those slices are proven, with a drastically narrowed M6 target:
   - submit one request
   - observe live progress
   - inspect one artifact/result
   - select one world object/unit tied to that state
   - trigger one explicit intervention command that changes visible macro state
   - inspect replay/history of that change

So the concrete answer is: **the repo is blocked mainly by absence of the mandated reusable/proven vertical slice in code, with planning guardrails acting as an additional formal blocker.** The problem is less "architecture indecision" than "the chosen architecture forbids app-first playability, and the prerequisite toolkit path has barely been implemented."
