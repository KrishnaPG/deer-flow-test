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

### 2. Accepted Input

Definition:

- an input accepted into the mediated system and now part of append-only truth or
  durable processing state

Examples:

- accepted upload
- source object acknowledged by the system

### 3. In-Flight Output

Definition:

- an output being generated or transferred but not yet fully available for normal
  user interaction

Examples:

- write in progress
- partial file generation
- delayed preview pipeline

### 4. Presented Artifact

Definition:

- an artifact explicitly made visible or attached to the session/task context for
  user-facing discovery

Examples:

- output shown in artifact shelf
- file added to result context

### 5. Previewable Artifact

Definition:

- an artifact available for inline or mediated preview

Examples:

- markdown preview
- snippet preview
- authorized image preview

### 6. Retrievable Artifact

Definition:

- an artifact available through mediated download, pointer resolution, or search
  retrieval

Examples:

- authorized download
- pointer-based open
- retrieval/search hit with backlinks

### 7. Projected Artifact

Definition:

- an artifact that has been projected into another downstream surface such as a
  world marker, codex item, replay link, or planning object

Examples:

- world unlock
- replay-linked output
- insight overlay source

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
