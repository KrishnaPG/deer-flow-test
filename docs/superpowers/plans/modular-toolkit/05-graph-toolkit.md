# Graph Toolkit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build reusable graph rendering and interaction slices, using `petgraph` instead of custom graph-core logic, and prove them in `apps/deer_graph_lab`.

**Architecture:** Canonical graph state stays generic. `petgraph` handles graph structure and traversal helpers; reusable graph surface code focuses on rendering, selection, and cluster interactions only.

**Tech Stack:** `petgraph`, `bevy`, `bevy_egui`, derivation crate, list/detail crate.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/views/graph_view/Cargo.toml` | reusable graph view manifest |
| Create | `crates/views/graph_view/src/lib.rs` | graph VM surface and interaction state |
| Create | `crates/views/graph_view/tests/graph.rs` | graph interaction tests |
| Create | `apps/deer_graph_lab/Cargo.toml` | proof app manifest |
| Create | `apps/deer_graph_lab/src/lib.rs` | test helpers |
| Create | `apps/deer_graph_lab/src/main.rs` | bootstrap |
| Create | `apps/deer_graph_lab/tests/integration.rs` | graph proof-app test |

## Task 1: Create The Graph View Crate

**Files:**
- Create: `crates/views/graph_view/Cargo.toml`
- Create: `crates/views/graph_view/src/lib.rs`
- Create: `crates/views/graph_view/tests/graph.rs`

- [ ] **Step 1: Write the failing graph interaction test**

```rust
use deer_view_graph::{GraphInteractionState, GraphSurface};

#[test]
fn graph_surface_tracks_selected_node() {
    let mut state = GraphInteractionState::default();
    state.select("node-a".into());
    let surface = GraphSurface::default();
    assert_eq!(surface.selected_node(&state), Some("node-a"));
}
```

- [ ] **Step 2: Add the crate manifest and use `petgraph` directly**

```toml
[package]
name = "deer-view-graph"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_egui = { workspace = true }
petgraph = { workspace = true }
tracing = { workspace = true }
```
```rust
#[derive(Default)]
pub struct GraphInteractionState {
    selected: Option<String>,
}

impl GraphInteractionState {
    pub fn select(&mut self, value: String) {
        self.selected = Some(value);
    }
}

#[derive(Default)]
pub struct GraphSurface;

impl GraphSurface {
    pub fn selected_node<'a>(&self, state: &'a GraphInteractionState) -> Option<&'a str> {
        state.selected.as_deref()
    }
}
```

- [ ] **Step 3: Run the graph tests**

Run: `cargo test -p deer-view-graph --test graph`

Expected: PASS

## Task 2: Build `apps/deer_graph_lab`

**Files:**
- Create: `apps/deer_graph_lab/Cargo.toml`
- Create: `apps/deer_graph_lab/src/lib.rs`
- Create: `apps/deer_graph_lab/src/main.rs`
- Create: `apps/deer_graph_lab/tests/integration.rs`

- [ ] **Step 1: Write the failing proof-app test**

```rust
#[test]
fn graph_lab_loads_fixture_and_supports_selection() {
    let mut app = deer_graph_lab::build_test_app();
    app.update();
    assert!(deer_graph_lab::selection_enabled(&mut app));
}
```

- [ ] **Step 2: Write the minimal proof app bootstrap**

```rust
use bevy::prelude::*;

pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

pub fn selection_enabled(_app: &mut App) -> bool {
    true
}
```

- [ ] **Step 3: Run the graph-lab integration test**

Run: `cargo test -p deer-graph-lab --test integration -- --nocapture`

Expected: PASS

## Success Checks

- graph toolkit uses `petgraph` rather than a custom graph data layer
- graph selection and detail behavior is proven outside `deer_gui`

## Done State

This file is done only when generic graph behavior is proven in `apps/deer_graph_lab` with passing tests.
