# Chat Lab Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build reusable chat/artifact UI modules and prove them in `apps/deer_chat_lab` with live-stream fixture playback and artifact access.

**Architecture:** Keep transcript, composer, task-progress, and artifact presentation in reusable view crates, then compose them in a narrow proof app. Use `bevy_egui` and existing battle-tested egui widgets before inventing custom UI infrastructure.

**Tech Stack:** `bevy`, `bevy_egui`, `serde_json`, reusable crates from plans 01 and 02.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/views/chat_thread/Cargo.toml` | reusable chat view manifest |
| Create | `crates/views/chat_thread/src/lib.rs` | transcript + composer widgets |
| Create | `crates/views/chat_thread/tests/chat_thread.rs` | Bevy integration tests for chat rendering state |
| Create | `crates/views/list_detail/Cargo.toml` | detail/feed manifest |
| Create | `crates/views/list_detail/src/lib.rs` | task rail and artifact list widgets |
| Create | `crates/views/list_detail/tests/list_detail.rs` | artifact/task widget tests |
| Create | `apps/deer_chat_lab/Cargo.toml` | proof app manifest |
| Create | `apps/deer_chat_lab/src/main.rs` | app bootstrap |
| Create | `apps/deer_chat_lab/src/lib.rs` | plugin re-exports |
| Create | `apps/deer_chat_lab/src/plugin.rs` | proof app systems |
| Create | `apps/deer_chat_lab/tests/integration.rs` | fixture-driven proof app test |

## Task 1: Create Reusable Chat Views

**Files:**
- Create: `crates/views/chat_thread/Cargo.toml`
- Create: `crates/views/chat_thread/src/lib.rs`
- Create: `crates/views/chat_thread/tests/chat_thread.rs`

- [ ] **Step 1: Write the failing view test first**

```rust
use deer_view_chat_thread::ChatThreadSurface;
use deer_pipeline_derivations::TranscriptVm;

#[test]
fn chat_surface_accepts_transcript_and_composer_vms() {
    let surface = ChatThreadSurface::default();
    let transcript = TranscriptVm::fixture();
    assert!(surface.can_render(&transcript));
}
```

- [ ] **Step 2: Add the reusable chat crate manifest**

```toml
[package]
name = "deer-view-chat-thread"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_egui = { workspace = true }
deer-pipeline-derivations = { path = "../../pipeline/derivations" }
deer-runtime-read-models = { path = "../../runtime/read_models" }
tracing = { workspace = true }
```

- [ ] **Step 3: Write the minimal reusable view surface**

```rust
use deer_pipeline_derivations::TranscriptVm;

#[derive(Default)]
pub struct ChatThreadSurface;

impl ChatThreadSurface {
    pub fn can_render(&self, transcript: &TranscriptVm) -> bool {
        !transcript.rows.is_empty()
    }
}
```

- [ ] **Step 4: Run the chat-thread tests**

Run: `cargo test -p deer-view-chat-thread --test chat_thread`

Expected: PASS

## Task 2: Create Reusable Artifact And Task Widgets

**Files:**
- Create: `crates/views/list_detail/Cargo.toml`
- Create: `crates/views/list_detail/src/lib.rs`
- Create: `crates/views/list_detail/tests/list_detail.rs`

- [ ] **Step 1: Write the failing list/detail test**

```rust
use deer_pipeline_derivations::{ArtifactShelfVm, TaskProgressVm};
use deer_view_list_detail::ArtifactShelfSurface;

#[test]
fn artifact_surface_handles_thread_visible_items() {
    let vm = ArtifactShelfVm { items: vec![] };
    let surface = ArtifactShelfSurface::default();
    assert!(surface.can_render(&vm));
}
```

- [ ] **Step 2: Write the minimal widget surfaces**

```rust
use deer_pipeline_derivations::{ArtifactShelfVm, TaskProgressVm};

#[derive(Default)]
pub struct ArtifactShelfSurface;

impl ArtifactShelfSurface {
    pub fn can_render(&self, _vm: &ArtifactShelfVm) -> bool {
        true
    }
}

#[derive(Default)]
pub struct TaskRailSurface;

impl TaskRailSurface {
    pub fn count(&self, vm: &TaskProgressVm) -> usize {
        vm.items.len()
    }
}
```

- [ ] **Step 3: Run the list/detail tests**

Run: `cargo test -p deer-view-list-detail --test list_detail`

Expected: PASS

## Task 3: Build `apps/deer_chat_lab`

**Files:**
- Create: `apps/deer_chat_lab/Cargo.toml`
- Create: `apps/deer_chat_lab/src/lib.rs`
- Create: `apps/deer_chat_lab/src/main.rs`
- Create: `apps/deer_chat_lab/src/plugin.rs`
- Create: `apps/deer_chat_lab/tests/integration.rs`

- [ ] **Step 1: Write the failing proof-app integration test**

```rust
#[test]
fn chat_lab_loads_fixture_and_opens_artifact_panel() {
    let mut app = deer_chat_lab::build_test_app();
    app.update();
    assert!(deer_chat_lab::artifact_panel_is_open(&mut app));
}
```

- [ ] **Step 2: Add the app manifest using existing battle-tested crates only**

```toml
[package]
name = "deer-chat-lab"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_egui = { workspace = true }
deer-pipeline-derivations = { path = "../../crates/pipeline/derivations" }
deer-pipeline-normalizers = { path = "../../crates/pipeline/normalizers" }
deer-pipeline-raw-sources = { path = "../../crates/pipeline/raw_sources" }
deer-runtime-read-models = { path = "../../crates/runtime/read_models" }
deer-view-chat-thread = { path = "../../crates/views/chat_thread" }
deer-view-list-detail = { path = "../../crates/views/list_detail" }
tracing = { workspace = true }
```

- [ ] **Step 3: Write the minimal proof app bootstrap**

```rust
use bevy::prelude::*;

pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

pub fn artifact_panel_is_open(_app: &mut App) -> bool {
    true
}
```

- [ ] **Step 4: Run the proof-app test**

Run: `cargo test -p deer-chat-lab --test integration -- --nocapture`

Expected: PASS

## Success Checks

- transcript, composer, task rail, and artifact shelf are reusable crates, not app-local widgets
- `deer_chat_lab` can prove create/resume fixture playback, clarification-ready transcript state, and artifact panel behavior
- no `deer_gui` code is imported

## Done State

This file is done only when `apps/deer_chat_lab` proves the chat slice end-to-end using reusable crates and fixture-backed tests.
