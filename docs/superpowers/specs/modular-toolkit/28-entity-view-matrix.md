# Design: Modular Toolkit - Entity View Matrix

**Date:** 2026-03-31
**Status:** Approved

| Entity | Ontology Family | Thumbnail Fields | Tooltip Fields | Panel Fields | World Fields |
| --- | --- | --- | --- | --- | --- |
| agent | carrier/view-derived | identity, status | role, task, health | assignments, history, outputs | optional unit/beacon fields |
| `SessionRecord` | carrier/orchestration | title, state | last turn, counts | transcript, composer, provenance | `WorldConversationAnchor` fields |
| `ArtifactRecord` | carrier/orchestration | type, status | preview, producer | full metadata, provenance, actions | optional `WorldArtifactUnlock` fields |
| `TaskRecord` | carrier/orchestration | state, severity | owner, blocker | subtasks, timings, dependencies | `WorldTaskBeacon` fields |
| replay event | governance/view-derived | type, time | summary, cause | payload, before/after, links | optional pulse/ghost fields |
| `InsightRecord` | semantic spine | type, confidence | summary, source | rationale, provenance, related records | `WorldInsightOverlay` fields |
| `PredictionRecord` | semantic spine | type, confidence, horizon | summary, assumptions | full forecast, assumptions, lineage | `WorldPredictionOverlay` fields |
| `PrescriptionRecord` | semantic spine | urgency, type | action summary, target | full plan, expected effect, lineage | `WorldPrescriptionPrompt` fields |
| `OutcomeRecord` | semantic spine | verdict, delta | summary, comparison | full evaluation, evidence, lineage | `WorldOutcomeMarker` fields |

## Required Contract Fields Per Entry

Every row above must later define:

- required commands/events
- source/backlink metadata
- allowed contexts
- whether world representation is permitted
- lifecycle and supersession rules
