# Design: Modular Toolkit - Canonical Record Families

**Date:** 2026-03-31
**Status:** Draft revision

## Carrier-First Core Families

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

## Typed Semantic Extensions

- `L0_SourceRecord`
- `L1_SanitizedRecord`
- `L2_ViewRecord`
- `L3_InsightRecord`
- `L4_PredictionRecord`
- `L5_PrescriptionRecord`
- `L6_OutcomeRecord`

## Typed Representation / Index Extensions

- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

## Typed Governance / Operational Extensions

- `IntentRecord`
- `TransformRecord`
- `ExclusionRecord`
- `ConflictRecord`
- `ResolutionRecord`
- `ReplayCheckpointRecord`
- `DedupRecord`
- `BatchRecord`
- `BranchRecord`
- `VersionRecord`
- `WriteOperationRecord`

## Structural / Knowledge Families

- `GraphNodeRecord`
- `GraphEdgeRecord`
- `KnowledgeEntityRecord`
- `KnowledgeRelationRecord`

## Rule

Each family must later define:

- identity fields
- allowed level/plane occupancy
- required lineage fields
- downstream consumers
- whether it belongs to the carrier-first core, a typed semantic extension, a
  typed representation extension, a governance/operational extension, or a
  structural family
