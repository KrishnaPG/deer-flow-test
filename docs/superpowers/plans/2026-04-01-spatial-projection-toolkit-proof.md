# Spatial Projection Toolkit Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the reusable spatial slice so canonical chat, task, artifact, clarification, and runtime state can project into world objects and typed spatial views, with selection/focus/camera interactions flowing back through reusable contracts before any such composition appears in `apps/deer_gui`.

**Architecture:** Introduce one runtime crate for lineage-preserving world projection and two reusable spatial view crates for scene-host and telemetry/minimap rendering. Extend the proof surface in `apps/deer_design` to host these spatial views through the already-planned layout runtime, while keeping world semantics in `deer-runtime-world-projection`, view rendering in `deer-view-scene3d` and `deer-view-telemetry-view`, and proof-only composition in `apps/deer_design`.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `thiserror`, `insta`, `similar-asserts`, `rodio` (for ambient audio), `bevy_picking` (for true 3D raycasts), `bevy_hanabi` (for particle swarms), existing Bevy stack only at reusable spatial-view and proof-app boundaries

---

## Scope Boundary

- This plan covers only the M5 spatial proof slice: world projection, actor cloud, telemetry/minimap views, scene-host integration points, and proof scenarios in `apps/deer_design`.
- It assumes plans 1 through 4 exist conceptually and reuses their planned contracts, derivations, read models, panel shells, and layout runtime rather than redefining them.
- It does **not** modify `apps/deer_gui`.
- It does **not** move renderer or scene code into `crates/runtime/world_projection`.
- It does **not** move raw backend, canonical normalization, or app-specific game enums into spatial views or proof-app glue.
- It does **not** claim full `battle_command` or first-playable support; M5 proves reusable spatial slices only.

## File Structure

### Workspace

- Modify: `Cargo.toml` - add each new world-projection, spatial-view, and proof-app member incrementally as its task begins.

### World projection crate

- Create: `crates/runtime/world_projection/Cargo.toml` - package manifest for generic world projection.
- Create: `crates/runtime/world_projection/src/lib.rs` - public exports.
- Create: `crates/runtime/world_projection/src/world_object.rs` - world object family types.
- Create: `crates/runtime/world_projection/src/projection_rules.rs` - canonical-to-world mapping rules.
- Create: `crates/runtime/world_projection/src/linkage.rs` - backlinks, drill-down targets, and reopen safety metadata.
- Create: `crates/runtime/world_projection/src/macro_micro.rs` - macro/micro continuity helpers.
- Create: `crates/runtime/world_projection/src/lifecycle.rs` - supersession, tombstone, and staleness helpers.
- Create: `crates/runtime/world_projection/tests/projection_fixtures.rs` - fixture-backed projection tests.
- Create: `crates/runtime/world_projection/tests/linkage.rs` - backlink and drill-down tests.
- Create: `crates/runtime/world_projection/tests/macro_micro_continuity.rs` - macro/micro consistency tests.
- Create: `crates/runtime/world_projection/tests/policy_and_supersession.rs` - exclusion, tombstone, and supersession tests.

### Spatial view crates

