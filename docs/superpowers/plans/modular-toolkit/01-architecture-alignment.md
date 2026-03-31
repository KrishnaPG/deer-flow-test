# Architecture Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the modular-toolkit implementation path with the storage-native state-server architecture before any crate-level implementation begins.

**Architecture:** This tranche is discovery-first and contract-first. It does not begin by building runtime crates. It begins by reconciling toolkit layering with storage truth, mediated reads, intent writes, immutable lineage, and tri-plane storage so later implementation tasks have stable architectural boundaries.

**Tech Stack:** Markdown specs, existing `docs/architecture/state-server.md`, current modular-toolkit spec set, existing `deer_gui` bridge/runtime code as reference only.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `docs/superpowers/specs/modular-toolkit/11-state-server-alignment.md` | finalize storage-truth and boundary rules if clarifications emerge |
| Create | `docs/superpowers/specs/modular-toolkit/19-ui-state-server-boundary.md` | define exact UI/runtime interaction boundaries |
| Create | `docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md` | define read/write assumptions and payload families |
| Create | `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md` | freeze go/no-go conditions before crate implementation |

## Task 1: Write the failing alignment checklist first

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`

- [ ] **Step 1: Write the checklist document skeleton with failing conditions called out**

```md
# Design: Modular Toolkit - Implementation Readiness Checklist

## Must Be True Before Crate Implementation

- [ ] storage truth vs client truth is explicitly documented
- [ ] mediated read boundary is explicitly documented
- [ ] intent/write boundary is explicitly documented
- [ ] L0-L5 semantics are attached to canonical record design
- [ ] planes (`as_is`, `chunks`, `embeddings`) are attached to canonical record design
- [ ] immutable lineage fields are defined
- [ ] generator-agnostic mapping rules exist
- [ ] view taxonomy and LOD rules exist

## Failure Rule

If any item is incomplete, do not start foundation crate implementation.
```

- [ ] **Step 2: Review the checklist against current specs and confirm gaps exist**

Run: `grep -n "Implementation Readiness Checklist\|Must Be True Before Crate Implementation" docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`

Expected: PASS, and the team can manually confirm the checklist now governs implementation readiness.

## Task 2: Freeze UI / State-Server Boundary Rules

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/19-ui-state-server-boundary.md`

- [ ] **Step 1: Write the boundary spec with exact allowed/forbidden flows**

```md
# Design: Modular Toolkit - UI State Server Boundary

## Allowed External Read Paths

- UI reads live activity through state-server-mediated streams
- UI reads historical data through state-server-mediated warm-cache or DB-backed responses
- UI reads artifacts through authorized pointers or mediated previews

## Allowed External Write Paths

- UI submits intents only
- intents are policy-checked before becoming append-only records/events

## Forbidden Assumptions

- UI does not treat raw backend internal payloads as source truth
- UI does not assume direct storage paths are safe to open
- proof apps must simulate mediated reads, not bypass them conceptually
```

- [ ] **Step 2: Confirm the boundary doc is referenced from the readiness checklist**

Add to `21-implementation-readiness-checklist.md`:

```md
- Boundary source: `docs/superpowers/specs/modular-toolkit/19-ui-state-server-boundary.md`
```

## Task 3: Freeze Intent And Mediated-Read Payload Families

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md`

- [ ] **Step 1: Write the mediated payload family definitions**

```md
# Design: Modular Toolkit - Intent And Mediated Read Model

## External Read Families

- live-activity stream payload
- historical snapshot payload
- artifact preview payload
- artifact pointer payload
- replay window payload
- retrieval/search result payload

## External Write Families

- operator intent
- clarification response
- approval/denial intent
- intervention intent

## Required Metadata

- ABAC result or visibility scope
- correlation identifiers
- level and plane hints
- lineage backlinks
```

- [ ] **Step 2: Verify the three architecture-alignment docs form a complete gate**

Run: `ls docs/superpowers/specs/modular-toolkit/19-ui-state-server-boundary.md docs/superpowers/specs/modular-toolkit/20-intent-and-mediated-read-model.md docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`

Expected: PASS

## Success Checks

- implementation cannot start until storage-truth, mediated-read, and intent-write boundaries are frozen
- future crate plans will reference these docs as prerequisites

## Done State

This file is done only when the architecture readiness checklist and state-server boundary docs exist and are used as blocking gates.
