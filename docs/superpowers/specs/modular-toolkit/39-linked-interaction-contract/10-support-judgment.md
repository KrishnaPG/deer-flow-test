## Support Judgment

### `full`

- all required and optional linked interaction paths work as declared
- every required participant can publish and consume linked state as declared
- propagation uses stable canonical join keys and required metadata without
  semantic guessing
- linked state preserves identity, provenance, and drill-down safety end to end

### `partial`

- all required linked interaction paths still work
- one or more optional participants, optional dimensions, or optional
  enrichments degrade only as declared
- degraded behavior still preserves stable identity, provenance, and drill-down
  safety

### `unsupported`

- any required linked interaction path cannot be satisfied
- any required participant cannot publish or consume the required linked state
- required linkage would depend on guessed joins, lossy identity, missing
  provenance, or unsafe drill-downs
- navigational gestures can implicitly arm or retarget commands
- broker recovery can resume under the same epoch after failover
- excluded references remain in selection, compare, focus, or drill-down paths
  without tombstone or removal

## Acceptable Degradation

Examples of acceptable degradation:

- an optional participant does not join the linked state
- an optional brush visualization degrades to explicit selection or filter chips
- an optional continuous range brush degrades to discrete bucket filtering
- optional ranking, embedding, or world enrichments are absent while canonical
  linked filtering still works
- explicit checkpoint stepping may replace continuous temporal scrubbing when
  canonical temporal anchors, freshness visibility, and drill-down safety remain
  intact

Examples of unacceptable degradation:

- required linkage becomes manual navigation instead of shared interaction state
- required bidirectional linkage becomes one-way only
- filters propagate visually but lose stable identity or provenance
- temporal or lineage correlation is approximated from display order alone
- world and minimap silently diverge when bidirectional sync is required

## Anti-Drift Rules

- Do not claim linkage from side-by-side placement alone.
- Do not claim linkage from matching labels, colors, or simultaneous updates
  alone.
- Do not claim linkage because views read from the same backend unless they share
  declared canonical join keys and linked-state semantics.
- Do not claim filtering support when each view applies independent local filters
  without shared canonical state.
- Do not claim brushing support when highlight coincidence cannot be traced to
  stable canonical records.
- Do not claim `action_intent_emission` unless the emitted action passes through
  the mediated intent model.
- Do not claim linked dashboard support if selection, filtering, brushing,
  focus, or drill-down require direct storage access or non-mediated reads.

## Design Consequence

Backend adapters alone should not be expected to prove linked shell behavior.

The reusable toolkit layer must own the composition logic that turns isolated
view support into:

- shared selection
- brush and filter propagation
- focus routing
- pinning and compare state
- drill-down-safe escalation
- mediated action intent emission