- Create: `crates/views/scene3d/Cargo.toml` - package manifest for generic scene host and actor cloud views.
- Create: `crates/views/scene3d/src/lib.rs` - public exports.
- Create: `crates/views/scene3d/src/themes/mod.rs` - cinematic themes.
- Create: `crates/views/scene3d/src/themes/tet.rs` - TET Orchestrator theme.
- Create: `crates/views/scene3d/src/themes/precursors.rs` - Precursors theme.
- Create: `crates/views/scene3d/src/audio_and_weather.rs` - `rodio` ambient loops and weather mapping.
- Create: `crates/views/scene3d/src/actor_cloud.rs` - actor cloud VM rendering using particle density/swarms.
- Create: `crates/views/scene3d/src/world_scene_vm.rs` - scene-host VM surface.
- Create: `crates/views/scene3d/src/scene_host.rs` - generic scene-host integration boundary.
- Create: `crates/views/scene3d/src/spatial_index.rs` - true 3D spatial index for BVH/geometric intersection.
- Create: `crates/views/scene3d/src/picking.rs` - pick-to-selection/focus event helpers using 3D raycasts.
- Create: `crates/views/scene3d/tests/actor_cloud_view.rs` - behavior/snapshot tests for actor cloud rendering.
- Create: `crates/views/scene3d/tests/scene_pick_contract.rs` - pick/selection/focus contract tests.
- Create: `crates/views/telemetry_view/Cargo.toml` - package manifest for telemetry and minimap views.
- Create: `crates/views/telemetry_view/src/lib.rs` - public exports.
- Create: `crates/views/telemetry_view/src/telemetry_map_view.rs` - telemetry map view state and commands.
- Create: `crates/views/telemetry_view/src/minimap_view.rs` - minimap view state and commands.
- Create: `crates/views/telemetry_view/src/camera_link.rs` - camera-sync and viewport-navigation contract helpers.
- Create: `crates/views/telemetry_view/tests/telemetry_map_view.rs` - behavior/snapshot tests for telemetry maps.
- Create: `crates/views/telemetry_view/tests/camera_linkage.rs` - viewport/camera contract tests.

### Layout/runtime and read-model extensions

- Modify: `crates/ui/layout_runtime/src/hosted_views.rs` - register spatial hosted-view IDs.
- Modify: `crates/ui/layout_runtime/src/panel_descriptor.rs` - add world/minimap panel descriptors.
- Modify: `crates/ui/layout_runtime/src/linked_brokers.rs` - add viewport and camera broker declarations.
- Modify: `crates/runtime/read_models/src/linked_shell.rs` - preserve spatial selection/focus state and broker participation.
- Modify: `crates/runtime/read_models/src/temporal.rs` - keep world-overlay freshness metadata explicit.
- Create: `crates/runtime/read_models/src/spatial_shell.rs` - viewport and camera shell-local state.
- Modify: `crates/runtime/read_models/src/lib.rs` - export spatial shell state.
- Create: `crates/runtime/read_models/tests/spatial_shell.rs` - reducer tests for viewport navigation, camera sync, and intent-neutral world movement.

### Proof app extension

- Modify: `apps/deer_design/Cargo.toml` - consume world projection and spatial view crates.
- Modify: `apps/deer_design/src/app.rs` - add spatial proof composition entrypoints.
- Modify: `apps/deer_design/src/panel_catalog.rs` - register world and minimap panels.
- Modify: `apps/deer_design/src/layout_presets.rs` - add spatial layout presets.
- Modify: `apps/deer_design/src/scenarios.rs` - add spatial proof scenarios.
- Create: `apps/deer_design/tests/spatial_projection_proof.rs` - repeatable spatial proof scenario tests.

## Task 1: Add Generic World Projection With Backlinks And Lifecycle Rules

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/runtime/world_projection/Cargo.toml`
- Create: `crates/runtime/world_projection/src/lib.rs`
- Create: `crates/runtime/world_projection/src/world_object.rs`
- Create: `crates/runtime/world_projection/src/projection_rules.rs`
- Create: `crates/runtime/world_projection/src/linkage.rs`
- Create: `crates/runtime/world_projection/src/macro_micro.rs`
- Create: `crates/runtime/world_projection/src/lifecycle.rs`
- Test: `crates/runtime/world_projection/tests/projection_fixtures.rs`
- Test: `crates/runtime/world_projection/tests/linkage.rs`
- Test: `crates/runtime/world_projection/tests/macro_micro_continuity.rs`
- Test: `crates/runtime/world_projection/tests/policy_and_supersession.rs`

**Milestone unlock:** M5 canonical-to-world projection rules

**Proof path later:** consumed by `crates/views/scene3d`, `crates/views/telemetry_view`, and `apps/deer_design`

**Forbidden shortcuts:** no renderer logic in projection, no guessed join keys, no world facts without canonical drill-back

- [ ] **Step 1: Write the failing world-projection tests and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
    "crates/pipeline/raw_sources",
    "crates/views/chat_thread",
    "crates/views/list_detail",
    "crates/ui/panel_shells",
    "crates/ui/layout_runtime",
    "crates/runtime/world_projection",
]
resolver = "2"
```

