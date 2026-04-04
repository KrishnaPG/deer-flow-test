# Design: Modular Toolkit - World Projection Object Contract

**Date:** 2026-03-31
**Status:** Approved

## World Projection Object Families

- `WorldConversationAnchor`
- `WorldTaskBeacon`
- `WorldArtifactUnlock`
- `WorldInsightOverlay`
- `WorldPredictionOverlay`
- `WorldPrescriptionPrompt`
- `WorldOutcomeMarker`

## Rule

Every world projection object derived from `C:L2` views must retain backlinks to:

- backing `C:L2` view IDs

- source A/B storage row IDs
- level/plane metadata
- discovery object kind
- drill-down panel targets
- retrieval mode
- lifecycle and supersession semantics
- whether the object is observed, inferred, predicted, prescribed, or an outcome
