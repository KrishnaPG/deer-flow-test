# Design: Modular Toolkit - Data Trajectory Model

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

No data item exists in isolation.

Each item has a trajectory:

- where it comes from
- how it is transformed
- where it goes next
- how later outcomes relate back to it

## Trajectory Rule

The toolkit must model not just state, but movement across levels.

The generic trajectory is:

`source capture -> sanitation -> view shaping -> insight discovery -> prediction -> prescription -> outcome`

## Stage Model

### Stage A - Source Capture

Typically L0 / As-Is.

Examples:

- generator stream events
- logs
- tool outputs
- media payloads
- user intents

### Stage B - Sanitation

Typically L1.

Examples:

- coercion
- schema cleanup
- file conversion
- de-duplication metadata

### Stage C - View Shaping

Typically L2.

Examples:

- canonical domain records
- reusable derived views
- domain joins and projections

### Stage D - Insight Discovery

Typically L3.

Examples:

- summaries
- anomaly detections
- extracted entities/relations
- operational findings

### Stage E - Prediction

Typically L4.

Examples:

- risk forecasts
- generated future reports
- projected outcomes
- counterfactual scenarios

### Stage F - Prescription

Typically L5.

Examples:

- mitigation plans
- optimization strategies
- interventions
- operator recommendations

### Stage G - Outcome

Typically L6.

Outcome records evaluate later observed evidence against earlier L4/L5 chains.

This matters because a successful prescription may intentionally make a prior
prediction "wrong."

## Contract Implication

Records should be able to encode trajectory relationships such as:

- `derived_from`
- `inferred_from`
- `predicts`
- `prescribes`
- `influences`
- `observed_outcome_of`
- `supersedes`

Operational and governance records may also participate in trajectory:

- `intent_causes`
- `transforms`
- `deduplicates`
- `batches`
- `resolves_conflict_for`
- `checkpoints_replay_for`

## UI Implication

The toolkit should support views that make trajectory visible, not hidden.

Examples:

- artifact provenance chains
- replay from source to effect
- graph edges that include transform semantics
- detail panels that show source, transform, destination, and next consumers

## World Implication

World projection should distinguish:

- observed current world state
- insight overlays
- forecast overlays
- prescription/intervention overlays

Those are not the same thing and should not be flattened together.

## Anti-Drift Rule

If the architecture treats predictions and prescriptions as just another artifact
file type, it has lost the trajectory model.
