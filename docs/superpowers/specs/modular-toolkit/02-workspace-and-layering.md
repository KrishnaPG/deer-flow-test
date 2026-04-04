# Design: Modular Toolkit - Workspace And Layering

**Date:** 2026-03-31
**Status:** Accepted for planning

## Target Workspace Shape

| Path | Purpose |
| --- | --- |
| `crates/foundation/contracts` | shared traits, DTOs, commands, events |
| `crates/foundation/domain` | shared A/B storage-row models for execution truth plus typed semantic, representation, governance, and knowledge extensions |
| `crates/foundation/replay` | replay model, fixture loading, event history |
| `crates/pipeline/raw_sources` | backend, file, and replay source adapters |
| `crates/pipeline/normalizers` | raw schema to shared A/B L2+ storage row transforms |
| `crates/pipeline/derivations` | A/B storage rows to C:L2 SQL view definitions |
| `crates/runtime/read_models` | transient client-side state |
| `crates/runtime/world_projection` | generic projection host for app/world mappings |
| `crates/views/scene3d` | generic 3D scene host and actor primitives |
| `crates/views/graph_view` | generic graph and lineage renderer |
| `crates/views/chat_thread` | transcript, composer, tool-call, and clarification surfaces |
| `crates/views/timeline_view` | trace, replay, and event-history view |
| `crates/views/telemetry_view` | minimap, heatmap, and telemetry overlays |
| `crates/views/list_detail` | feeds, inspectors, cards, tables |
| `crates/ui/panel_shells` | egui panel framing and shell contracts |
| `crates/ui/layout_runtime` | panel registry, docking, modal/layout runtime |
| `crates/ui/theme_tokens` | theme, spacing, typography, icon contracts |
| `apps/deer_replay` | replay and timeline proving ground |
| `apps/deer_graph_lab` | graph proving ground |
| `apps/deer_chat_lab` | live chat and artifact pipeline proving ground |
| `apps/deer_design` | layout and composition proving ground |
| `apps/deer_gui` | first playable hybrid composition |

We do not need all crates immediately, but this is the target decomposition.

## Layer Responsibilities

### Raw Sources

Purpose:

- ingest live backend data, fixture files, replay recordings, static samples

Owns:

- transport lifecycle
- deserialization
- reconnect and retry
- thread creation and run-stream bridging
- uploads and artifact gateway access

Must not know:

- Bevy
- egui
- panel layout
- game metaphors

Examples for the chat pipeline:

- LangGraph run-stream source
- embedded Python client stream wrapper
- thread uploads gateway
- thread artifacts gateway

### Normalizers

Purpose:

- convert raw backend schema into shared A/B storage-row families, with normalizers responsible for the shared `L2+` rows that downstream `C:L2` views query

Owns:

- schema mapping
- version normalization
- semantic conversion
- validation and coercion
- translation of stream events into shared run, message, task, artifact, and governance rows

Must not know:

- rendering
- view widgets
- camera or layout state

### A/B Storage Row Families

Purpose:

- represent stable storage truth used by all apps
- A hierarchy: observed execution/orchestration truth such as lifecycle, metrics, decisions, tool calls, status pings, intents, exclusions, replay checkpoints, and backpressure
- B hierarchy: content, semantic, representation, governance, and knowledge-bearing rows linked to that truth across the relevant levels
- physically exists in storage across L0-L6, with the shared consumer-facing A/B row contracts taking shape at `L2+`

Owns:

- identities
- relationships
- histories
- invariants

Chat-oriented A/B storage rows should include at least:

- thread/session record
- message record
- tool-call record
- clarification record
- artifact record
- run-progress or subtask record

Must not know:

- transport
- visual layout
- animation state
- specific app workflows

### C:L2 Presentation Views

Purpose:

- define `C:L2` SQL views and materialized views over shared A/B storage rows that consumers query

Owns:

- SQL view definitions / materialized views over A:L2/L3/L4/L5/L6 and B:L2/L3/L4/L5/L6
- aggregation
- grouping
- filtering
- metric shaping
- semantic decoration for views

Chat-oriented `C:L2` SQL views should include at least:

- transcript query view
- composer state query view
- artifact shelf query view
- run status query view
- task progress query view

Must not know:

- raw backend schema
- egui details
- renderer internals

Key rule: consumers (apps/tools/games) NEVER query L0/L1 directly. They query C:L2 views only.

### Read Models

Purpose:

- hold transient client state not owned by the backend

Examples:

- selection and hover
- camera focus
- smoothing and interpolation
- alert ordering
- replay cursor
- open tabs, modals, layout instance state
- draft prompt text and attachment queue
- stream connection state and optimistic send state
- selected artifact and artifact panel state
- clarification pending state

Authority rule:

- backend owns persisted truth
- client owns transient experience state

### World Projection Layer

Purpose:

- map backing `C:L2` view state into composition-specific world semantics without
  polluting reusable chat or view modules
- world projection is a downstream C-side consumer layer, not a separate truth layer

Owns:

- projection rules from backing `C:L2` views into world objects
- stable IDs linking transcript state, artifacts, alerts, and world markers
- app-facing semantic grouping for macro and micro views

Examples:

- thread session -> command channel or dialogue anchor
- run progress -> task-force or squad status
- clarification request -> intervention opportunity
- artifact record -> world item, codex entry, or unlock marker

Rule:

- reusable modules stay generic
- world metaphor mapping lives in projection rules at the runtime/app boundary
- each shell/app defines its own required C:L2 views within the same presentation hierarchy

## Layering Rule

No reusable component may jump layers.

Bad:

- `GraphView` rendering a raw backend response
- panel shell normalizing transport payloads
- app composition patching domain gaps with ad hoc UI structs

Good:

- raw source -> A/B storage normalizer -> C:L2 SQL views -> panel -> app
- raw source -> A/B storage normalizer -> C:L2 SQL views -> world projection objects -> scene/hud derivation -> app
