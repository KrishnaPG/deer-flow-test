# Design: Modular Toolkit - Canonical Record Families

**Date:** 2026-03-31
**Status:** Draft revision

## Semantic Spine Families

- `SourceRecord`
- `SanitizedRecord`
- `ViewRecord`
- `InsightRecord`
- `PredictionRecord`
- `PrescriptionRecord`
- `OutcomeRecord`

## Carrier / Orchestration Families

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`
- `ClarificationRecord`
- `RuntimeStatusRecord`
- `DeliveryRecord`

## Representation / Index Families

- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

## Governance / Operational Families

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
- whether it is semantic, carrier, representation, governance, or structural
