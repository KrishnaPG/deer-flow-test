# Design: Modular Toolkit - View Taxonomy And LOD

**Date:** 2026-03-31
**Status:** Approved

## Why This File Exists

The toolkit needs explicit representation rules for entities across contexts.

Each entity should have defined levels of detail and context-specific views.

## LOD Tiers

### Thumbnail / Icon

Use when:

- dense overview
- minimap or rail
- compact search results
- low-attention summary

### Tooltip / Medium

Use when:

- hover
- scrub target
- pre-click decision
- focus without full context switch

### Panel / Full Detail

Use when:

- explicit inspection
- compare/pin
- act/edit/respond
- provenance review

### World / 3D Proxy

Use when:

- spatial continuity matters
- macro/micro world interpretation matters
- the item affects mission state, location, or intervention flow

## Entity Matrix

Major entity families should support these representations where relevant:

| Entity | Thumbnail | Tooltip | Panel | World |
| --- | --- | --- | --- | --- |
| agent | yes | yes | yes | optional |
| session/thread | yes | yes | yes | yes |
| artifact | yes | yes | yes | optional |
| task progress | yes | yes | yes | yes |
| graph node | yes | yes | yes | optional |
| replay event | yes | yes | yes | optional |
| world marker | yes | yes | yes | yes |

## Metadata Requirement

Every view tier should preserve enough metadata to show or recover:

- stable source IDs
- level and plane
- discovery object kind
- correlation and provenance backlinks

## Context Rules

The toolkit should explicitly choose view tier by context:

- hover -> tooltip
- dense list/rail/minimap -> thumbnail
- explicit inspect/open/pin -> panel
- macro/micro spatial flow -> world/3D proxy

## World Proxy Rule

Not every entity needs a world proxy.

World proxies are allowed only when they clarify spatial, mission, or
intervention meaning.

## Anti-Drift Rule

Do not let world projection invent entirely new information that cannot be traced
back to a supported panel/detail representation.
