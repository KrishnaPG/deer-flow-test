# Design: Modular Toolkit - Intent And Mediated Read Model

**Date:** 2026-03-31
**Status:** Approved

## External Read Families

- live-activity stream payload
- historical snapshot payload
- artifact preview payload
- artifact pointer payload
- replay window payload
- retrieval/search result payload

## External Write Families

- operator intent
- clarification response
- approval/denial intent
- intervention intent

These are external write families. They may exist as shell-local
`prefill_seed`, `prefill`, `draft`, or `validated` state, but become canonical
only on explicit `submitted`. `approved`, `executed`, and `rejected` are later
observed mediated outcomes, not locally asserted stages.

## Required Metadata

- ABAC result or visibility scope
- correlation identifiers
- level and plane hints
- lineage backlinks

## Intent Lifecycle Rule

Intent handling must follow this pattern:

1. accept or prefill candidate intent context from an allowed surface
2. materialize shell-local candidate state as `prefill_seed`
3. promote to `prefill` only when the shell instantiates an editable intent
   entry from that seed
4. promote to `draft` when the operator takes ownership of the editable entry
5. perform explicit target, parameter, and policy validation, producing
   `validated`
6. append `IntentRecord` only when the operator explicitly enters `submitted`
7. emit `WriteOperationRecord` or related mediated progress after submission
8. surface `approved`, `executed`, or `rejected` later as observed mediated
   state

Shortcut mutation models are forbidden.
Local gestures may not jump directly from selection, replay, or world
interaction into `prefill`, `draft`, `submitted`, `approved`, or `executed`.
