# Modular Toolkit Implementation Plan Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first production-grade modular toolkit slices, prove them in narrow tool apps, and compose the first playable `deer_gui` experience without violating the toolkit-first architecture.

**Architecture:** Implementation proceeds in strict layers: `raw -> normalizer -> canonical domain -> derivation/read model/world projection -> reusable view/panel -> proof app -> deer_gui composition`. Reusable crates stay app-agnostic; `deer_gui` is a thin composition target. Tool apps, fixtures, and integration tests are mandatory gates before gameplay composition.

**Tech Stack:** Rust, Bevy 0.18.1, `bevy_egui`, `bevy_hanabi`, `bevy_tweening`, `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`, `crossbeam-channel`, `tracing`, `camino`, `smallvec`, `bytes`, `petgraph`, `egui_dock`, existing DeerFlow Python client + existing bridge transport.

---

## Non-Negotiable Inputs

Read before implementation:

- `docs/superpowers/01-current-state.md`
- `docs/superpowers/02-rules-in-force.md`
- `docs/superpowers/specs/modular-toolkit/09-non-negotiables.md`
- `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`

## Plan Files

| File                                 | Purpose                                                                             | Depends On         | Parallel Lane |
| ------------------------------------ | ----------------------------------------------------------------------------------- | ------------------ | ------------- |
| `01-workspace-and-foundation.md`     | workspace split, shared deps, contracts/domain/replay/read-model skeleton           | —                  | sequential    |
| `02-chat-pipeline-core.md`           | DeerFlow stream adapters, normalization, canonical chat/artifact state, derivations | 01                 | lane A        |
| `03-chat-lab.md`                     | reusable chat/artifact views + `apps/deer_chat_lab` proof app                       | 02                 | lane A        |
| `04-replay-toolkit.md`               | replay/timeline reusable slices + `apps/deer_replay`                                | 01                 | lane B        |
| `05-graph-toolkit.md`                | graph reusable slices + `apps/deer_graph_lab`                                       | 01                 | lane C        |
| `06-layout-and-design.md`            | panel shells, layout runtime, `apps/deer_design`                                    | 03, 04, 05         | lane D        |
| `07-world-projection-and-spatial.md` | world projection, actor cloud, telemetry, spatial preview                           | 02, 03, 06         | lane E        |
| `08-deer-gui-composition.md`         | thin `deer_gui` composition, playable loop, hardening gates                         | 03, 04, 05, 06, 07 | final         |

## Practical Success Definition

The plan succeeds only when all of the following are true:

- reusable crates own the core logic; `deer_gui` owns mostly composition
- every major slice has a fixture-backed proof app before `deer_gui` uses it
- all public reusable APIs consume canonical records, derived VMs, or read-model state only
- chat preserves tool calls, clarification, progress, and artifact lifecycle
- world projection is the only place RTS/RPG semantics enter the runtime
- tests are real integration tests or fixture-backed crate tests; no mocks
- files stay under 400 LOC and functions under 50 LOC by design
- battle-tested open-source libraries are preferred over custom infrastructure

## Definition Of Done

Do not mark a slice done unless it has:

- clear owning crate(s)
- typed contracts and fixtures
- passing automated tests
- tracing/logging in public execution paths
- a proof app scenario that can be rerun deterministically
- no app-specific or transport-specific leakage above the allowed layer

## Library Policy

Use existing, battle-tested libraries first:

- use DeerFlow's Python client and current bridge transport before inventing a new Rust protocol client
- use `petgraph` for graph structures and traversal helpers before rolling a graph core
- use `egui_dock` for docking/layout primitives before custom docking
- use existing Bevy plugins already in the repo before new rendering infrastructure

Only replace or supplement a library when:

- a required capability is missing
- performance evidence shows it is necessary
- the replacement is documented in the relevant plan task before code is written

## Global Verification

After each plan file is completed, run the narrow verification first, then the repo-wide check:

```bash
cargo test -p deer-gui --test integration -- --nocapture
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Expected result:

- all existing `apps/deer_gui/tests/integration/*` tests stay green
- new crate/app tests pass
- no warnings remain

## Drift Prevention Checklist

Before starting any task, answer:

1. Which layer owns this?
2. Which existing library or framework can solve most of it?
3. What is the failing test or fixture first?
4. Which proof app validates it?
5. Why is this not debuting inside `deer_gui`?

If any answer is unclear, stop and refine the task before coding.
