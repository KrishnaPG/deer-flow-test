# Modular Toolkit Architecture Design

**Date**: 2026-03-31
**Project**: `apps/deer_gui` and supporting toolkit/apps
**Goal**: Build a reusable toolkit for data-driven game views, authoring tools, and hybrid RTS/RPG experiences, with `deer_gui` as one composition target rather than the monolith.

## 1. Architectural Position

The primary product is not a single game app.

The primary product is a **toolkit of reusable modules** for:
- data ingestion and normalization
- canonical simulation/read models
- reusable 3D and 2D views
- panel/layout composition
- authoring and inspection tools

`deer_gui` is then one composed application built from these modules.

## 2. Design Priorities

Priority order:

1. reusable view toolkit
2. foundational data pipeline
3. authoring tools
4. game/app composition
5. vertical slice proving ground

This is still agile, but it is toolkit-first agile.

## 3. Core Principles

### 3.1 Strict Layering

Every feature must pass through these layers:

`raw backend data -> canonical domain model -> derived view model -> reusable view/panel -> app composition`

No reusable view may consume raw backend schema directly.

### 3.2 Reusable Modules Must Be App-Agnostic

Reusable modules must not know about:
- `deer_gui`
- specific backend transports like NATS or LiveKit
- specific game fantasy (RTS vs RPG)
- specific layout preset names

They may only know:
- typed contracts
- generic commands/events
- rendering/view responsibilities

### 3.3 Authoring Tools Are First-Class Clients

Authoring tools must use the same core modules as the runtime app.

That means:
- the layout editor consumes the same panel registry/runtime as the game
- the graph inspector consumes the same graph view as the game
- the replay browser consumes the same timeline/replay modules as the game

### 3.4 TDD By Contract

Each reusable module must be testable in isolation through:
- input fixtures
- typed contract assertions
- golden/snapshot output tests where useful
- minimal integration harnesses

## 4. Recommended Workspace Shape

Recommended high-level workspace layout:

| Path | Purpose |
| --- | --- |
| `crates/foundation/contracts` | shared interfaces, traits, DTOs, command/event contracts |
| `crates/foundation/domain` | canonical domain model |
| `crates/foundation/replay` | replay model, fixture loading, event history |
| `crates/pipeline/raw_sources` | backend/file/live source adapters |
| `crates/pipeline/normalizers` | raw schema -> canonical domain transforms |
| `crates/pipeline/derivations` | canonical domain -> typed view models |
| `crates/runtime/read_models` | transient client state, smoothing, selection, alert ranking |
| `crates/views/scene3d` | reusable 3D scene host and actor rendering primitives |
| `crates/views/graph_view` | generic network/knowledge graph renderer |
| `crates/views/timeline_view` | timeline, trace, replay visualization |
| `crates/views/telemetry_view` | minimap, heatmap, telemetry overlays |
| `crates/views/list_detail` | event feeds, inspectors, tables, card stacks |
| `crates/ui/panel_shells` | reusable egui panel shells and contracts |
| `crates/ui/layout_runtime` | docking/layout runtime, panel registry, modal runtime |
| `crates/ui/theme_tokens` | theme, spacing, typography, icon contracts |
| `apps/deer_gui` | composed hybrid RTS/RPG app |
| `apps/deer_design` | authoring and experimentation tool |
| `apps/deer_replay` | replay/trace browser |
| `apps/deer_graph_lab` | graph/knowledge view sandbox |

We do not need to create all crates immediately, but this should be the target decomposition.

## 5. Layered Data Pipeline

### 5.1 Raw Sources Layer

Purpose:
- ingest live backend data, fixture files, replay recordings, static samples

Responsibilities:
- transport concerns
- deserialization
- source lifecycle
- reconnect/retry logic

Must not know:
- Bevy
- egui
- panel layout
- game metaphors

Example contracts:
- `RawEventSource`
- `RawSnapshotSource`
- `ReplaySource`

Tests:
- source lifecycle tests
- deserialization tests
- replay playback tests

