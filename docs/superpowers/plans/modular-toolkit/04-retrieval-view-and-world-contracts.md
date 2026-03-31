# Retrieval, View, And World Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define retrieval/index contracts, view taxonomy, detail levels, and world-projection contracts so later runtime crates know exactly what inputs/outputs they implement.

**Architecture:** This stage finalizes the contract surface for reusable views and world semantics. It distinguishes source/view/insight/prediction/prescription objects, defines thumbnail/tooltip/panel/world tiers, and connects them to storage planes, mediated retrieval, and trajectory-aware world projection.

**Tech Stack:** Markdown specs, `petgraph`/`egui_dock`/Bevy decision inputs, retrieval and view taxonomy specs already added to modular-toolkit.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `docs/superpowers/specs/modular-toolkit/15-discovery-object-taxonomy.md` | finalize object-family distinctions |
| Modify | `docs/superpowers/specs/modular-toolkit/16-retrieval-and-indexing-contract.md` | finalize retrieval/index contracts |
| Modify | `docs/superpowers/specs/modular-toolkit/17-view-taxonomy-and-lod.md` | finalize LOD/context rules |
| Create | `docs/superpowers/specs/modular-toolkit/28-entity-view-matrix.md` | entity-by-entity representation matrix |
| Create | `docs/superpowers/specs/modular-toolkit/29-world-projection-object-contract.md` | world object families and source-link rules |

## Task 1: Freeze The Entity View Matrix

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/28-entity-view-matrix.md`

- [ ] **Step 1: Write the entity view matrix**

```md
# Design: Modular Toolkit - Entity View Matrix

| Entity | Thumbnail Fields | Tooltip Fields | Panel Fields | World Fields |
| --- | --- | --- | --- | --- |
| agent | identity, status | role, task, health | assignments, history, outputs | optional unit/beacon fields |
| session | title, state | last turn, counts | transcript, composer, provenance | anchor/beacon fields |
| artifact | type, status | preview, producer | full metadata, provenance, actions | optional unlock/marker fields |
| task | state, severity | owner, blocker | subtasks, timings, dependencies | beacon/hotspot fields |
| replay event | type, time | summary, cause | payload, before/after, links | optional pulse/ghost fields |
```

- [ ] **Step 2: Freeze world object families**

Create `docs/superpowers/specs/modular-toolkit/29-world-projection-object-contract.md` with:

```md
# Design: Modular Toolkit - World Projection Object Contract

## World Object Families

- `WorldConversationAnchor`
- `WorldTaskBeacon`
- `WorldArtifactUnlock`
- `WorldInsightOverlay`
- `WorldPredictionOverlay`
- `WorldPrescriptionPrompt`
- `WorldOutcomeMarker`

## Rule

Every world object must retain backlinks to:

- source record IDs
- level/plane metadata
- discovery object kind
- drill-down panel targets
```

## Task 2: Freeze Retrieval And LOD Rules As Blocking Inputs

**Files:**
- Modify: `docs/superpowers/specs/modular-toolkit/16-retrieval-and-indexing-contract.md`
- Modify: `docs/superpowers/specs/modular-toolkit/17-view-taxonomy-and-lod.md`

- [ ] **Step 1: Add explicit readiness bullets to retrieval/indexing contract**

Append:

```md
## Readiness Gate

Before implementation, retrieval-aware crates must know:

- which payloads are full As-Is objects
- which payloads are chunk-derived
- which views are embedding-assisted retrieval only
- which fields must be surfaced to users to avoid provenance loss
```

- [ ] **Step 2: Add explicit readiness bullets to view taxonomy**

Append:

```md
## Readiness Gate

Before implementation, each entity/view pair must have:

- required fields
- commands/events
- source/backlink metadata
- allowed contexts
- whether world representation is permitted
```

- [ ] **Step 3: Verify the view/world contract docs exist**

Run: `ls docs/superpowers/specs/modular-toolkit/28-entity-view-matrix.md docs/superpowers/specs/modular-toolkit/29-world-projection-object-contract.md`

Expected: PASS

## Success Checks

- implementation teams can now answer what every major entity looks like at each view tier
- world projection is grounded in source-linked object families, not ad hoc markers
- retrieval/indexing semantics are visible to later runtime crates

## Done State

This file is done only when entity view matrix and world projection object contracts exist and the retrieval/LOD docs contain explicit readiness gates.