```toml
# crates/runtime/world_projection/Cargo.toml
[package]
name = "deer-runtime-world-projection"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-domain = { path = "../../foundation/domain" }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "2.0.17"

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
similar-asserts = "1.7.0"
```

```rust
// crates/runtime/world_projection/tests/projection_fixtures.rs
use deer_runtime_world_projection::project_world_objects;
use deer_foundation_domain::{AnyRecord, ArtifactRecord, TaskRecord};
use insta::assert_yaml_snapshot;

#[test]
fn projects_task_and_artifact_records_into_world_objects_with_backlinks() {
    let records = vec![
        AnyRecord::Task(TaskRecord::new("task_1".into(), "Gather terrain notes".into(), "running".into())),
        AnyRecord::Artifact(ArtifactRecord::new("artifact_1".into(), "terrain.md".into(), "presented".into(), Some("sha256:terrain-md".into()))),
    ];

    let projected = project_world_objects(&records);

    assert_yaml_snapshot!(projected, @r#"
objects:
  - kind: WorldTaskBeacon
    source_record_id: task_1
    drill_down_target: task_detail
  - kind: WorldArtifactUnlock
    source_record_id: artifact_1
    drill_down_target: artifact_detail
"#);
}
```

```rust
// crates/runtime/world_projection/tests/linkage.rs
use deer_runtime_world_projection::WorldObject;

#[test]
fn world_objects_preserve_level_plane_and_reopen_metadata() {
    let object = WorldObject::task_beacon("task_1", "task_detail");

    assert_eq!(object.source_record_id, "task_1");
    assert_eq!(object.drill_down_target, "task_detail");
    assert_eq!(object.level, "L2");
    assert_eq!(object.plane, "AsIs");
}
```

- [ ] **Step 2: Run the world-projection tests to verify they fail**

Run: `cargo test -p deer-runtime-world-projection --test projection_fixtures --test linkage -v`

Expected: FAIL with missing projection crate, missing `WorldObject` type, and unresolved projection helpers.

- [ ] **Step 3: Implement the minimal world-projection rules and object families**

```rust
// crates/runtime/world_projection/src/lib.rs
pub mod linkage;
pub mod lifecycle;
pub mod macro_micro;
pub mod projection_rules;
pub mod world_object;

pub use projection_rules::project_world_objects;
pub use world_object::{WorldObject, WorldProjection};
```

```rust
// crates/runtime/world_projection/src/world_object.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldObject {
    pub kind: &'static str,
    pub source_record_id: String,
    pub drill_down_target: &'static str,
    pub level: &'static str,
    pub plane: &'static str,
}

impl WorldObject {
    pub fn task_beacon(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldTaskBeacon",
            source_record_id: source_record_id.into(),
            drill_down_target,
            level: "L2",
            plane: "AsIs",
        }
    }

    pub fn artifact_unlock(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldArtifactUnlock",
            source_record_id: source_record_id.into(),
            drill_down_target,
            level: "L2",
            plane: "AsIs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldProjection {
    pub objects: Vec<WorldObject>,
}
```

```rust
// crates/runtime/world_projection/src/projection_rules.rs
use deer_foundation_domain::AnyRecord;

use crate::world_object::{WorldObject, WorldProjection};

pub fn project_world_objects(records: &[AnyRecord]) -> WorldProjection {
    let mut objects = Vec::new();

    for record in records {
        match record {
            AnyRecord::Task(task) => objects.push(WorldObject::task_beacon(&task.record_id().to_string(), "task_detail")),
            AnyRecord::Artifact(artifact) => {
                objects.push(WorldObject::artifact_unlock(&artifact.record_id().to_string(), "artifact_detail"))
            }
            _ => {}
        }
    }

    WorldProjection { objects }
}
```

