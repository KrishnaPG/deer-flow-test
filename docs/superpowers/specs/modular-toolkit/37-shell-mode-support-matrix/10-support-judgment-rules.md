## Support Judgment Rules

### Shell-Supported

A backend supports a shell mode only when all of the following are true:

- all required views are `view-supported`
- all required panels derived from those views are supported
- all required layout traits are satisfiable by the supported panel set
- all required linked dashboard interactions are satisfiable across the hosted
  views and panels in that shell mode
- all required canonical record families are available through stable canonical
  mappings
- all required metadata is present at the point of use
- all required mediated reads are available through declared read contracts
- all required intents are available where the shell mode is actionable
- all required broker ownership maps are declared and satisfiable for each
  required linked interaction type
- shell modes that rely on linked-gesture command seeding declare which paths are
  `prefill_seed`-capable and which remain intent-neutral
- shells with temporal behavior preserve broker epoch semantics, freshness
  visibility, and late-event safety
- shells with policy overlays invalidate or tombstone linked state on relevant
  `policy_epoch` changes
- required world projections, if any, preserve drill-down backlinks to supported
  detail views

Required linked dashboard interactions may include:

- shared selection propagation
- brushing from one view into filters on sibling views
- shell-level filter state applied across multiple hosted views
- focus propagation from rails, queues, replay, or search into inspector and
  center views

Required linked dashboard interactions are mode-specific.

- `filtering` is required only where the shell mode explicitly declares shared
  filter behavior
- world-primary shells are not `full` unless navigation, camera sync, and
  command-target separation are all preserved

### Coverage Labels

- `full`
  - all required and optional shell contracts are supported
- `partial`
  - all required shell contracts are supported, but one or more optional views,
    enrichments, or world-bearing extensions degrade according to the declared
    shell mode contract
- `unsupported`
  - one or more required shell contracts cannot be satisfied

### Partial Support Constraint

Do not use `partial` unless the operator can still:

- enter the shell mode
- inspect required detail
- traverse required drill-downs
- preserve stable identity and provenance
- perform required cross-view brushing and filtering when the shell mode depends
  on it
- issue required intents, if the shell mode is actionable

If any of those fail, the shell mode is `unsupported`.

## Anti-Drift Rules

- Do not claim shell support from backend nouns alone.
- Do not treat raw payloads as canonical support.
- Do not treat direct database or storage access as mediated read support.
- Do not treat direct mutation endpoints as intent support.
- Do not treat side-by-side hosted views as independent silos when the shell mode
  requires linked dashboard behavior.
- Do not let world projection invent semantics that cannot be traced to
  supported canonical records and supported detail views.
- Do not label a shell mode `full` when required metadata is missing and the UI
  would need to guess identity, lineage, level, plane, or lifecycle.

## Next Step This Enables

The next backend evaluation pass should now judge DeerFlow, Hermes, PocketFlow,
and Rowboat against:

- required canonical mappings per shell mode
- required view support
- derived panel support
- derived layout trait support
- final `full` / `partial` / `unsupported` shell mode support
