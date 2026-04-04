# Design: Modular Toolkit - Normalizer Promotion Rules

**Date:** 2026-03-31
**Status:** Approved

## Promotion Path

- L0 source capture -> L1 sanitation
- L1 sanitation -> A/B `L2` storage-row shaping
- A/B `L2` storage-row shaping -> L3 insights
- L2/L3 -> L4 predictions
- L4 -> L5 prescriptions
- later observed evidence plus prior L4/L5 chains -> L6 outcomes

## Normalizer Rule

Normalizers produce shared A/B L2+ storage rows that C:L2 SQL views query.
They do not invent arbitrary future-state semantics.

They transform raw L0/L1 evidence into A/B storage rows according to explicit rules and declared producers.

Governance and operational rows such as `TransformRecord`, `IntentRecord`,
`ExclusionRecord`, `ConflictRecord`, `ResolutionRecord`, `DedupRecord`,
`BatchRecord`, `ReplayCheckpointRecord`, and `WriteOperationRecord` may be
emitted alongside A/B storage row production, rather than being forced into the level-prefixed semantic extension families.
