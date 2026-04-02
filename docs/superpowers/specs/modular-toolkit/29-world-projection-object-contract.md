# Design: Modular Toolkit - World Projection Object Contract

**Date:** 2026-03-31
**Status:** Approved

## World Object Families

- `WorldConversationAnchor`
- `WorldTaskBeacon`
- `WorldArtifactUnlock`
- `WorldInsightOverlay`
- `WorldPredictionOverlay`
- `WorldPrescriptionPrompt`
- `WorldOutcomeMarker`

## Rule

Every world object must retain backlinks to:

- source record IDs
- level/plane metadata
- discovery object kind
- drill-down panel targets
- retrieval mode
- lifecycle and supersession semantics
- whether the object is observed, inferred, predicted, prescribed, or an outcome
