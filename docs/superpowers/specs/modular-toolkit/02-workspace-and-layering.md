# Design: Modular Toolkit - Workspace And Layering

**Date:** 2026-03-31
**Status:** Draft revision

## Target Workspace Shape

| Path | Purpose |
| --- | --- |
| `crates/foundation/contracts` | shared traits, DTOs, commands, events |
| `crates/foundation/domain` | canonical domain model |
| `crates/foundation/replay` | replay model, fixture loading, event history |
| `crates/pipeline/raw_sources` | backend, file, and replay source adapters |
| `crates/pipeline/normalizers` | raw schema to canonical domain transforms |
| `crates/pipeline/derivations` | canonical domain to typed view models |
| `crates/runtime/read_models` | transient client-side state |
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

- convert raw backend schema into canonical domain objects and events

Owns:

- schema mapping
- version normalization
- semantic conversion
- validation and coercion
- translation of stream events into canonical run, message, task, and artifact events

Must not know:

- rendering
- view widgets
- camera or layout state

### Canonical Domain

Purpose:

- represent stable internal truth used by all apps

Owns:

- identities
- relationships
- histories
- invariants

Chat-oriented canonical records should include at least:

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

### Derivations

Purpose:

- transform canonical domain state into typed view models

Owns:

- aggregation
- grouping
- filtering
- metric shaping
- semantic decoration for views

Chat-oriented view models should include at least:

- `TranscriptVm`
- `ComposerVm`
- `ArtifactShelfVm`
- `RunStatusVm`
- `TaskProgressVm`

Must not know:

- raw backend schema
- egui details
- renderer internals

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

## Layering Rule

No reusable component may jump layers.

Bad:

- `GraphView` rendering a raw backend response
- panel shell normalizing transport payloads
- app composition patching domain gaps with ad hoc UI structs

Good:

- raw source -> normalizer -> domain -> derivation -> view -> panel -> app
