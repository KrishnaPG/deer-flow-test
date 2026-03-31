# Data Ontology And Canonical Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the generator-agnostic canonical ontology, storage-aware metadata envelopes, and normalized record families before any foundation crate code is written.

**Architecture:** This stage designs the schema, not the implementation. It captures L0-L5 levels, tri-plane storage, lineage metadata, correlation fields, discovery object kinds, and generator-agnostic record families so foundation crates can later be typed directly from approved specs.

**Tech Stack:** Markdown specs, `docs/architecture/state-server.md`, generator research from DeerFlow, PocketFlow, Rowboat, and Hermes.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `docs/superpowers/specs/modular-toolkit/12-levels-and-planes-contract.md` | finalize level/plane field requirements |
| Modify | `docs/superpowers/specs/modular-toolkit/13-lineage-and-immutability-contract.md` | finalize lineage metadata envelopes |
| Create | `docs/superpowers/specs/modular-toolkit/22-canonical-record-families.md` | define generator-agnostic record families |
| Create | `docs/superpowers/specs/modular-toolkit/23-correlation-and-identity-contract.md` | define IDs, correlation IDs, and record identity rules |
| Create | `docs/superpowers/specs/modular-toolkit/24-level-plane-lineage-matrix.md` | matrix of which record families can inhabit which level/plane |

## Task 1: Define Canonical Record Families

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/22-canonical-record-families.md`

- [ ] **Step 1: Write the canonical family inventory**

```md
# Design: Modular Toolkit - Canonical Record Families

## Core Families

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `InsightRecord`
- `PredictionRecord`
- `PrescriptionRecord`
- `OutcomeRecord`
- `GraphNodeRecord`
- `GraphEdgeRecord`
- `KnowledgeEntityRecord`
- `KnowledgeRelationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

## Rule

Each family must later define:

- identity fields
- level/plane occupancy
- required lineage fields
- downstream consumers
```

- [ ] **Step 2: Verify the inventory exists and is referenced by the readiness checklist**

Add to `21-implementation-readiness-checklist.md`:

```md
- canonical record families approved in `docs/superpowers/specs/modular-toolkit/22-canonical-record-families.md`
```

## Task 2: Define Correlation And Identity Rules

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/23-correlation-and-identity-contract.md`

- [ ] **Step 1: Write the correlation contract**

```md
# Design: Modular Toolkit - Correlation And Identity Contract

## Required Correlation Fields

- `mission_id`
- `run_id`
- `agent_id`
- `artifact_id`
- `trace_id`

## Additional Common IDs

- `thread_id`
- `task_id`
- `event_id`
- `chunk_hash`
- `as_is_hash`

## Rule

Every major record family must declare:

- primary identity
- parent/source identity references
- correlation fields it requires
```

- [ ] **Step 2: Write the level/plane/lineage occupancy matrix**

Create `docs/superpowers/specs/modular-toolkit/24-level-plane-lineage-matrix.md` with:

```md
# Design: Modular Toolkit - Level Plane Lineage Matrix

| Record Family | L0 | L1 | L2 | L3 | L4 | L5 | As-Is | Chunks | Embeddings |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ArtifactRecord` | yes | yes | yes | optional | optional | optional | yes | optional | optional |
| `InsightRecord` | no | no | optional | yes | no | no | optional | optional | optional |
| `PredictionRecord` | no | no | optional | optional | yes | no | optional | optional | optional |
| `PrescriptionRecord` | no | no | optional | optional | optional | yes | optional | optional | optional |
```

- [ ] **Step 3: Run the file existence check**

Run: `ls docs/superpowers/specs/modular-toolkit/22-canonical-record-families.md docs/superpowers/specs/modular-toolkit/23-correlation-and-identity-contract.md docs/superpowers/specs/modular-toolkit/24-level-plane-lineage-matrix.md`

Expected: PASS

## Success Checks

- canonical ontology exists independently of DeerFlow specifics
- identity/correlation rules align with the state-server correlation contract
- record-family occupancy across levels/planes is now explicit

## Done State

This file is done only when canonical families, identity rules, and occupancy matrix are documented and linked from the readiness checklist.