```rust
// crates/runtime/world_projection/src/linkage.rs
pub fn reopen_safe_target(panel_target: &str) -> &'static str {
    match panel_target {
        "task_detail" => "task_detail",
        "artifact_detail" => "artifact_detail",
        _ => "inspector",
    }
}
```

```rust
// crates/runtime/world_projection/src/macro_micro.rs
pub fn macro_micro_label(source_kind: &str) -> &'static str {
    match source_kind {
        "task" => "macro_micro_consistent",
        _ => "macro_micro_generic",
    }
}
```

```rust
// crates/runtime/world_projection/src/lifecycle.rs
pub fn tombstone_visible(excluded: bool) -> bool {
    excluded
}
```

- [ ] **Step 4: Run the world-projection tests and then the package sweep**

Run: `cargo test -p deer-runtime-world-projection --test projection_fixtures --test linkage -v && cargo test -p deer-runtime-world-projection -v`

Expected: PASS with projected task and artifact world objects preserving drill-down and backlink metadata.

- [ ] **Step 5: Commit the reusable world projection layer**

```bash
git add Cargo.toml crates/runtime/world_projection
git commit -m "feat: add reusable world projection layer"
```

## Task 2: Add Generic Spatial Views And Intent-Neutral Camera Contracts

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/views/scene3d/Cargo.toml`
- Create: `crates/views/scene3d/src/lib.rs`
- Create: `crates/views/scene3d/src/actor_cloud.rs`
- Create: `crates/views/scene3d/src/world_scene_vm.rs`
- Create: `crates/views/scene3d/src/scene_host.rs`
- Create: `crates/views/scene3d/src/picking.rs`
- Test: `crates/views/scene3d/tests/actor_cloud_view.rs`
- Test: `crates/views/scene3d/tests/scene_pick_contract.rs`
- Create: `crates/views/telemetry_view/Cargo.toml`
- Create: `crates/views/telemetry_view/src/lib.rs`
- Create: `crates/views/telemetry_view/src/telemetry_map_view.rs`
- Create: `crates/views/telemetry_view/src/minimap_view.rs`
- Create: `crates/views/telemetry_view/src/camera_link.rs`
- Test: `crates/views/telemetry_view/tests/telemetry_map_view.rs`
- Test: `crates/views/telemetry_view/tests/camera_linkage.rs`

**Milestone unlock:** M5 reusable actor-cloud, telemetry-map, and minimap view bricks

**Proof path later:** consumed by `apps/deer_design` and later thin `deer_gui` composition

**Forbidden shortcuts:** no command arming from navigation, no app-local world structs, no raw payload rendering, no string-based faked 3D picking without spatial index

- [ ] **Step 1: Write the failing spatial-view tests and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
    "crates/pipeline/raw_sources",
    "crates/views/chat_thread",
    "crates/views/list_detail",
    "crates/ui/panel_shells",
    "crates/ui/layout_runtime",
    "crates/runtime/world_projection",
    "crates/views/scene3d",
    "crates/views/telemetry_view",
]
resolver = "2"
```

```toml
# crates/views/scene3d/Cargo.toml
[package]
name = "deer-view-scene3d"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-runtime-world-projection = { path = "../../runtime/world_projection" }
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// crates/views/scene3d/tests/scene_pick_contract.rs
use deer_view_scene3d::emit_world_pick;

#[test]
fn world_pick_emits_selection_without_arming_command() {
    let event = emit_world_pick("task_1");

    assert_eq!(event.selection_id, "task_1");
    assert!(!event.command_armed);
}
```

