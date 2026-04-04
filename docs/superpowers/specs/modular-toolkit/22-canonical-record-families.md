# Design: Modular Toolkit - A/B Storage Row Families

**Date:** 2026-03-31
**Status:** Approved

## Mandatory Carrier/Orchestration Row Families

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

## Profile-Driven Semantic Row Families

- `L0_SourceRecord`
- `L1_SanitizedRecord`
- `L2_ViewRecord`
- `L3_InsightRecord`
- `L4_PredictionRecord`
- `L5_PrescriptionRecord`
- `L6_OutcomeRecord`

## Profile-Driven Representation / Index Row Families

- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

## Profile-Driven Governance / Operational Row Families

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

## Structural / Knowledge Row Families

- `GraphNodeRecord`
- `GraphEdgeRecord`
- `KnowledgeEntityRecord`
- `KnowledgeRelationRecord`

## Rule

Each row family must later define:

- identity fields
- allowed level/plane occupancy
- required lineage fields
- supported downstream `C:L2` view contracts and world-projection consumers
- whether it belongs to the mandatory carrier/orchestration module, the
  profile-driven level-prefixed semantic extension module, the
  profile-driven representation/index module, the profile-driven
  governance/operational module, or a structural/knowledge family
