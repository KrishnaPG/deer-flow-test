# Design: Modular Toolkit - Entity View Matrix

**Date:** 2026-03-31
**Status:** Draft revision

| Entity | Thumbnail Fields | Tooltip Fields | Panel Fields | World Fields |
| --- | --- | --- | --- | --- |
| agent | identity, status | role, task, health | assignments, history, outputs | optional unit/beacon fields |
| session | title, state | last turn, counts | transcript, composer, provenance | anchor/beacon fields |
| artifact | type, status | preview, producer | full metadata, provenance, actions | optional unlock/marker fields |
| task | state, severity | owner, blocker | subtasks, timings, dependencies | beacon/hotspot fields |
| replay event | type, time | summary, cause | payload, before/after, links | optional pulse/ghost fields |
| insight | type, confidence | summary, source | rationale, provenance, related records | optional overlay fields |
| prediction | type, confidence, horizon | summary, assumptions | full forecast, assumptions, lineage | optional overlay fields |
| prescription | urgency, type | action summary, target | full plan, expected effect, lineage | prompt/marker fields |

## Required Contract Fields Per Entry

Every row above must later define:

- required commands/events
- source/backlink metadata
- allowed contexts
- whether world representation is permitted
- lifecycle and supersession rules