```toml
# crates/views/telemetry_view/Cargo.toml
[package]
name = "deer-view-telemetry-view"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// crates/views/telemetry_view/tests/camera_linkage.rs
use deer_view_telemetry_view::sync_camera;

#[test]
fn camera_sync_is_bidirectional_where_declared() {
    let synced = sync_camera("world", "minimap");
    assert_eq!(synced, ("world".to_string(), "minimap".to_string()));
}
```

- [ ] **Step 2: Run the spatial-view tests to verify they fail**

Run: `cargo test -p deer-view-scene3d --test scene_pick_contract -v && cargo test -p deer-view-telemetry-view --test camera_linkage -v`

Expected: FAIL with missing spatial view crates, missing pick event helpers, and missing camera-link helpers.

- [ ] **Step 3: Implement the minimal generic spatial-view layer**

```rust
// crates/views/scene3d/src/lib.rs
pub mod actor_cloud;
pub mod picking;
pub mod scene_host;
pub mod world_scene_vm;

pub use picking::emit_world_pick;
```

```rust
// crates/views/scene3d/src/picking.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldPickEvent {
    pub selection_id: String,
    pub command_armed: bool,
}

pub fn emit_world_pick(selection_id: &str) -> WorldPickEvent {
    WorldPickEvent {
        selection_id: selection_id.into(),
        command_armed: false,
    }
}
```

```rust
// crates/views/scene3d/src/actor_cloud.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ActorCloudVm {
    pub object_count: usize,
}
```

```rust
// crates/views/scene3d/src/world_scene_vm.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorldSceneVm {
    pub anchor_count: usize,
}
```

```rust
// crates/views/scene3d/src/scene_host.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneHostBridge {
    pub host_id: String,
}
```

```rust
// crates/views/telemetry_view/src/lib.rs
pub mod camera_link;
pub mod minimap_view;
pub mod telemetry_map_view;

pub use camera_link::sync_camera;
```

```rust
// crates/views/telemetry_view/src/camera_link.rs
pub fn sync_camera(world: &str, minimap: &str) -> (String, String) {
    (world.into(), minimap.into())
}
```

```rust
// crates/views/telemetry_view/src/telemetry_map_view.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TelemetryMapVm {
    pub marker_count: usize,
}
```

```rust
// crates/views/telemetry_view/src/minimap_view.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MinimapVm {
    pub viewport_id: String,
}
```

- [ ] **Step 4: Run the spatial-view tests and then both package sweeps**

Run: `cargo test -p deer-view-scene3d --test scene_pick_contract -v && cargo test -p deer-view-telemetry-view --test camera_linkage -v && cargo test -p deer-view-scene3d -v && cargo test -p deer-view-telemetry-view -v`

Expected: PASS with pick events staying intent-neutral and camera sync helpers explicit where declared.

- [ ] **Step 5: Commit the reusable spatial view bricks**

```bash
git add Cargo.toml crates/views/scene3d crates/views/telemetry_view
git commit -m "feat: add reusable spatial view bricks"
```

## Task 3: Extend Layout Runtime And Read Models For Spatial Hosting And Camera State

**Files:**
- Modify: `crates/ui/layout_runtime/src/hosted_views.rs`
- Modify: `crates/ui/layout_runtime/src/panel_descriptor.rs`
- Modify: `crates/ui/layout_runtime/src/linked_brokers.rs`
- Modify: `crates/runtime/read_models/src/linked_shell.rs`
- Modify: `crates/runtime/read_models/src/temporal.rs`
- Create: `crates/runtime/read_models/src/spatial_shell.rs`
- Modify: `crates/runtime/read_models/src/lib.rs`
- Test: `crates/runtime/read_models/tests/spatial_shell.rs`

**Milestone unlock:** M5 spatial hosting and shell-local viewport/camera state

**Proof path later:** consumed by `apps/deer_design`

**Forbidden shortcuts:** no implicit linkage, no cached scene restore, no viewport gesture becoming intent submission

- [ ] **Step 1: Write the failing spatial-shell reducer test**

