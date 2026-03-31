# Design: Modular Toolkit - Canonical Record Families

**Date:** 2026-03-31
**Status:** Draft revision

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
- whether it is observed, inferred, predicted, prescribed, or outcome-bearing
