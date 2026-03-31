# Design: Modular Toolkit - Level Plane Lineage Matrix

**Date:** 2026-03-31
**Status:** Draft revision

## Occupancy Matrix

| Record Family | L0 | L1 | L2 | L3 | L4 | L5 | As-Is | Chunks | Embeddings |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `RunRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no |
| `SessionRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no |
| `TaskRecord` | optional | optional | yes | yes | optional | optional | optional | optional | no |
| `MessageRecord` | yes | yes | yes | optional | optional | optional | yes | optional | optional |
| `ToolCallRecord` | yes | yes | yes | optional | optional | optional | yes | optional | optional |
| `ArtifactRecord` | yes | yes | yes | optional | optional | optional | yes | optional | optional |
| `ClarificationRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no |
| `InsightRecord` | no | no | optional | yes | no | no | optional | optional | optional |
| `PredictionRecord` | no | no | optional | optional | yes | no | optional | optional | optional |
| `PrescriptionRecord` | no | no | optional | optional | optional | yes | optional | optional | optional |
| `OutcomeRecord` | yes | yes | yes | yes | optional | optional | yes | optional | optional |
| `GraphNodeRecord` | optional | optional | yes | yes | optional | optional | optional | optional | optional |
| `GraphEdgeRecord` | optional | optional | yes | yes | optional | optional | optional | optional | optional |
| `KnowledgeEntityRecord` | optional | optional | yes | yes | optional | optional | optional | optional | optional |
| `KnowledgeRelationRecord` | optional | optional | yes | yes | optional | optional | optional | optional | optional |
| `RuntimeStatusRecord` | yes | yes | yes | optional | optional | optional | yes | optional | no |
| `DeliveryRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no |

## Rule

This matrix is not illustrative only; future contract work should fill it out
completely and keep it authoritative.
