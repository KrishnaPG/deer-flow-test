# Library And Build Order Freeze Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freeze the battle-tested library decisions, identify unavoidable custom-code zones, and produce the corrected post-discovery build order before runtime implementation resumes.

**Architecture:** This stage turns the discovery work into a precise build sequence. It explicitly records what will be reused, what remains custom, what performance/quality constraints apply, and what milestone gates must be satisfied before foundation crates and proof apps are planned again.

**Tech Stack:** Markdown specs/plans, existing repo dependencies, identified OSS choices (`petgraph`, `egui_dock`, existing DeerFlow bridge, Bevy stack).

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `docs/superpowers/specs/modular-toolkit/30-library-decision-matrix.md` | exact library choices by stage |
| Create | `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` | what remains custom and why |
| Create | `docs/superpowers/plans/modular-toolkit/06-post-discovery-build-order.md` | corrected milestone order after discovery |
| Modify | `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md` | link to discovery readiness prerequisites |

## Task 1: Freeze Library Decisions

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/30-library-decision-matrix.md`

- [ ] **Step 1: Write the library matrix**

```md
# Design: Modular Toolkit - Library Decision Matrix

| Stage | Preferred Library / Reuse | Why | Avoid Replacing Unless |
| --- | --- | --- | --- |
| transport/client | existing DeerFlow bridge + Python client | already integrated, low protocol risk | hard capability gap or measured performance issue |
| contracts/serde | `serde`, `serde_json`, `chrono`, `uuid` | mature, standard Rust fit | schema export needs force additions |
| graph core | `petgraph` | mature graph primitives | missing required graph capability |
| layout/docking | `egui_dock` | battle-tested egui docking | missing core docking behavior |
| 3D/spatial | existing Bevy stack | already in repo and aligned | measured inadequacy |
| snapshots | `insta`, `similar-asserts` | stable snapshot diff tooling | snapshot instability forces narrower scope |
```

- [ ] **Step 2: Freeze custom-code boundaries**

Create `docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md` with:

```md
# Design: Modular Toolkit - Custom Code Boundaries

## Must Be Custom

- state-server-aware normalizers
- canonical storage/trajectory-aware record types
- lineage-preserving world projection
- artifact provenance and retrieval adapters
- trajectory-aware replay semantics

## Must Not Be Custom First

- graph data structure core
- docking framework
- generic transport replacement for DeerFlow bridge
```

## Task 2: Freeze Corrected Post-Discovery Build Order

**Files:**
- Create: `docs/superpowers/plans/modular-toolkit/06-post-discovery-build-order.md`
- Modify: `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`

- [ ] **Step 1: Write the corrected build-order plan file**

```md
# Post-Discovery Build Order Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the implementation order that becomes valid only after discovery-first contracts are frozen.

**Architecture:** This file is a bridge from discovery to runtime implementation. It does not add new architecture; it sequences the approved slices so implementation remains toolkit-first, storage-aware, and generator-agnostic.

**Tech Stack:** Approved spec set, approved library decision matrix.

---

## Corrected Order

1. foundation contracts/domain metadata envelopes
2. raw envelope and normalizer crates
3. canonical record families and lineage-aware replay base
4. derivations/read-models with level/plane metadata
5. retrieval-aware and view-tier-aware reusable surfaces
6. proof apps for chat/replay/graph/design
7. world projection and spatial views
8. thin `deer_gui` composition
```

- [ ] **Step 2: Link discovery prerequisites into planning guardrails**

Append to `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`:

```md
## Discovery Prerequisite Rule

Do not write or execute crate-level implementation plans until the discovery-first
spec tranche through `31-custom-code-boundaries.md` is approved.
```

- [ ] **Step 3: Verify all freeze docs exist**

Run: `ls docs/superpowers/specs/modular-toolkit/30-library-decision-matrix.md docs/superpowers/specs/modular-toolkit/31-custom-code-boundaries.md docs/superpowers/plans/modular-toolkit/06-post-discovery-build-order.md`

Expected: PASS

## Success Checks

- the project now has a documented, discovery-first build order
- library choices are frozen before coding
- custom-code zones are explicit
- planning guardrails now block premature implementation

## Done State

This file is done only when library choices, custom-code boundaries, and the corrected post-discovery build order are documented and linked from planning guardrails.
