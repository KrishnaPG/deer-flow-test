# Design: Modular Toolkit - Custom Code Boundaries

**Date:** 2026-03-31
**Status:** Draft revision

## Must Be Custom

- state-server-aware normalizers
- canonical storage/trajectory-aware record types
- lineage-preserving world projection
- artifact provenance and retrieval adapters
- trajectory-aware replay semantics

## Must Not Be Custom First

- graph data structure core
- docking framework
- generic transport replacement for DeerFlow bridge

## Drift Rule

If a custom implementation appears in an area listed under "Must Not Be Custom
First" before a documented exception is approved, it is architectural drift.
