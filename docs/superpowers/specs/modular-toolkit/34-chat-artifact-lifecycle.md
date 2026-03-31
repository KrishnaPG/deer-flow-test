# Design: Modular Toolkit - Chat Artifact Lifecycle

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

Chat-oriented systems often blur several different artifact states together.

The toolkit needs a generic artifact lifecycle that fits DeerFlow and other chat
clients.

## Artifact Lifecycle States

### 1. Staged Input

Definition:

- a user-selected or client-staged input that is not yet part of persisted
  conversational truth

Examples:

- upload queue item
- local attachment selection

Rule:

- staged input is client-transient and not yet canonical persisted truth

### 2. Accepted Input

Definition:

- an input accepted into the mediated system and now part of append-only truth or
  durable processing state

Examples:

- accepted upload
- source object acknowledged by the system

Canonical implication:

- `ArtifactRecord` as carrier/orchestration record
- linked `SourceRecord` or `SanitizedRecord`
- linked `AsIsRepresentationRecord`

### 3. In-Flight Output

Definition:

- an output being generated or transferred but not yet fully available for normal
  user interaction

Examples:

- write in progress
- partial file generation
- delayed preview pipeline

Canonical implication:

- `ArtifactRecord` plus governance/operational records such as
  `WriteOperationRecord` and related runtime/progress state

### 4. Presented Artifact

Definition:

- an artifact explicitly made visible or attached to the session/task context for
  user-facing discovery

Examples:

- output shown in artifact shelf
- file added to result context

Canonical implication:

- carrier/presentation state on `ArtifactRecord`
- payload access still comes through linked representation records

### 5. Previewable Artifact

Definition:

- an artifact available for inline or mediated preview

Examples:

- markdown preview
- snippet preview
- authorized image preview

Canonical implication:

- preview payload comes from linked `AsIsRepresentationRecord` or `ChunkRecord`

### 6. Retrievable Artifact

Definition:

- an artifact available through mediated download, pointer resolution, or search
  retrieval

Examples:

- authorized download
- pointer-based open
- retrieval/search hit with backlinks

Canonical implication:

- retrieval mode and pointer semantics are representation/retrieval concerns,
  not raw artifact identity

### 7. Downstream Projection

Definition:

- an artifact-linked downstream projection into another surface such as a world
  object, replay link, codex entry, or planning object

Rule:

- this is not an artifact lifecycle state by itself; it is a downstream
  projection backed by artifact, semantic, and representation links

## Lifecycle Rule

These states should be modeled separately.

The toolkit must not collapse:

- staged vs accepted input
- in-flight vs presented output
- presented vs previewable vs retrievable output
- artifact vs downstream projection

## Canonical Contract Implication

`ArtifactRecord` and related read-model/view contracts should preserve:

- artifact lifecycle state
- source/input vs generated/output role
- presentation status
- preview readiness
- retrieval mode
- projection/backlink targets

## Retrieval And View Implication

Artifact-facing views should be able to indicate:

- whether the user is seeing a staged input, a visible session artifact, or a
  retrieval-backed result
- whether the payload is full, partial, chunk-based, or pointer-based
- what downstream surfaces already depend on it

## Anti-Drift Rule

Do not model all artifacts as simple files with a path.

Artifact lifecycle and visibility are part of the domain contract.
