# Design: Modular Toolkit - Raw Envelope Family Catalog

**Date:** 2026-03-31
**Status:** Approved

## Raw Families

- `SourceObjectEnvelope`
- `StreamDeltaEnvelope`
- `SnapshotEnvelope`
- `ArtifactEnvelope`
- `LineageEventEnvelope`
- `ProgressEnvelope`
- `RuntimeStatusEnvelope`
- `IntentEnvelope`
- `RetrievalHitEnvelope`

## Rule

Generators map into these envelope families (L0/L1 raw evidence) before normalizers transform them into shared A/B `L2+` storage rows. `C:L2` SQL views and materialized views are derived from those rows afterward.