### 5.2 Normalizer Layer

Purpose:
- convert raw backend schema into canonical domain objects/events

Responsibilities:
- schema mapping
- version normalization
- semantic conversion
- validation and coercion

Must not know:
- rendering
- egui widgets
- camera/layout state

Example outputs:
- `AgentRecord`
- `SessionRecord`
- `ArtifactRecord`
- `TraceRecord`
- `KnowledgeNode`

Tests:
- fixture-in / canonical-out tests
- contract tests for schema upgrades

### 5.3 Canonical Domain Layer

Purpose:
- represent stable internal truth used by all apps

Responsibilities:
- identities
- relationships
- histories
- domain invariants

Must not know:
- backend transport
- visual layout
- animation state
- specific app workflows

Tests:
- invariant tests
- merging/update tests
- relationship integrity tests

### 5.4 Derivation Layer

Purpose:
- transform canonical domain state into typed view models for reusable views

Responsibilities:
- aggregation
- filtering
- grouping
- metric shaping
- semantic decoration for views

Must not know:
- raw backend schema
- egui implementation details
- Bevy renderer internals

Example view models:
- `TimelineVm`
- `GraphVm`
- `TelemetryMapVm`
- `EventFeedVm`
- `DetailCardVm`
- `ActorCloudVm`

Tests:
- deterministic fixture-to-view-model tests
- snapshot/golden tests

### 5.5 Read-Model Layer

Purpose:
- hold transient client state not owned by backend

Examples:
- selection
- hover
- camera focus
- interpolation/smoothing
- alert ordering
- replay cursor
- open tabs
- modal state
- layout instance state

Rule:
- backend remains authoritative for persisted truth
- client is authoritative for transient derived experience

Tests:
- reducer/state-machine tests
- replay cursor tests
- smoothing and decay logic tests

## 6. Reusable View Toolkit

### 6.1 Scene Host

Purpose:
- generic Bevy 3D host for world actors, camera, overlays, and interaction hooks

Input:
- `SceneViewModel`

Output:
- renderable scene
- selection events
- camera/focus events

Extension points:
- actor glyph renderer
- material provider
- interaction mapper

Must not know:
- agents specifically
- backend event sources

### 6.2 Actor Cloud View

Purpose:
- render many entities as swarms, units, nodes, or grouped markers

Use cases:
- agent fleets
- queue workers
- concurrent task bursts

Input:
- `ActorCloudVm`

### 6.3 Graph View

Purpose:
- reusable node/edge renderer for knowledge graphs, dependencies, lineage, and sector maps

Input:
- `GraphVm`

Features:
- zoom
- pan
- cluster collapse
- selection
- path highlighting

### 6.4 Timeline / Replay View

Purpose:
- visualize traces, meetings, playbacks, and event histories

Input:
- `TimelineVm`

Features:
- scrub
- bookmarks
- branch markers
- anomaly markers

### 6.5 Telemetry / Map View

Purpose:
- minimap, heatmap, alert overlays, spatial summaries

Input:
- `TelemetryMapVm`

### 6.6 Detail / Card / Inspector Views

Purpose:
- reusable structured inspection surfaces for agents, artifacts, meetings, references, insights

Input:
- `DetailCardVm`
- `InspectorVm`

### 6.7 Theater / Media View

Purpose:
- play audio/video/replay with synchronized metadata

Input:
- `MediaVm`
- `TranscriptVm`

## 7. Reusable Panel And Layout Toolkit

### 7.1 Panel Shells

Purpose:
- reusable egui shells that host arbitrary view modules

Examples:
- command console shell
- event feed shell
- inspector shell
- graph panel shell
- theater panel shell

Contract shape:
- shell owns framing, header, toolbar, collapse, actions
- inner view owns content rendering from typed VM

### 7.2 Panel Registry

Purpose:
- register reusable panels and their typed input adapters

Responsibilities:
- panel identifiers
- panel capabilities
- toolbar actions
- serialization hooks for layout/runtime

