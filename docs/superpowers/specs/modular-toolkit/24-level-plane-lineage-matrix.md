# Design: Modular Toolkit - Level Plane Lineage Matrix

**Date:** 2026-03-31
**Status:** Approved

## Occupancy Matrix

| Record Family | L0 | L1 | L2 | L3 | L4 | L5 | L6 | As-Is | Chunks | Embeddings |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `L0_SourceRecord` | yes | no | no | no | no | no | no | optional | exceptional | no |
| `L1_SanitizedRecord` | no | yes | no | no | no | no | no | yes | typical | typical |
| `L2_ViewRecord` | no | no | yes | no | no | no | no | yes | typical | typical |
| `L3_InsightRecord` | no | no | optional | yes | no | no | no | yes | typical | typical |
| `L4_PredictionRecord` | no | no | optional | optional | yes | no | no | yes | typical | typical |
| `L5_PrescriptionRecord` | no | no | optional | optional | optional | yes | no | yes | optional | optional |
| `L6_OutcomeRecord` | no | no | optional | optional | optional | optional | yes | yes | optional | optional |
| `RunRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `SessionRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `TaskRecord` | optional | optional | yes | yes | optional | optional | optional | optional | no | no |
| `MessageRecord` | yes | yes | yes | optional | optional | optional | optional | optional | optional | no |
| `ToolCallRecord` | yes | yes | yes | optional | optional | optional | optional | optional | optional | no |
| `ArtifactRecord` | yes | yes | yes | optional | optional | optional | optional | optional | optional | optional |
| `ClarificationRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `RuntimeStatusRecord` | yes | yes | yes | optional | optional | optional | optional | optional | no | no |
| `DeliveryRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `AsIsRepresentationRecord` | allowed | allowed | allowed | allowed | allowed | allowed | allowed | yes | no | no |
| `ChunkRecord` | exceptional | typical | typical | typical | typical | optional | optional | no | yes | no |
| `EmbeddingRecord` | no | typical | typical | typical | typical | optional | optional | no | no | yes |
| `IntentRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `TransformRecord` | optional | optional | yes | yes | yes | yes | yes | optional | optional | optional |
| `ExclusionRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `ConflictRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `ResolutionRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `ReplayCheckpointRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `DedupRecord` | optional | optional | yes | optional | optional | optional | optional | optional | optional | no |
| `BatchRecord` | optional | optional | yes | optional | optional | optional | optional | optional | optional | no |
| `BranchRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `VersionRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |
| `WriteOperationRecord` | optional | optional | yes | optional | optional | optional | optional | optional | no | no |

## Rule

This matrix is not illustrative only; future contract work should fill it out
completely and keep it authoritative.
