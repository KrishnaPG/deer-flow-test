# Design: Modular Toolkit - Generator Mapping Matrix

**Date:** 2026-03-31
**Status:** Approved

## Why This File Exists

The toolkit must support multiple data generators, not only DeerFlow.

So the A/B storage families must be generator-agnostic.

## Supported Generator Families

### DeerFlow

Strengths:

- thread/run/chat semantics
- live stream events
- tool activity
- artifacts and clarification

Natural fit:

- carrier/orchestration-heavy flows with rich chat, tool, and artifact behavior

### PocketFlow

Strengths:

- graph execution
- transitions
- retries
- batch/parallel flow structure

Natural fit:

- carrier/orchestration plus structural workflow topology and execution lifecycle

### Rowboat

Strengths:

- runs/messages/tools
- knowledge graph
- background services
- file/version provenance

Natural fit:

- representation/provenance-heavy flows with knowledge and file lineage

### Hermes Agent

Strengths:

- session/message/tool streaming
- delegation
- delivery and scheduling
- runtime/platform status

Natural fit:

- carrier/orchestration-heavy flows with delivery, scheduling, and runtime state

## Canonical Mapping Rule

The shared generator contract should be centered on generator-agnostic A/B storage families plus downstream `C:L2` view support:

- mandatory carrier/orchestration families
- level-prefixed semantic extension families
- representation/index families
- governance/operational families
- structural/knowledge families where the generator supports them

Graph structure should be supported, but not required as the only source truth.

## Mapping Expectation

Each generator family should eventually have a mapping spec that answers:

- what raw event families it emits
- what storage levels and planes it naturally populates
- what ontology families it can populate richly
- what ontology families it can only populate sparsely
- what `C:L2` view contracts or world-projection surfaces it supports directly vs indirectly

## Anti-Drift Rule

Do not let one generator's natural model become the toolkit's only worldview.

DeerFlow is important, but it is not the canonical shape of every future source.