### 7.3 Layout Runtime

Purpose:
- place panels, save layouts, embed 3D hosts, manage drill-downs and modals

Requirements:
- data-driven layout
- strongly typed panel APIs
- reusable outside `deer_gui`

### 7.4 View Host Bridge

Purpose:
- embed 3D or special rendering surfaces inside panel-managed layouts

This is a critical reusable module.

It allows:
- graph view in one panel
- scene view in another
- minimap in a floating panel
- replay theater in modal or docked state

## 8. Authoring Tool Apps

Authoring tools should be peer apps built from the same toolkit.

### 8.1 `deer_design`

Purpose:
- layout editing
- panel composition
- scene/view composition
- theme experimentation
- simulation fixtures

### 8.2 `deer_replay`

Purpose:
- inspect traces, meetings, sessions, playback paths

### 8.3 `deer_graph_lab`

Purpose:
- experiment with graph rendering, clustering, lineage, and knowledge layouts

### 8.4 Future Tooling

- asset/icon browser
- telemetry lab
- actor cloud sandbox
- panel contract playground

## 9. API Contract Style

Recommended contract pattern:

- raw-source traits in `foundation/contracts`
- domain records in `foundation/domain`
- typed VM structs in `pipeline/derivations`
- view traits that consume VMs, not domain or raw source

Example direction:

- good: `GraphView::render(&GraphVm, &GraphInteractionState)`
- bad: `GraphView::render(&RawKnowledgeGraphResponse)`

Every reusable component must clearly specify:
- accepted input type
- emitted events
- allowed commands
- persistence expectations
- threading/runtime assumptions

## 10. TDD Strategy

### 10.1 Test Pyramid

| Module Type | Primary Test Type |
| --- | --- |
| contracts/domain | pure unit tests |
| raw sources | fixture and protocol tests |
| normalizers | fixture transform tests |
| derivations | golden/snapshot tests |
| read-models | reducer/state-machine tests |
| reusable views | behavior tests and screenshot/snapshot where feasible |
| panel/layout runtime | interaction tests |
| app composition | thin integration tests |

### 10.2 TDD Order

For each reusable module:

1. define contract
2. write failing fixture/contract test
3. implement minimal version
4. add interaction test
5. integrate into one tool/app

### 10.3 Anti-Patterns

Avoid:
- views reading raw backend payloads
- panels owning normalization logic
- backend adapters importing Bevy UI code
- app-specific enums leaking into reusable crates
- authoring tools forking runtime components
- building one giant “ui_core” dumping ground

## 11. Build Order

Recommended practical order:

1. `foundation/contracts`
2. `foundation/domain`
3. `foundation/replay`
4. `pipeline/normalizers`
5. `pipeline/derivations`
6. `runtime/read_models`
7. one reusable view each:
   - timeline view
   - graph view
   - event feed / detail panel shell
8. `ui/layout_runtime`
9. `apps/deer_replay`
10. `apps/deer_design`
11. `apps/deer_gui` composition

This order is intentionally biased toward toolkit and tools first, while still enabling the game/app to emerge by composition.

## 12. First Reusable Modules To Prove

The best first set is:

1. **timeline/replay module**
2. **graph view module**
3. **event feed + detail inspector shells**
4. **layout runtime with panel registry**
5. **actor cloud / telemetry map view**

These five modules are broadly reusable and give immediate visible payoff.

## 13. Recommendation

Build this system as a **modular data-to-view toolkit**.

Let the apps prove the toolkit:
- `deer_replay` proves replay + timeline + detail views
- `deer_graph_lab` proves graph rendering + clustering + inspection
- `deer_design` proves layout runtime + panel/view hosting + authoring flow
- `deer_gui` proves hybrid RTS/RPG composition

That is the cleanest path to:
- reusable lego-brick modules
- low coupling
- standalone tools
- strong API contracts
- long-term experimentation power

It also preserves the agile requirement because each app becomes a narrow proving ground for a subset of the toolkit rather than a giant monolith.
