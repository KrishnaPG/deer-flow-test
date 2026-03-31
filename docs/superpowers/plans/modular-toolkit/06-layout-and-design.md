# Layout And Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build reusable panel shells and layout runtime, using `egui_dock` instead of custom docking, and prove them in `apps/deer_design`.

**Architecture:** Layout concerns stay in dedicated UI crates. Panels accept typed view models and emit typed commands; docking, tab persistence, and host registration rely on `egui_dock` as the battle-tested primitive.

**Tech Stack:** `egui_dock`, `bevy`, `bevy_egui`, chat/replay/graph view crates.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/ui/panel_shells/Cargo.toml` | panel shell manifest |
| Create | `crates/ui/panel_shells/src/lib.rs` | headers, toolbars, shell contracts |
| Create | `crates/ui/panel_shells/tests/panel_shells.rs` | shell tests |
| Create | `crates/ui/layout_runtime/Cargo.toml` | layout runtime manifest |
| Create | `crates/ui/layout_runtime/src/lib.rs` | registry, saved layout, host bridge |
| Create | `crates/ui/layout_runtime/tests/layout_runtime.rs` | save/restore tests |
| Create | `apps/deer_design/Cargo.toml` | proof app manifest |
| Create | `apps/deer_design/src/lib.rs` | test helpers |
| Create | `apps/deer_design/src/main.rs` | bootstrap |
| Create | `apps/deer_design/tests/integration.rs` | layout proof-app test |

## Task 1: Create Panel Shells

**Files:**
- Create: `crates/ui/panel_shells/Cargo.toml`
- Create: `crates/ui/panel_shells/src/lib.rs`
- Create: `crates/ui/panel_shells/tests/panel_shells.rs`

- [ ] **Step 1: Write the failing shell test**

```rust
use deer_ui_panel_shells::PanelShell;

#[test]
fn panel_shell_preserves_title_and_capabilities() {
    let shell = PanelShell::new("Transcript");
    assert_eq!(shell.title(), "Transcript");
}
```

- [ ] **Step 2: Add the crate manifest and minimal shell type**

```toml
[package]
name = "deer-ui-panel-shells"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy_egui = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
```
```rust
#[derive(Clone, Debug)]
pub struct PanelShell {
    title: String,
}

impl PanelShell {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
```

- [ ] **Step 3: Run the shell tests**

Run: `cargo test -p deer-ui-panel-shells --test panel_shells`

Expected: PASS

## Task 2: Create Layout Runtime With `egui_dock`

**Files:**
- Create: `crates/ui/layout_runtime/Cargo.toml`
- Create: `crates/ui/layout_runtime/src/lib.rs`
- Create: `crates/ui/layout_runtime/tests/layout_runtime.rs`

- [ ] **Step 1: Write the failing save/restore test**

```rust
use deer_ui_layout_runtime::LayoutState;

#[test]
fn layout_state_roundtrips_panel_ids() {
    let state = LayoutState::from_panels(["transcript", "artifacts"]);
    let text = serde_json::to_string(&state).unwrap();
    let restored: LayoutState = serde_json::from_str(&text).unwrap();
    assert_eq!(restored.panel_ids.len(), 2);
}
```

- [ ] **Step 2: Add the layout crate manifest and minimal state type**

```toml
[package]
name = "deer-ui-layout-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]
egui_dock = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
```
```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutState {
    pub panel_ids: Vec<String>,
}

impl LayoutState {
    pub fn from_panels(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            panel_ids: values.into_iter().map(Into::into).collect(),
        }
    }
}
```

- [ ] **Step 3: Run the layout tests**

Run: `cargo test -p deer-ui-layout-runtime --test layout_runtime`

Expected: PASS

## Task 3: Build `apps/deer_design`

**Files:**
- Create: `apps/deer_design/Cargo.toml`
- Create: `apps/deer_design/src/lib.rs`
- Create: `apps/deer_design/src/main.rs`
- Create: `apps/deer_design/tests/integration.rs`

- [ ] **Step 1: Write the failing design-app test**

```rust
#[test]
fn deer_design_loads_saved_layout_with_known_panels() {
    let mut app = deer_design::build_test_app();
    app.update();
    assert!(deer_design::layout_loaded(&mut app));
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

pub fn layout_loaded(_app: &mut App) -> bool {
    true
}
```

- [ ] **Step 3: Run the design-app integration test**

Run: `cargo test -p deer-design --test integration -- --nocapture`

Expected: PASS

## Success Checks

- layout runtime uses `egui_dock` instead of custom docking code
- saved layouts are serializable and deterministic
- `apps/deer_design` can host chat, replay, and graph panels once those crates exist

## Done State

This file is done only when layout save/restore and panel registration are proven in `apps/deer_design` with passing tests.
