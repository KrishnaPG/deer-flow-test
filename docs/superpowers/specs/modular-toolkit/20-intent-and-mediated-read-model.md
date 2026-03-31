# Design: Modular Toolkit - Intent And Mediated Read Model

**Date:** 2026-03-31
**Status:** Draft revision

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

These should become first-class `IntentRecord` statements in the canonical
ontology, not just transient UI commands.

## Required Metadata

- ABAC result or visibility scope
- correlation identifiers
- level and plane hints
- lineage backlinks

## Intent Lifecycle Rule

Intent handling must follow this pattern:

1. validate intent and policy scope
2. append `IntentRecord` as storage-backed truth or append-only event
3. emit `WriteOperationRecord` or related mediated progress
4. surface later observed state as a result of the append-only flow

Shortcut mutation models are forbidden.
