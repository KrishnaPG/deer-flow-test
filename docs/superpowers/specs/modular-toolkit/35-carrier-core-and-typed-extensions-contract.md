# Design: Modular Toolkit - Carrier Core And Typed Extensions Contract

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The ontology should be optimized for practical use.

That means engineers adding a new backend or building a new UI should not need to
 reason about the entire ontology at once.

## Practical Winner

The winning shape is:

- a carrier-first core for everyday work
- typed extensions for semantic, representation, and governance concerns when
  they add real value

## Carrier-First Core

The default integration and UI-facing core is:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

All core records must carry:

- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`

## Typed Semantic Extensions

Semantic meaning is attached through level-prefixed extension families:

- `L0_SourceRecord`
- `L1_SanitizedRecord`
- `L2_ViewRecord`
- `L3_InsightRecord`
- `L4_PredictionRecord`
- `L5_PrescriptionRecord`
- `L6_OutcomeRecord`

These are not the everyday integration surface for every backend.

They are promoted to first-class use where product, retrieval, replay, or world
projection needs them.

## Typed Representation Extensions

Representation concerns attach through:

- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

Representation policy:

- `AsIsRepresentationRecord` is broadly valid across `L0..L6`
- `ChunkRecord` is typical for `L1..L4`, optional for `L5..L6`, exceptional for
  `L0`
- `EmbeddingRecord` is always derived from `ChunkRecord`; it is typical for
  `L1..L4`, optional/policy-driven for `L5..L6`, and generally not default for
  `L0`

## Typed Governance And Operational Extensions

Governance and operational concerns attach through records such as:

- `IntentRecord`
- `TransformRecord`
- `WriteOperationRecord`
- `ExclusionRecord`
- `ConflictRecord`
- `ResolutionRecord`
- `ReplayCheckpointRecord`
- `DedupRecord`
- `BatchRecord`
- `BranchRecord`
- `VersionRecord`

Heavier runtime extensions may be added later where needed.

## Attachment Rule

Carrier/core identity is the primary anchor.

Typed extensions may either:

- carry their own IDs with mandatory backlinks to carrier IDs, or
- be derived views over carrier-linked truth where no standalone identity is
  needed

Backlinks are mandatory either way.

## Anti-Drift Rule

Do not force every backend to populate every extension family.

Backend mappings should populate:

- the carrier-first core by default
- semantic, representation, and governance extensions only where supported and
  valuable
