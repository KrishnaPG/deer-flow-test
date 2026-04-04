# Design: Modular Toolkit - Index

**Date:** 2026-03-31
**Status:** Accepted for planning
**Scope:** `apps/deer_gui` and supporting toolkit/apps

## Overview

This spec set replaces the earlier monolithic modular-toolkit design with a
smaller, toolkit-first set of documents.

Core position:

- the primary product is a reusable toolkit, not a single app
- authoring and inspection tools are first-class proving grounds
- TDD happens at module boundaries before app composition
- `deer_gui` becomes the first playable composition target, not the place where
  core modules are invented

## File Map

| File | Purpose |
| --- | --- |
| `00-index.md` | entry point, scope, file map |
| `01-architecture-position.md` | architectural stance and core principles |
| `02-workspace-and-layering.md` | workspace shape, A/B storage rows, and C:L2 presentation layering |
| `03-reusable-modules.md` | lego-brick modules, contracts, and boundaries |
| `04-authoring-tools-as-proving-grounds.md` | tool apps that prove each module slice |
| `05-tdd-by-contract.md` | test strategy and module-first TDD workflow |
| `06-milestones-to-first-playable.md` | milestone ladder from toolkit to playable app |
| `07-chat-and-artifact-pipeline.md` | live chat, streaming, uploads, and artifact-access mapping |
| `08-chat-to-world-bridging.md` | how generic chat state projects into RTS/RPG world state |
| `09-non-negotiables.md` | top architectural principles that must survive planning and implementation |
| `10-planning-guardrails.md` | mandatory planning and milestone guardrails to prevent drift |
| `11-state-server-alignment.md` | how toolkit architecture aligns with storage-native state-server rules |
| `12-levels-and-planes-contract.md` | L0-L5 and As-Is/Chunks/Embeddings contract dimensions |
| `13-lineage-and-immutability-contract.md` | hash-based provenance, append-only lineage, and correlation requirements |
| `14-data-trajectory-model.md` | how data moves across levels, transforms, and future outcomes |
| `15-discovery-object-taxonomy.md` | taxonomy for artifacts, views, insights, predictions, and prescriptions |
| `16-retrieval-and-indexing-contract.md` | mediated reads, chunking, embeddings, and index/backlink contracts |
| `17-view-taxonomy-and-lod.md` | thumbnail/tooltip/panel/world view taxonomy and context rules |
| `18-generator-mapping-matrix.md` | how DeerFlow, PocketFlow, Rowboat, Hermes, and others map into shared A/B contracts |
| `19-ui-state-server-boundary.md` | exact allowed and forbidden UI/runtime boundary assumptions |
| `20-intent-and-mediated-read-model.md` | external read/write payload families and mediated access rules |
| `21-implementation-readiness-checklist.md` | hard no-go gate before any crate/runtime implementation |
| `22-canonical-record-families.md` | generator-agnostic A/B storage row families |
| `23-correlation-and-identity-contract.md` | identity and correlation contract for all major record families |
| `24-level-plane-lineage-matrix.md` | occupancy matrix across levels, planes, and lineage needs |
| `25-generator-capability-matrix.md` | rich/sparse support map per generator and record family |
| `26-normalizer-promotion-rules.md` | normalizer rules from raw capture into shared A/B L2+ rows |
| `27-raw-envelope-family-catalog.md` | raw envelope families before A/B row production |
| `28-entity-view-matrix.md` | field and context expectations for presentation-layer entities and C:L2 view tiers |
| `29-world-projection-object-contract.md` | downstream world projection objects derived from backing C:L2 views |
| `30-library-decision-matrix.md` | exact OSS choices per implementation stage |
| `31-custom-code-boundaries.md` | what remains custom and what must not be custom first |
| `32-interactive-chat-lifecycle.md` | generator-agnostic lifecycle for interactive chat-oriented clients |
| `33-chat-event-and-state-mapping.md` | generic chat event families and canonical mapping rules |
| `34-chat-artifact-lifecycle.md` | generic upload/output/presentation/retrieval lifecycle for chat artifacts |
| `35-carrier-core-and-typed-extensions-contract.md` | shared base, mandatory modules, and profile-driven module attachment rules |
| `36-view-contract-support-model.md` | primary support model from view contracts to panels and layouts |
| `37-shell-mode-support-matrix.md` | shell-mode support from operating modes through panels, layouts, and canonical dependencies |
| `38-generator-shell-mode-support-evaluation.md` | backend evaluation of DeerFlow, Hermes, PocketFlow, and Rowboat against shell-mode support |
| `39-linked-interaction-contract.md` | canonical shell-level brushing, filtering, selection, focus, pinning, compare, and intent interaction rules |
| `40-shell-and-linked-interaction-stress-test.md` | stress test of shell and linked-interaction contracts against RTS HUD and state-server edge cases |
| `41-final-integration-tranche.md` | world-primary, temporal, failover, policy, and intent-boundary integration tranche before planning |
| `42-new-generator-onboarding-contract.md` | contract and readiness gate for onboarding any new backend generator |