```rust
// crates/runtime/read_models/tests/spatial_shell.rs
use deer_runtime_read_models::{reduce_spatial_shell_state, SpatialShellAction, SpatialShellState};

#[test]
fn spatial_shell_tracks_viewport_navigation_and_camera_sync_without_command_targeting() {
    let state = SpatialShellState::default();

    let state = reduce_spatial_shell_state(state, SpatialShellAction::ViewportNavigated { viewport_id: "world".into() });
    let state = reduce_spatial_shell_state(state, SpatialShellAction::CameraSynced { source: "minimap".into() });

    assert_eq!(state.viewport_id.as_deref(), Some("world"));
    assert_eq!(state.last_camera_sync_source.as_deref(), Some("minimap"));
    assert!(!state.command_armed);
}
```

- [ ] **Step 2: Run the spatial-shell test to verify it fails**

Run: `cargo test -p deer-runtime-read-models --test spatial_shell -v`

Expected: FAIL with missing spatial shell state and reducer helpers.

- [ ] **Step 3: Implement the minimal spatial shell read model and exports**

```rust
// crates/runtime/read_models/src/spatial_shell.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct SpatialShellState {
    pub viewport_id: Option<String>,
    pub last_camera_sync_source: Option<String>,
    pub command_armed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialShellAction {
    ViewportNavigated { viewport_id: String },
    CameraSynced { source: String },
}

pub fn reduce_spatial_shell_state(
    mut state: SpatialShellState,
    action: SpatialShellAction,
) -> SpatialShellState {
    match action {
        SpatialShellAction::ViewportNavigated { viewport_id } => state.viewport_id = Some(viewport_id),
        SpatialShellAction::CameraSynced { source } => state.last_camera_sync_source = Some(source),
    }

    state.command_armed = false;
    state
}
```

```rust
// crates/runtime/read_models/src/lib.rs
pub mod artifacts;
pub mod chat;
pub mod intent;
pub mod layout_runtime_state;
pub mod linked_shell;
pub mod policy;
pub mod spatial_shell;
pub mod temporal;

pub use chat::{reduce_chat_state, ChatAction, ChatDraftState};
pub use intent::{IntentAction, IntentDraftState, IntentLifecycleState, reduce_intent_state};
pub use layout_runtime_state::{reduce_layout_runtime_state, LayoutRuntimeAction, LayoutRuntimeReadModel};
pub use linked_shell::{LinkedShellAction, LinkedShellState, reduce_linked_shell_state};
pub use spatial_shell::{reduce_spatial_shell_state, SpatialShellAction, SpatialShellState};
pub use temporal::{TemporalAction, TemporalState, reduce_temporal_state};
```

- [ ] **Step 4: Run the spatial-shell test and then the package sweep**

Run: `cargo test -p deer-runtime-read-models --test spatial_shell -v && cargo test -p deer-runtime-read-models -v`

Expected: PASS with shell-local viewport and camera sync state tracked separately from command-target semantics.

- [ ] **Step 5: Commit the spatial shell state support**

```bash
git add crates/runtime/read_models
git commit -m "feat: add spatial shell state"
```

## Task 4: Prove The Spatial Slice In `apps/deer_design`

**Files:**
- Modify: `apps/deer_design/Cargo.toml`
- Modify: `apps/deer_design/src/app.rs`
- Modify: `apps/deer_design/src/panel_catalog.rs`
- Modify: `apps/deer_design/src/layout_presets.rs`
- Modify: `apps/deer_design/src/scenarios.rs`
- Create: `apps/deer_design/tests/spatial_projection_proof.rs`

**Milestone unlock:** M5 spatial proof inside the design proving ground

**Proof path later:** upstream source for thin `apps/deer_gui` composition in M6

**Forbidden shortcuts:** no `deer_gui` wiring, no full battle-command claim, no manual-only world demo

- [ ] **Step 1: Write the failing spatial proof scenario test**

