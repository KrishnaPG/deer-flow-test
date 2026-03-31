# Design: Modular Toolkit - Levels And Planes Contract

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The toolkit cannot treat "raw" and "derived" as enough.

The state-server architecture distinguishes both:

- levels `L0` through `L6`
- planes `as_is`, `chunks`, and `embeddings`

These must become first-class contract dimensions.

## Levels

### L0 - Source

Examples:

- raw logs
- live tokens/events
- source files
- media feeds
- archives

Canonical semantic extension:

- `L0_SourceRecord`

Meaning:

- minimally interpreted capture from generators or ingress systems

### L1 - Sanitized

Examples:

- normalized field formats
- cleaned payloads
- converted file formats
- null-imputed records

Canonical semantic extension:

- `L1_SanitizedRecord`

Meaning:

- storage-backed cleanup of raw capture without yet turning it into target-specific
  views

### L2 - Views

Examples:

- joined domain-facing records
- filtered projections
- segment/crop/channel outputs
- reusable toolkit-facing canonical views

Canonical semantic extension:

- `L2_ViewRecord`

Meaning:

- the first layer where domain-facing and client-facing views are shaped

### L3 - Aggregates / Insights

Examples:

- summaries
- historical trends
- anomaly findings
- knowledge extraction
- identified objects/relations

Canonical semantic extension:

- `L3_InsightRecord`

Meaning:

- discovered facts or historical insights derived from lower-level records

### L4 - Predictions

Examples:

- forecasts
- synthetic reports
- anticipated risks
- generated future outcomes
- creative projections

Canonical semantic extension:

- `L4_PredictionRecord`

Meaning:

- non-observed projected futures or speculative outputs

### L5 - Prescriptions

Examples:

- intervention plans
- mitigation playbooks
- optimization actions
- decision support actions

Canonical semantic extension:

- `L5_PrescriptionRecord`

Meaning:

- hypothetical actions intended to influence future outcomes and possibly render
  an L4 prediction obsolete

### L6 - Outcomes / Deviations

Examples:

- evaluation of predicted vs observed outcomes
- assessment of prescription effect
- deviation reports
- adjudication of what really happened relative to expectations

Canonical semantic extension:

- `L6_OutcomeRecord`

Meaning:

- an explicit evaluation layer linking prior L4/L5 chains to later observed
  evidence

## Planes

### As-Is Plane

Meaning:

- canonical payload or payload-equivalent object at a given level

Examples:

- raw media
- chat JSON
- cleaned text
- generated report markdown

Identity:

- anchored by an immutable content hash

### Chunks Plane

Meaning:

- segmented data derived from an As-Is object

Examples:

- text chunks
- image crops
- audio segments
- section slices

Identity:

- anchored by source object hash + chunking strategy + chunk index

### Embeddings Plane

Meaning:

- vector representation of chunk content

Rule:

- embeddings reference chunk identity and do not duplicate payload text as source
  truth

## Contract Rule

Every persisted or projected record of interest should be able to declare:

- current level
- current plane
- source level
- source plane
- whether it is storage-native, index-native, or client-transient

## Family Rule

Levels and planes are not the same thing as module classification or record
families.

The ontology should distinguish:

- mandatory carrier/orchestration module records
- profile-driven level-prefixed semantic extensions
- profile-driven representation/index module records
- profile-driven governance/operational module records

Planes remain orthogonal dimensions, while representation families such as
`AsIsRepresentationRecord`, `ChunkRecord`, and `EmbeddingRecord` make those
planes usable in practice.

Representation policy is selective rather than uniform:

- `AsIsRepresentationRecord` is broadly universal
- `ChunkRecord` is typical for `L1..L4`
- `EmbeddingRecord` is a derivative of `ChunkRecord`, not a default for every
  level

## Toolkit Interpretation

The toolkit should treat these as independent from UI layers.

Examples:

- a `TranscriptVm` row may reflect an L2 view backed by L0 and L1 records
- an anomaly marker may be an L3 insight
- a forecast card may be L4
- an intervention prompt may be L5
- an evaluated deviation card may be L6

## Required Record Metadata

At minimum, records should support fields equivalent to:

- `level`
- `plane`
- `source_level`
- `source_plane`
- `is_persisted_truth`
- `is_index_projection`
- `is_client_transient`

## Anti-Drift Rule

If a view cannot state whether it is showing L0, L2, L3, L4, or L5 data, the
design is too vague.
