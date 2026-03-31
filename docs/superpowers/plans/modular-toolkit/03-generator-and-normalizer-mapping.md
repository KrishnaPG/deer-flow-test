# Generator And Normalizer Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define how supported generators map into the canonical contracts and how storage-native raw records become normalized canonical records.

**Architecture:** This stage produces mapping specs and normalization rules, not runtime code. The goal is to eliminate ambiguity about what each generator can populate richly or sparsely, how L0/L1 records promote into L2/L3/L4/L5 objects, and what open-source/runtime boundary is reused at each stage.

**Tech Stack:** Markdown specs, DeerFlow/PocketFlow/Rowboat/Hermes code and docs, existing repo bridge/runtime references.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `docs/superpowers/specs/modular-toolkit/18-generator-mapping-matrix.md` | finalize high-level generator fit |
| Create | `docs/superpowers/specs/modular-toolkit/25-generator-capability-matrix.md` | rich/sparse support per generator and record family |
| Create | `docs/superpowers/specs/modular-toolkit/26-normalizer-promotion-rules.md` | L0/L1 -> L2/L3/L4/L5 promotion rules |
| Create | `docs/superpowers/specs/modular-toolkit/27-raw-envelope-family-catalog.md` | generator-agnostic raw envelope families |

## Task 1: Freeze Raw Envelope Families

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/27-raw-envelope-family-catalog.md`

- [ ] **Step 1: Write the envelope family catalog**

```md
# Design: Modular Toolkit - Raw Envelope Family Catalog

## Raw Families

- `SourceObjectEnvelope`
- `StreamDeltaEnvelope`
- `SnapshotEnvelope`
- `ArtifactEnvelope`
- `LineageEventEnvelope`
- `ProgressEnvelope`
- `RuntimeStatusEnvelope`
- `IntentEnvelope`
- `RetrievalHitEnvelope`

## Rule

Generators map into these envelope families before canonical normalization.
```

- [ ] **Step 2: Add generator capability matrix**

Create `docs/superpowers/specs/modular-toolkit/25-generator-capability-matrix.md` with:

```md
# Design: Modular Toolkit - Generator Capability Matrix

| Generator | Runs/Sessions | Tasks | Messages | Tools | Artifacts | Graph | Knowledge | Delivery | Runtime Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DeerFlow | rich | rich | rich | rich | rich | medium | low | low | medium |
| PocketFlow | medium | rich | low | medium | low | rich | low | low | medium |
| Rowboat | rich | rich | rich | rich | rich | rich | rich | medium | rich |
| Hermes | rich | rich | rich | rich | medium | low | low | rich | rich |
```

## Task 2: Freeze Promotion And Normalizer Rules

**Files:**
- Create: `docs/superpowers/specs/modular-toolkit/26-normalizer-promotion-rules.md`

- [ ] **Step 1: Write the promotion rule doc**

```md
# Design: Modular Toolkit - Normalizer Promotion Rules

## Promotion Path

- L0 source capture -> L1 sanitation
- L1 sanitation -> L2 canonical/view shaping
- L2 view shaping -> L3 insights
- L2/L3 -> L4 predictions
- L4 -> L5 prescriptions
- later observed outcomes append new L0-L3 records linked back to prior L4/L5 chains

## Rule

Normalizers do not invent arbitrary future-state semantics.
They promote records according to explicit rules and declared producers.
```

- [ ] **Step 2: Verify mapping and promotion docs exist together**

Run: `ls docs/superpowers/specs/modular-toolkit/25-generator-capability-matrix.md docs/superpowers/specs/modular-toolkit/26-normalizer-promotion-rules.md docs/superpowers/specs/modular-toolkit/27-raw-envelope-family-catalog.md`

Expected: PASS

## Success Checks

- generator-specific assumptions are isolated to mapping docs
- normalization now includes level promotion, not just field translation
- DeerFlow is no longer treated as the only generator worldview

## Done State

This file is done only when raw envelope families, generator capability matrix, and promotion rules are documented and linked from the readiness checklist.
