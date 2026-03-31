# Design: Modular Toolkit - Generator Mapping Matrix

**Date:** 2026-03-31
**Status:** Draft revision

## Why This File Exists

The toolkit must support multiple data generators, not only DeerFlow.

So the canonical model must be generator-agnostic.

## Supported Generator Families

### DeerFlow

Strengths:

- thread/run/chat semantics
- live stream events
- tool activity
- artifacts and clarification

Natural fit:

- session/run/message/tool/artifact-heavy flows

### PocketFlow

Strengths:

- graph execution
- transitions
- retries
- batch/parallel flow structure

Natural fit:

- workflow topology and execution lifecycle

### Rowboat

Strengths:

- runs/messages/tools
- knowledge graph
- background services
- file/version provenance

Natural fit:

- knowledge and file-provenance enriched orchestration

### Hermes Agent

Strengths:

- session/message/tool streaming
- delegation
- delivery and scheduling
- runtime/platform status

Natural fit:

- session/task/delivery-centric orchestration

## Canonical Model Rule

The canonical model should be centered on:

- runs and sessions
- tasks and progress
- messages and tool activity
- artifacts and outputs
- provenance and trajectory
- optional graph and knowledge projections

Graph structure should be supported, but not required as the only source truth.

## Mapping Expectation

Each generator family should eventually have a mapping spec that answers:

- what raw event families it emits
- what storage levels and planes it naturally populates
- what canonical records it can populate richly
- what canonical records it can only populate sparsely
- what view tiers it supports directly vs indirectly

## Anti-Drift Rule

Do not let one generator's natural model become the toolkit's only worldview.

DeerFlow is important, but it is not the canonical shape of every future source.