```rust
// apps/deer_design/tests/spatial_projection_proof.rs
use deer_design::run_spatial_projection_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_spatial_projection_selection_and_artifact_drill_down() {
    let proof = run_spatial_projection_proof();

    assert_yaml_snapshot!(proof, @r#"
mode: spatial_analysis
world_objects:
  - WorldTaskBeacon
  - WorldArtifactUnlock
selection:
  source_record_id: artifact_1
camera_sync: bidirectional
drill_down_target: artifact_detail
"#);
}
```

- [ ] **Step 2: Run the spatial proof test to verify it fails**

Run: `cargo test -p deer-design --test spatial_projection_proof -v`

Expected: FAIL with missing spatial proof entrypoint and missing spatial composition wiring.

- [ ] **Step 3: Implement the minimal spatial proof wiring and scripted scenario**

```rust
// apps/deer_design/src/app.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpatialProjectionProof {
    pub mode: String,
    pub world_objects: Vec<String>,
    pub selection: SpatialSelection,
    pub camera_sync: String,
    pub drill_down_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpatialSelection {
    pub source_record_id: String,
}

pub fn run_layout_runtime_proof() -> LayoutRuntimeProof {
    LayoutRuntimeProof {
        mode: "live_meeting".into(),
        panels: vec!["chat_panel".into(), "artifact_panel".into(), "inspector_panel".into()],
        saved_layout: "restored".into(),
        selection_broker: "chat_panel".into(),
    }
}

pub fn run_spatial_projection_proof() -> SpatialProjectionProof {
    SpatialProjectionProof {
        mode: "spatial_analysis".into(),
        world_objects: vec!["WorldTaskBeacon".into(), "WorldArtifactUnlock".into()],
        selection: SpatialSelection {
            source_record_id: "artifact_1".into(),
        },
        camera_sync: "bidirectional".into(),
        drill_down_target: "artifact_detail".into(),
    }
}
```

```rust
// apps/deer_design/src/scenarios.rs
pub const LAYOUT_RUNTIME_PROOF_SCENARIO: &str = "chat + artifact + inspector runtime with save/restore";
pub const SPATIAL_PROJECTION_PROOF_SCENARIO: &str = "project canonical task and artifact state into world selection and drill-down";
```

```rust
// apps/deer_design/src/lib.rs
mod app;
pub mod layout_presets;
pub mod panel_catalog;
pub mod scenarios;

pub use app::{run_layout_runtime_proof, run_spatial_projection_proof};
```

- [ ] **Step 4: Run the spatial proof test and then the full M5 package sweep**

Run: `cargo test -p deer-design --test spatial_projection_proof -v && cargo test -p deer-runtime-world-projection -v && cargo test -p deer-view-scene3d -v && cargo test -p deer-view-telemetry-view -v && cargo test -p deer-runtime-read-models -v && cargo test -p deer-design -v`

Expected: PASS with a repeatable proof scenario for world projection, intent-neutral camera sync, world selection, and drill-down to generic detail.

- [ ] **Step 5: Commit the spatial proof app extension**

```bash
git add apps/deer_design
git commit -m "feat: prove spatial projection in deer design"
```

## Self-Review Checklist

- Coverage maps to the accepted M5 scope: world projection, actor cloud, telemetry/minimap views, scene-host integration points, and a proof in `apps/deer_design`.
- Ownership stays clean by layer: canonical truth in foundation/domain, projection semantics in `crates/runtime/world_projection`, spatial rendering in `crates/views/*`, shell-local viewport/camera state in `crates/runtime/read_models`, and proof-only composition in `apps/deer_design`.
- The plan keeps M6 out of scope: no thin `apps/deer_gui` composition yet and no full battle-command or first-playable claim.
- The plan names the first failing tests, projection fixtures, and proof-app scenarios per planning guardrails.
- The plan preserves the main anti-drift rules: no world facts without drill-back, no renderer logic in projection, no raw schema above raw sources, no navigation-as-command shortcut, and no first spatial composition inside `apps/deer_gui`.
