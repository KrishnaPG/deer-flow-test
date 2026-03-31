# Design: Modular Toolkit - Normalizer Promotion Rules

**Date:** 2026-03-31
**Status:** Draft revision

## Promotion Path

- L0 source capture -> L1 sanitation
- L1 sanitation -> L2 canonical/view shaping
- L2 view shaping -> L3 insights
- L2/L3 -> L4 predictions
- L4 -> L5 prescriptions
- later observed evidence plus prior L4/L5 chains -> L6 outcomes

## Rule

Normalizers do not invent arbitrary future-state semantics.

They promote records according to explicit rules and declared producers.

Governance and operational records such as `TransformRecord`, `IntentRecord`,
`ExclusionRecord`, `ConflictRecord`, `ResolutionRecord`, `DedupRecord`,
`BatchRecord`, `ReplayCheckpointRecord`, and `WriteOperationRecord` may be
emitted alongside promotion, rather than being forced into the semantic spine.
