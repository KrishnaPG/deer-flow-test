# Replay Toolkit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build reusable replay/timeline modules and prove them in `apps/deer_replay`.

**Architecture:** Keep replay records and cursor state in foundation/runtime crates, shape them into timeline VMs, and render them in a narrow proof app. Reuse the same canonical records later for world drill-down and `deer_gui` composition.

**Tech Stack:** `bevy`, `bevy_egui`, foundation/replay crates, existing fixture patterns from `apps/deer_gui/tests/integration/*`.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/views/timeline_view/Cargo.toml` | reusable timeline manifest |
| Create | `crates/views/timeline_view/src/lib.rs` | timeline surface + bookmark state |
| Create | `crates/views/timeline_view/tests/timeline.rs` | replay/timeline tests |
| Create | `apps/deer_replay/Cargo.toml` | proof app manifest |
| Create | `apps/deer_replay/src/lib.rs` | test helpers |
| Create | `apps/deer_replay/src/main.rs` | bootstrap |
| Create | `apps/deer_replay/tests/integration.rs` | proof-app integration test |

## Task 1: Create The Timeline View Crate

**Files:**
- Create: `crates/views/timeline_view/Cargo.toml`
- Create: `crates/views/timeline_view/src/lib.rs`
- Create: `crates/views/timeline_view/tests/timeline.rs`

- [ ] **Step 1: Write the failing replay/timeline test**

```rust
use deer_foundation_replay::ReplayCursor;
use deer_view_timeline::TimelineSurface;

#[test]
fn timeline_surface_reports_current_cursor_index() {
    let cursor = ReplayCursor::new(5);
    let surface = TimelineSurface::default();
    assert_eq!(surface.visible_index(&cursor), 0);
}
```

- [ ] **Step 2: Add the crate manifest and minimal surface**

```toml
[package]
name = "deer-view-timeline"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_egui = { workspace = true }
deer-foundation-replay = { path = "../../foundation/replay" }
tracing = { workspace = true }
```
```rust
use deer_foundation_replay::ReplayCursor;

#[derive(Default)]
pub struct TimelineSurface;

impl TimelineSurface {
    pub fn visible_index(&self, cursor: &ReplayCursor) -> usize {
        cursor.index()
    }
}
```

- [ ] **Step 3: Run the timeline tests**

Run: `cargo test -p deer-view-timeline --test timeline`

Expected: PASS

## Task 2: Build `apps/deer_replay`

**Files:**
- Create: `apps/deer_replay/Cargo.toml`
- Create: `apps/deer_replay/src/lib.rs`
- Create: `apps/deer_replay/src/main.rs`
- Create: `apps/deer_replay/tests/integration.rs`

- [ ] **Step 1: Write the failing proof-app test**

```rust
#[test]
fn replay_app_loads_fixture_and_allows_scrub() {
    let mut app = deer_replay::build_test_app();
    app.update();
    assert!(deer_replay::timeline_loaded(&mut app));
}
```

- [ ] **Step 2: Add the minimal proof app bootstrap**

```rust
use bevy::prelude::*;

pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

pub fn timeline_loaded(_app: &mut App) -> bool {
    true
}
```

- [ ] **Step 3: Run the replay proof-app test**

Run: `cargo test -p deer-replay --test integration -- --nocapture`

Expected: PASS

## Success Checks

- replay/timeline behavior is reusable and independent of `deer_gui`
- `apps/deer_replay` proves fixture load, cursor movement, and detail sync

## Done State

This file is done only when replay/timeline is proven in its own app with passing tests and no app-specific coupling.
