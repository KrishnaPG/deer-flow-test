# Design: Modular Toolkit - UI State Server Boundary

**Date:** 2026-03-31
**Status:** Approved

## Allowed External Read Paths

- UI reads live activity through state-server-mediated streams
- UI reads historical state through mediated warm-cache or DB-backed responses
- UI reads artifacts through authorized pointers or mediated previews
- UI reads retrieval/search results through mediated result payloads with backlinks

## Allowed External Write Paths

- UI submits intents only
- intents are policy-checked before becoming append-only records/events

## Forbidden Assumptions

- UI does not treat raw backend internal payloads as source truth
- UI does not read storage, DB, or vector index backends directly in external-facing flows
- UI does not assume direct storage paths are safe to open
- proof apps must simulate mediated reads conceptually, not bypass them

## No-Bypass Rule

For external-facing runtime behavior, direct S3, ClickHouse, Milvus, or other
backend-specific reads are forbidden unless surfaced through a mediated contract
or authorized pointer model.
