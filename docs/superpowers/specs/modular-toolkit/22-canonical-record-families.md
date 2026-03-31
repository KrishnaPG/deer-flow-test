# Design: Modular Toolkit - Canonical Record Families

**Date:** 2026-03-31
**Status:** Draft revision

## Mandatory Carrier/Orchestration Module Families

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

## Profile-Driven Semantic Module Families

- `L0_SourceRecord`
- `L1_SanitizedRecord`
- `L2_ViewRecord`
- `L3_InsightRecord`
- `L4_PredictionRecord`
- `L5_PrescriptionRecord`
- `L6_OutcomeRecord`

## Profile-Driven Representation / Index Module Families

- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

## Profile-Driven Governance / Operational Module Families

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
- whether it belongs to the mandatory carrier/orchestration module, the
  profile-driven semantic module, the profile-driven representation/index
  module, the profile-driven governance/operational module, or a structural
  family
