# Design: Modular Toolkit - State Server Alignment

**Date:** 2026-03-31
**Status:** Approved

## Why This File Exists

The modular-toolkit design originally focused on reusable UI/runtime layering.

That is necessary, but not sufficient.

The toolkit must also align with the storage-native state-server architecture in
`docs/architecture/state-server.md`.

## Alignment Rule

The toolkit does not define truth independently.

It consumes and presents truth that is governed by these state-server rules:

- storage is the immutable source of truth
- databases are views and indexes over storage
- state-server is the mediated read gateway for external clients
- external writes are intents, not direct storage mutations
- lineage is append-only and never rewritten
- storage-native truth records never use model-level `UPDATE` or `DELETE` to
  rewrite prior truth; correction, redaction, and suppression append new records
  that reference prior truth without mutating or removing its storage-native
  lineage anchor

## Architectural Mapping

### Storage Truth

The toolkit must assume that persisted truth originates in storage-backed records
and lineage events, not in app-local state.

Examples:

- raw event objects
- artifact payloads
- chunk records
- embedding records
- append-only lineage and progress events

### State Server Boundary

The toolkit should treat State Server as the external read boundary.

That means proof apps and runtime clients should be modeled as consumers of:

- mediated live activity streams
- authorized artifact pointers
- cache-backed historical views
- ABAC-filtered snapshots and diffs

Not direct consumers of arbitrary backend internals.

### Intent Boundary

Human and UI-originated actions are intents.

They should be modeled as commands that:

- express desired action
- pass through policy/ABAC validation
- become append-only records/events
- later appear as new observed state

## Toolkit Consequence

The client/runtime layering must be read as:

`generator -> storage truth -> mediated source -> normalizer -> A/B storage rows -> C:L2 SQL views -> reusable hosted views / world projection objects -> app`

This is the correct storage-aware interpretation of the earlier toolkit layer
model.

## What The Toolkit Must Not Assume

The toolkit must not assume:

- raw backend payloads are the same thing as shared A/B storage truth
- direct artifact paths are safe for UI access
- DB/search/index records are the only source of truth
- a final assistant message is more important than the storage and lineage chain

## Required Contract Implications

Every important record family should carry enough metadata to answer:

- where did this come from?
- what storage level and plane does it belong to?
- what immutable source object or hash anchors it?
- what transformed it?
- what downstream consumers depend on it?

## Discovery Consequence

Before implementation planning continues, discovery must explicitly specify:

- state-server-aligned contract dimensions
- lineage semantics
- level/plane semantics
- mediated read and intent/write assumptions

That discovery tranche is what the next spec files define.
