# Design: Modular Toolkit - Discovery Object Taxonomy

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The toolkit needs a shared language for what kinds of objects it presents.

Without this, artifacts, insights, predictions, prescriptions, and views blur
together.

## Taxonomy

This taxonomy describes the semantic spine of discovery objects.

It does not describe every record family in the ontology.

Carrier/orchestration, representation/index, and governance/operational records
are separate ontology families.

### Source Object

Definition:

- a storage-native capture or payload object

Typical levels:

- L0 or L1

Examples:

- raw log object
- cleaned transcript payload
- uploaded file

### Sanitized Object

Definition:

- an L1 cleaned or normalized derivative of source capture

Examples:

- normalized transcript payload
- cleaned metadata view
- format-normalized object

### View Object

Definition:

- an L2 shaped record or reusable projection derived for inspection or runtime
  use

Examples:

- thread/session view
- artifact shelf item
- graph node projection

### Insight Object

Definition:

- an L3 historical finding or discovered fact

Examples:

- anomaly finding
- extracted entity
- summary of what happened

### Prediction Object

Definition:

- an L4 forecast, anticipated outcome, or speculative generation

Examples:

- risk forecast
- generated report
- projected failure path

### Prescription Object

Definition:

- an L5 intervention or recommended action intended to influence future state

Examples:

- mitigation plan
- operator action list
- resource reallocation recommendation

### Outcome Object

Definition:

- later observed data that confirms or invalidates prior insights, predictions,
  or prescriptions

Examples:

- actual mission result
- actual queue recovery
- actual operator intervention success/failure

## Toolkit Requirement

Major views and world projections should know what kind of discovery object they
are presenting.

At minimum they should distinguish:

- source/view
- insight
- prediction
- prescription
- outcome

Carrier records such as `SessionRecord`, `MessageRecord`, `TaskRecord`, and
`ArtifactRecord` are not themselves discovery kinds. They carry, reference, or
present discovery objects.

Representation families such as `AsIsRepresentationRecord`, `ChunkRecord`, and
`EmbeddingRecord` are also not discovery kinds.

## Anti-Drift Rule

Do not collapse all downstream results into a single `ArtifactRecord` concept.

Artifacts are important, but they are only one object family in the broader
discovery system.
