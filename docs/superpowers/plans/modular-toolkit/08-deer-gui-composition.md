# Deer GUI Composition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compose the proven toolkit slices into the first playable `deer_gui` loop without reintroducing monolith-first architecture.

**Architecture:** Keep `apps/deer_gui` thin. Reuse existing scene/theme/HUD assets, add panel/layout composition, bind chat/replay/graph/spatial slices through stable IDs, and verify the playable loop using fixtures before any live-mode polish work.

**Tech Stack:** existing `apps/deer_gui/src/*`, Bevy 0.18.1, `bevy_egui`, reusable crates from plans 01-07.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `apps/deer_gui/Cargo.toml` | depend on reusable toolkit crates |
| Modify | `apps/deer_gui/src/main.rs` | add proof-app-derived plugins in composition order |
| Modify | `apps/deer_gui/src/lib.rs` | re-export thin composition modules only |
| Create | `apps/deer_gui/src/composition/mod.rs` | composition-only module root |
| Create | `apps/deer_gui/src/composition/plugin.rs` | compose panels, views, and projection |
| Create | `apps/deer_gui/src/composition/state.rs` | stable IDs and app wiring state |
| Create | `apps/deer_gui/tests/integration/playable_loop.rs` | first playable loop test |
| Modify | `apps/deer_gui/tests/integration.rs` | register new integration module |

## Task 1: Add Toolkit Dependencies To `deer_gui`

**Files:**
- Modify: `apps/deer_gui/Cargo.toml`

- [ ] **Step 1: Write the dependency block using local reusable crates**

Add under `[dependencies]`:

```toml
deer-foundation-domain = { path = "../../crates/foundation/domain" }
deer-pipeline-derivations = { path = "../../crates/pipeline/derivations" }
deer-runtime-read-models = { path = "../../crates/runtime/read_models" }
deer-runtime-world-projection = { path = "../../crates/runtime/world_projection" }
deer-ui-layout-runtime = { path = "../../crates/ui/layout_runtime" }
deer-ui-panel-shells = { path = "../../crates/ui/panel_shells" }
deer-view-chat-thread = { path = "../../crates/views/chat_thread" }
deer-view-graph = { path = "../../crates/views/graph_view" }
deer-view-list-detail = { path = "../../crates/views/list_detail" }
deer-view-telemetry = { path = "../../crates/views/telemetry_view" }
deer-view-timeline = { path = "../../crates/views/timeline_view" }
tracing = { workspace = true }
```

- [ ] **Step 2: Run a compile check and verify missing composition modules fail next**

Run: `cargo check -p deer-gui`

Expected: FAIL only for missing composition modules or unused dependency wiring not yet added.

## Task 2: Add A Thin Composition Module

**Files:**
- Create: `apps/deer_gui/src/composition/mod.rs`
- Create: `apps/deer_gui/src/composition/plugin.rs`
- Create: `apps/deer_gui/src/composition/state.rs`
- Modify: `apps/deer_gui/src/lib.rs`
- Modify: `apps/deer_gui/src/main.rs`

- [ ] **Step 1: Write the failing integration test first**

```rust
#[test]
fn playable_loop_loads_composed_panels_and_world_projection() {
    let mut app = deer_gui::tests::build_app();
    app.update();
    assert!(deer_gui::tests::playable_loop_ready(&mut app));
}
```

- [ ] **Step 2: Add the composition module root**

```rust
pub mod plugin;
pub mod state;

pub use plugin::CompositionPlugin;
```

- [ ] **Step 3: Add a minimal composition plugin and wire it into `main.rs`**

```rust
use bevy::prelude::*;

pub struct CompositionPlugin;

impl Plugin for CompositionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::composition::state::CompositionState>();
    }
}
```
```rust
#[derive(Default, Resource)]
pub struct CompositionState {
    pub chat_ready: bool,
    pub world_ready: bool,
}
```

- [ ] **Step 4: Run the targeted integration test**

Run: `cargo test -p deer-gui --test integration playable_loop -- --nocapture`

Expected: PASS

## Task 3: Prove The First Playable Loop

**Files:**
- Create: `apps/deer_gui/tests/integration/playable_loop.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Write the closed-loop fixture test**

```rust
#[test]
fn first_playable_loop_covers_request_to_intervention() {
    let mut app = deer_gui::tests::build_app();
    app.update();

    assert!(deer_gui::tests::chat_panel_visible(&mut app));
    assert!(deer_gui::tests::artifact_panel_visible(&mut app));
    assert!(deer_gui::tests::world_projection_visible(&mut app));
}
```

- [ ] **Step 2: Add the test registration in `tests/integration.rs`**

```rust
pub mod playable_loop;
```

- [ ] **Step 3: Run full `deer_gui` integration verification**

Run: `cargo test -p deer-gui --test integration -- --nocapture`

Expected: PASS, including all existing tests and the new playable-loop test.

## Task 4: Run Final Quality Gates

**Files:**
- Modify only if a warning or file-size issue appears during verification

- [ ] **Step 1: Run full workspace tests**

Run: `cargo test --workspace --all-targets`

Expected: PASS

- [ ] **Step 2: Run workspace lint gates**

Run: `cargo clippy --workspace --all-targets -- -D warnings`

Expected: PASS

- [ ] **Step 3: Verify drift guards manually before merge**

Check:

```text
- deer_gui did not become the first implementation of chat, replay, graph, or layout runtime
- no raw DeerFlow/LangGraph types leaked above raw_sources/normalizers
- world_projection contains projection logic only
- new files stay under 400 LOC and new functions stay under 50 LOC
```

## Success Checks

- `deer_gui` composes already-proven slices instead of inventing them first
- one closed loop exists: request -> live orchestration -> artifacts/results -> world reaction -> intervention -> return to macro state
- all old and new integration tests pass
- composition code remains thin and reusable crates remain independent

## Done State

This file is done only when the first playable loop is proven by passing tests and `deer_gui` still reads as composition, not as a new monolith.
