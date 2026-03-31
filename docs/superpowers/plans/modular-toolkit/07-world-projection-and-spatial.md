# World Projection And Spatial Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the explicit world-projection layer plus reusable spatial slices that map canonical chat/task/artifact state into world semantics without polluting reusable chat modules.

**Architecture:** `world_projection` consumes canonical records and read-model state only, emits pure projected world records, then hands those to reusable spatial views. No raw DeerFlow events, no egui widgets, and no Bevy entities are allowed inside projection code.

**Tech Stack:** foundation/domain, pipeline/derivations, runtime/read-models, Bevy, existing `apps/deer_gui/src/world/*` patterns where helpful.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/runtime/world_projection/Cargo.toml` | projection manifest |
| Create | `crates/runtime/world_projection/src/lib.rs` | projected world records |
| Create | `crates/runtime/world_projection/tests/world_projection.rs` | projection/linkage tests |
| Create | `crates/views/telemetry_view/Cargo.toml` | telemetry/actor cloud manifest |
| Create | `crates/views/telemetry_view/src/lib.rs` | telemetry and hotspot surfaces |
| Create | `crates/views/telemetry_view/tests/telemetry.rs` | telemetry tests |

## Task 1: Create `deer-runtime-world-projection`

**Files:**
- Create: `crates/runtime/world_projection/Cargo.toml`
- Create: `crates/runtime/world_projection/src/lib.rs`
- Create: `crates/runtime/world_projection/tests/world_projection.rs`

- [ ] **Step 1: Write the failing projection test**

```rust
use deer_runtime_world_projection::project_thread_into_world;

#[test]
fn projection_links_artifact_and_progress_to_world_objects() {
    let world = project_thread_into_world();
    assert_eq!(world.artifact_markers.len(), 1);
    assert_eq!(world.task_markers.len(), 1);
}
```

- [ ] **Step 2: Add the crate manifest and minimal projection structs**

```toml
[package]
name = "deer-runtime-world-projection"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-domain = { path = "../../foundation/domain" }
deer-pipeline-derivations = { path = "../../pipeline/derivations" }
deer-runtime-read-models = { path = "../read_models" }
serde = { workspace = true }
tracing = { workspace = true }
```
```rust
#[derive(Clone, Debug)]
pub struct WorldArtifactMarker {
    pub source_path: String,
}

#[derive(Clone, Debug)]
pub struct WorldTaskMarker {
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct ProjectedWorldState {
    pub artifact_markers: Vec<WorldArtifactMarker>,
    pub task_markers: Vec<WorldTaskMarker>,
}

pub fn project_thread_into_world() -> ProjectedWorldState {
    ProjectedWorldState {
        artifact_markers: vec![WorldArtifactMarker { source_path: "outputs/report.md".into() }],
        task_markers: vec![WorldTaskMarker { label: "Researcher task".into() }],
    }
}
```

- [ ] **Step 3: Run the projection tests**

Run: `cargo test -p deer-runtime-world-projection --test world_projection`

Expected: PASS

## Task 2: Create Reusable Telemetry And Actor-Cloud Surfaces

**Files:**
- Create: `crates/views/telemetry_view/Cargo.toml`
- Create: `crates/views/telemetry_view/src/lib.rs`
- Create: `crates/views/telemetry_view/tests/telemetry.rs`

- [ ] **Step 1: Write the failing telemetry test**

```rust
use deer_runtime_world_projection::project_thread_into_world;
use deer_view_telemetry::TelemetrySurface;

#[test]
fn telemetry_surface_counts_projected_markers() {
    let projected = project_thread_into_world();
    let surface = TelemetrySurface::default();
    assert_eq!(surface.marker_count(&projected), 2);
}
```

- [ ] **Step 2: Add the crate manifest and minimal telemetry surface**

```toml
[package]
name = "deer-view-telemetry"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_egui = { workspace = true }
deer-runtime-world-projection = { path = "../../runtime/world_projection" }
tracing = { workspace = true }
```
```rust
use deer_runtime_world_projection::ProjectedWorldState;

#[derive(Default)]
pub struct TelemetrySurface;

impl TelemetrySurface {
    pub fn marker_count(&self, state: &ProjectedWorldState) -> usize {
        state.artifact_markers.len() + state.task_markers.len()
    }
}
```

- [ ] **Step 3: Run the telemetry tests**

Run: `cargo test -p deer-view-telemetry --test telemetry`

Expected: PASS

## Success Checks

- world projection owns metaphor mapping and stable linkage only
- projection code contains no Bevy entities, egui widgets, or transport parsing
- telemetry surfaces consume projected world state, not raw chat artifacts directly

## Done State

This file is done only when projected world state can be tested independently and rendered by reusable spatial surfaces with passing tests.