## Stable Decisions

- Save this line of work under `docs/superpowers/` only.
- Prefer multiple small files over one large design file.
- Follow strict layering:
  `raw -> shared A/B storage rows -> C:L2 SQL/materialized-view presentation layer -> reusable view/panel consumers -> world projection as downstream consumer -> app`.
- Reusable modules must stay app-agnostic.
- Tool apps must consume the same reusable modules as runtime apps.
- Presentation logic belongs in `C:L2` SQL/materialized views rather than ad hoc app-local derivations.
- World projection is fed by presentation-layer outputs and must not be treated as the primary presentation contract.
- `deer_gui` is one composition target, not the monolithic center.
- planning acceptance recorded in
  `docs/superpowers/specs/modular-toolkit/10-planning-guardrails.md`
  covers the discovery tranche through `31-custom-code-boundaries.md`
  plus reconciliation `37`-`41`
- the indexed plan docs were written before implementation work began, and the
  current implementation-progress state now lives in
  `docs/superpowers/specs/modular-toolkit/21-implementation-readiness-checklist.md`
  and `docs/superpowers/plans/2026-04-01-functional-game-gap-plan-index.md`

## Reading Order

Read in this order:

1. `01-architecture-position.md`
2. `02-workspace-and-layering.md`
3. `03-reusable-modules.md`
4. `04-authoring-tools-as-proving-grounds.md`
5. `05-tdd-by-contract.md`
6. `06-milestones-to-first-playable.md`
7. `07-chat-and-artifact-pipeline.md`
8. `08-chat-to-world-bridging.md`
9. `09-non-negotiables.md`
10. `10-planning-guardrails.md`
11. `11-state-server-alignment.md`
12. `12-levels-and-planes-contract.md`
13. `13-lineage-and-immutability-contract.md`
14. `14-data-trajectory-model.md`
15. `15-discovery-object-taxonomy.md`
16. `16-retrieval-and-indexing-contract.md`
17. `17-view-taxonomy-and-lod.md`
18. `18-generator-mapping-matrix.md`
19. `19-ui-state-server-boundary.md`
20. `20-intent-and-mediated-read-model.md`
21. `21-implementation-readiness-checklist.md`
22. `22-canonical-record-families.md`
23. `23-correlation-and-identity-contract.md`
24. `24-level-plane-lineage-matrix.md`
25. `25-generator-capability-matrix.md`
26. `42-new-generator-onboarding-contract.md`
27. `26-normalizer-promotion-rules.md`
28. `27-raw-envelope-family-catalog.md`
29. `28-entity-view-matrix.md`
30. `35-carrier-core-and-typed-extensions-contract.md`
31. `36-view-contract-support-model.md`
32. `29-world-projection-object-contract.md`
33. `30-library-decision-matrix.md`
34. `31-custom-code-boundaries.md`
35. `32-interactive-chat-lifecycle.md`
36. `33-chat-event-and-state-mapping.md`
37. `34-chat-artifact-lifecycle.md`
38. `37-shell-mode-support-matrix.md`
39. `38-generator-shell-mode-support-evaluation.md`
40. `39-linked-interaction-contract.md`
41. `40-shell-and-linked-interaction-stress-test.md`
42. `41-final-integration-tranche.md`

## Superseded Document

The old single-file version is superseded by this directory:

- `docs/superpowers/specs/2026-03-31-modular-toolkit-architecture-design.md`
