# Design: Modular Toolkit - Generator Capability Matrix

**Date:** 2026-03-31
**Status:** Approved

## A/B Storage Family Capability Matrix

| Generator | `RunRecord` / `SessionRecord` | `TaskRecord` | `MessageRecord` | `ToolCallRecord` | `ArtifactRecord` | `GraphNodeRecord` / `GraphEdgeRecord` | `KnowledgeEntityRecord` / `KnowledgeRelationRecord` | `DeliveryRecord` | `RuntimeStatusRecord` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DeerFlow | rich | rich | rich | rich | rich | medium | low | low | medium |
| PocketFlow | medium | rich | low | medium | low | rich | low | low | medium |
| Rowboat | rich | rich | rich | rich | rich | rich | rich | medium | rich |
| Hermes | rich | rich | rich | rich | medium | low | low | rich | rich |

## Generator-Agnostic Rule

Each A/B storage row family should show:

- at least one DeerFlow mapping
- at least one non-DeerFlow mapping or an explicit current gap
- which fields are A/B storage-shared vs generator-specific

Use `36-view-contract-support-model.md` to evaluate how that A/B family coverage translates into `C:L2` consumer support.
