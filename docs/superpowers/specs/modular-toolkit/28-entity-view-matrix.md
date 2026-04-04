# Design: Modular Toolkit - Entity View Matrix

**Date:** 2026-03-31
**Status:** Approved

| Entity | Ontology Family | Thumbnail Fields | Tooltip Fields | Panel Fields | World Fields |
| --- | --- | --- | --- | --- | --- |
| agent | carrier/orchestration projection | identity, status | role, task, health | assignments, history, outputs | optional unit/beacon fields |
| `SessionRecord` | carrier/orchestration | title, state | last turn, counts | transcript, composer, provenance | `WorldConversationAnchor` fields |
| `ArtifactRecord` | carrier/orchestration | type, status | preview, producer | full metadata, provenance, actions | optional `WorldArtifactUnlock` fields |
| `TaskRecord` | carrier/orchestration | state, severity | owner, blocker | subtasks, timings, dependencies | `WorldTaskBeacon` fields |
| replay event | governance/operational projection | type, time | summary, cause | payload, before/after, links | optional pulse/ghost fields |
| `InsightRecord` | profile-driven semantic / admissible `C:L0` | type, confidence | summary, source | rationale, provenance, related records | shell `C:L2` -> `WorldInsightOverlay` |
| `PredictionRecord` | profile-driven semantic / admissible `C:L0` | type, confidence, horizon | summary, assumptions | full forecast, assumptions, lineage | shell `C:L2` -> `WorldPredictionOverlay` |
| `PrescriptionRecord` | profile-driven semantic / admissible `C:L0` | urgency, type | action summary, target | full plan, expected effect, lineage | shell `C:L2` -> `WorldPrescriptionPrompt` |
| `OutcomeRecord` | profile-driven semantic / admissible `C:L0` | verdict, delta | summary, comparison | full evaluation, evidence, lineage | shell `C:L2` -> `WorldOutcomeMarker` |

## Projection Note

- Matrix entries describe presentation-facing projection targets, not storage-truth declarations.
- Admissible `C:L0` inputs come from allowed A/B `L2+` rows; this includes high-order semantic rows such as `InsightRecord`, `PredictionRecord`, `PrescriptionRecord`, and `OutcomeRecord`.
- Shells own the canonical `C:L2` views that project those inputs into thumbnail, tooltip, panel, and optional world-facing surfaces.

## Required Contract Fields Per Entry

Every row above must later define:

- required commands/events
- source/backlink metadata
- allowed contexts
- whether world representation is permitted
- lifecycle and supersession rules
