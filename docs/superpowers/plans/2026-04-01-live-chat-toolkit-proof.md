# Live Chat Toolkit Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the first reusable live-chat slice in `apps/deer_chat_lab` so a thread can be created or resumed, uploads can be staged before submit, live message/tool/task updates can stream through reusable modules, clarification can pause and resume cleanly, and thread-visible artifacts can be previewed or opened only through mediated access.

**Architecture:** Extend the planned M0 crates with one raw-source crate for thread, upload, run-stream, and artifact gateways; two reusable view crates for transcript/composer and artifact/detail presentation; and a narrow proof app that wires those reusable layers together without introducing layout-runtime or world-projection behavior. The proof app remains a thin integration shell: canonical types stay in foundation crates, normalization remains in pipeline crates, transient interaction state remains in runtime read models, and all UI bricks consume typed VMs rather than raw backend payloads.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `thiserror`, `insta`, `similar-asserts`, existing Bevy stack only at reusable-view and proof-app boundaries

---

## Scope Boundary

- This plan covers only the M1 live chat proof slice: raw source ingress for DeerFlow-style thread/run/artifact transport, reusable chat and artifact presentation views, and the narrow proving app `apps/deer_chat_lab`.
- It assumes the contracts described in `docs/superpowers/plans/2026-04-01-foundation-spine-contracts-domain-replay.md` and `docs/superpowers/plans/2026-04-01-normalization-derivation-read-model-spine.md` exist conceptually and builds on those planned crates rather than redefining them.
- It does **not** create `crates/ui/panel_shells`, `crates/ui/layout_runtime`, `crates/runtime/world_projection`, `crates/views/scene3d`, `crates/views/telemetry_view`, or `apps/deer_design`.
- It does **not** modify `apps/deer_gui`.
- It does **not** allow raw backend types above `crates/pipeline/raw_sources` and `crates/pipeline/normalizers`.
- It does **not** allow direct file-path, DB, vector-index, S3, or thread-storage bypasses for artifact access; preview and open behavior must remain mediated by contract.

## File Structure

### Workspace

- Modify: `Cargo.toml` - add each new crate or app member incrementally as its task begins.

### Raw sources crate

- Create: `crates/pipeline/raw_sources/Cargo.toml` - package manifest for transport and fixture gateways.
- Create: `crates/pipeline/raw_sources/src/lib.rs` - public exports.
- Create: `crates/pipeline/raw_sources/src/error.rs` - gateway and stream error types.
- Create: `crates/pipeline/raw_sources/src/thread_gateway.rs` - create/resume thread gateway and snapshot DTOs.
- Create: `crates/pipeline/raw_sources/src/upload_gateway.rs` - pre-submit upload gateway and upload receipt DTOs.
- Create: `crates/pipeline/raw_sources/src/run_stream.rs` - live stream fixture player and event iterator.
- Create: `crates/pipeline/raw_sources/src/artifact_gateway.rs` - mediated preview and pointer retrieval gateway.
- Create: `crates/pipeline/raw_sources/tests/thread_and_upload.rs` - fixture-backed tests for thread create/resume and uploads.
- Create: `crates/pipeline/raw_sources/tests/run_stream_and_artifacts.rs` - fixture-backed tests for live events, clarification, and mediated artifact access.
- Create: `crates/pipeline/raw_sources/tests/fixtures/thread_resume.json` - existing thread snapshot fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/live_run_stream.json` - streaming message/tool/task/clarification/artifact fixture.

### Derivation and read-model extensions

- Modify: `crates/pipeline/derivations/src/lib.rs` - export the additional M1 VMs.
- Create: `crates/pipeline/derivations/src/composer.rs` - `ComposerVm` derivation.
- Create: `crates/pipeline/derivations/src/thread_header.rs` - `ThreadHeaderVm` derivation.
- Modify: `crates/pipeline/derivations/src/orchestration.rs` - include live stream/tool/clarification rows in `TranscriptVm` and run/task status shaping.
- Modify: `crates/pipeline/derivations/src/artifacts.rs` - add preview, retrieval, provenance, and presentation-state shaping.
- Create: `crates/pipeline/derivations/tests/composer_vm.rs` - deterministic composer-state snapshot tests.
- Create: `crates/pipeline/derivations/tests/thread_header_vm.rs` - deterministic thread-header snapshot tests.
- Modify: `crates/pipeline/derivations/tests/orchestration_vm.rs` - transcript/tool/clarification snapshot tests.
- Modify: `crates/pipeline/derivations/tests/artifact_vm.rs` - artifact shelf/presenter snapshot tests.
- Modify: `crates/runtime/read_models/src/chat.rs` - add staged attachments, optimistic send, clarification pending, and stream lifecycle state.
- Modify: `crates/runtime/read_models/src/artifacts.rs` - add artifact open and preview request state.
- Modify: `crates/runtime/read_models/src/temporal.rs` - add live stream freshness and degradation markers.
- Create: `crates/runtime/read_models/tests/chat_flow_reducer.rs` - chat draft, clarification, and stream lifecycle reducers.
- Modify: `crates/runtime/read_models/tests/temporal_and_policy.rs` - degradation and artifact invalidation tests.

### Reusable view crates

- Create: `crates/views/chat_thread/Cargo.toml` - package manifest for transcript and composer views.
- Create: `crates/views/chat_thread/src/lib.rs` - public exports.
- Create: `crates/views/chat_thread/src/thread_header_view.rs` - thread header rendering and commands.
- Create: `crates/views/chat_thread/src/transcript_view.rs` - transcript rendering for human, assistant, tool, and clarification rows.
- Create: `crates/views/chat_thread/src/composer_view.rs` - prompt composer and attachment controls.
- Create: `crates/views/chat_thread/src/progress_rail_view.rs` - run-status and task-progress rail.
- Create: `crates/views/chat_thread/tests/transcript_view.rs` - behavior/snapshot tests for transcript rendering.
- Create: `crates/views/chat_thread/tests/composer_view.rs` - behavior/snapshot tests for composer commands.
- Create: `crates/views/chat_thread/tests/progress_rail_view.rs` - behavior/snapshot tests for live progress and degradation banners.
- Create: `crates/views/list_detail/Cargo.toml` - package manifest for artifact shelf and file presenter.
- Create: `crates/views/list_detail/src/lib.rs` - public exports.
- Create: `crates/views/list_detail/src/artifact_shelf_view.rs` - shelf rendering and open/download/install commands.
- Create: `crates/views/list_detail/src/file_presenter_view.rs` - preview/pointer handoff rendering.
- Create: `crates/views/list_detail/tests/artifact_shelf_view.rs` - behavior/snapshot tests for artifact lifecycle states.
- Create: `crates/views/list_detail/tests/file_presenter_view.rs` - behavior/snapshot tests for preview vs pointer access.

### Proof app

- Create: `apps/deer_chat_lab/Cargo.toml` - package manifest for the narrow proof app.
- Create: `apps/deer_chat_lab/src/lib.rs` - proof-app library exports for scenario tests.
- Create: `apps/deer_chat_lab/src/main.rs` - app bootstrap.
- Create: `apps/deer_chat_lab/src/app.rs` - proof composition of raw sources, normalizers, derivations, read models, and reusable views.
- Create: `apps/deer_chat_lab/src/scenarios.rs` - scripted proof scenarios and fixture selection.
- Create: `apps/deer_chat_lab/tests/live_chat_lab.rs` - repeatable proof-app scenario tests.

## Task 1: Add Thread, Upload, Run-Stream, And Artifact Raw Sources

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/pipeline/raw_sources/Cargo.toml`
- Create: `crates/pipeline/raw_sources/src/lib.rs`
- Create: `crates/pipeline/raw_sources/src/error.rs`
- Create: `crates/pipeline/raw_sources/src/thread_gateway.rs`
- Create: `crates/pipeline/raw_sources/src/upload_gateway.rs`
- Create: `crates/pipeline/raw_sources/src/run_stream.rs`
- Create: `crates/pipeline/raw_sources/src/artifact_gateway.rs`
- Test: `crates/pipeline/raw_sources/tests/thread_and_upload.rs`
- Test: `crates/pipeline/raw_sources/tests/run_stream_and_artifacts.rs`
- Test fixtures: `crates/pipeline/raw_sources/tests/fixtures/thread_resume.json`, `crates/pipeline/raw_sources/tests/fixtures/live_run_stream.json`

**Milestone unlock:** M1 raw-source ingress for live chat proof

**Proof path later:** consumed by `apps/deer_chat_lab`

**Forbidden dependencies:** `apps/deer_gui`, app-local structs, layout runtime, direct artifact file-path opening above the gateway boundary

- [ ] **Step 1: Write the failing raw-source tests, fixtures, and workspace stubs**

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
]
resolver = "2"
```

```toml
# crates/pipeline/raw_sources/Cargo.toml
[package]
name = "deer-pipeline-raw-sources"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2.0.17"

[dev-dependencies]
similar-asserts = "1.7.0"
```

```rust
// crates/pipeline/raw_sources/src/lib.rs
pub mod artifact_gateway;
pub mod error;
pub mod run_stream;
pub mod thread_gateway;
pub mod upload_gateway;

pub use artifact_gateway::{preview_artifact, ArtifactAccess};
pub use run_stream::{load_stream_fixture, RawStreamEvent};
pub use thread_gateway::{create_thread, resume_thread, ThreadSnapshot};
pub use upload_gateway::{stage_upload, UploadReceipt};
```

```json
// crates/pipeline/raw_sources/tests/fixtures/thread_resume.json
{
  "thread_id": "thread_1",
  "title": "Survey the ridge",
  "messages": [
    {
      "message_id": "msg_1",
      "role": "assistant",
      "text": "Previous terrain notes are attached."
    }
  ],
  "artifacts": [
    {
      "artifact_id": "artifact_1",
      "name": "terrain.md",
      "presented": true
    }
  ]
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/live_run_stream.json
[
  { "kind": "message_delta", "message_id": "msg_2", "text": "Scanning sector" },
  { "kind": "tool_call", "tool_call_id": "tool_1", "tool_name": "map_scan" },
  { "kind": "task_progress", "task_id": "task_1", "state": "running", "label": "Gather terrain notes" },
  { "kind": "clarification", "clarification_id": "clar_1", "prompt": "Confirm the survey radius." },
  { "kind": "artifact_presented", "artifact_id": "artifact_2", "name": "scan.png" },
  { "kind": "runtime_status", "state": "live" }
]
```

```rust
// crates/pipeline/raw_sources/tests/thread_and_upload.rs
use deer_pipeline_raw_sources::{create_thread, resume_thread, stage_upload};

#[test]
fn creates_or_resumes_threads_and_stages_uploads_before_submit() {
    let created = create_thread("Survey the ridge").unwrap();
    let resumed = resume_thread(include_str!("fixtures/thread_resume.json")).unwrap();
    let upload = stage_upload(created.thread_id.as_str(), "briefing.pdf").unwrap();

    assert_eq!(created.title, "Survey the ridge");
    assert_eq!(resumed.thread_id, "thread_1");
    assert_eq!(upload.file_name, "briefing.pdf");
}
```

```rust
// crates/pipeline/raw_sources/tests/run_stream_and_artifacts.rs
use deer_pipeline_raw_sources::{load_stream_fixture, preview_artifact, ArtifactAccess, AdapterEvent, RawStreamEvent};

#[test]
fn loads_live_stream_events_and_keeps_artifact_access_mediated() {
    let events = load_stream_fixture(include_str!("fixtures/live_run_stream.json")).unwrap();
    let preview = preview_artifact("thread_1", "artifact_2").unwrap();

    assert!(events.iter().any(|event| matches!(event, AdapterEvent::Deerflow(RawStreamEvent::Clarification { .. }))));
    assert!(events.iter().any(|event| matches!(event, AdapterEvent::Deerflow(RawStreamEvent::TaskProgress { .. }))));
    assert_eq!(preview, ArtifactAccess::PreviewPayload { mime: "image/png".into() });
}
```

- [ ] **Step 2: Run the raw-source tests to verify they fail**

Run: `cargo test -p deer-pipeline-raw-sources --test thread_and_upload --test run_stream_and_artifacts -v`

Expected: FAIL with missing gateway modules, missing fixture loaders, and unresolved raw-source DTOs.

- [ ] **Step 3: Implement the minimal thread, upload, stream, and artifact gateways**

```rust
// crates/pipeline/raw_sources/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RawSourceError {
    #[error("invalid fixture payload")]
    InvalidFixture,
    #[error("artifact access must remain mediated")]
    UnmediatedArtifactAccess,
}
```

```rust
// crates/pipeline/raw_sources/src/thread_gateway.rs
use serde::Deserialize;

use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThreadSnapshot {
    pub thread_id: String,
    pub title: String,
}

pub fn create_thread(title: &str) -> Result<ThreadSnapshot, RawSourceError> {
    Ok(ThreadSnapshot {
        thread_id: "thread_created_1".into(),
        title: title.into(),
    })
}

pub fn resume_thread(fixture_json: &str) -> Result<ThreadSnapshot, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}
```

```rust
// crates/pipeline/raw_sources/src/upload_gateway.rs
use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceipt {
    pub thread_id: String,
    pub file_name: String,
}

pub fn stage_upload(thread_id: &str, file_name: &str) -> Result<UploadReceipt, RawSourceError> {
    Ok(UploadReceipt {
        thread_id: thread_id.into(),
        file_name: file_name.into(),
    })
}
```

```rust
// crates/pipeline/raw_sources/src/run_stream.rs
use serde::Deserialize;

use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RawStreamEvent {
    MessageDelta { message_id: String, text: String },
    ToolCall { tool_call_id: String, tool_name: String },
    TaskProgress { task_id: String, state: String, label: String },
    Clarification { clarification_id: String, prompt: String },
    ArtifactPresented { artifact_id: String, name: String },
    RuntimeStatus { state: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterEvent {
    Deerflow(RawStreamEvent),
    Hermes(RawStreamEvent),
}

pub fn load_stream_fixture(fixture_json: &str) -> Result<Vec<AdapterEvent>, RawSourceError> {
    let events: Vec<RawStreamEvent> = serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;
    Ok(events.into_iter().map(AdapterEvent::Deerflow).collect())
}
```

```rust
// crates/pipeline/raw_sources/src/artifact_gateway.rs
use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactAccess {
    PreviewPayload { mime: String },
    AuthorizedPointer { href: String },
}

pub fn preview_artifact(thread_id: &str, artifact_id: &str) -> Result<ArtifactAccess, RawSourceError> {
    if thread_id.is_empty() || artifact_id.is_empty() {
        return Err(RawSourceError::UnmediatedArtifactAccess);
    }

    Ok(ArtifactAccess::PreviewPayload { mime: "image/png".into() })
}
```

- [ ] **Step 4: Run the raw-source tests and then the package sweep**

Run: `cargo test -p deer-pipeline-raw-sources --test thread_and_upload --test run_stream_and_artifacts -v && cargo test -p deer-pipeline-raw-sources -v`

Expected: PASS with fixture-backed thread create/resume, upload staging, stream event loading, and mediated artifact preview access.

- [ ] **Step 5: Commit the raw-source chat ingress**

```bash
git add Cargo.toml crates/pipeline/raw_sources
git commit -m "feat: add live chat raw sources"
```

## Task 2: Extend Derivations And Read Models For Live Chat, Clarification, And Artifact Access

**Files:**
- Modify: `crates/pipeline/derivations/src/lib.rs`
- Create: `crates/pipeline/derivations/src/composer.rs`
- Create: `crates/pipeline/derivations/src/thread_header.rs`
- Modify: `crates/pipeline/derivations/src/orchestration.rs`
- Modify: `crates/pipeline/derivations/src/artifacts.rs`
- Test: `crates/pipeline/derivations/tests/composer_vm.rs`
- Test: `crates/pipeline/derivations/tests/thread_header_vm.rs`
- Modify: `crates/pipeline/derivations/tests/orchestration_vm.rs`
- Modify: `crates/pipeline/derivations/tests/artifact_vm.rs`
- Modify: `crates/runtime/read_models/src/chat.rs`
- Modify: `crates/runtime/read_models/src/artifacts.rs`
- Modify: `crates/runtime/read_models/src/temporal.rs`
- Test: `crates/runtime/read_models/tests/chat_flow_reducer.rs`
- Modify: `crates/runtime/read_models/tests/temporal_and_policy.rs`

**Milestone unlock:** M1 typed VMs and shell-local state for transcript, composer, progress, clarification, and artifact presentation

**Proof path later:** consumed by `crates/views/chat_thread`, `crates/views/list_detail`, and `apps/deer_chat_lab`

**Forbidden shortcuts:** no artifact visibility before presentation, no direct jump from clarification prompt to resolved state, no hidden degraded stream state

- [ ] **Step 1: Write the failing derivation and reducer tests**

```rust
// crates/pipeline/derivations/tests/composer_vm.rs
use deer_pipeline_derivations::derive_composer_vm;
use deer_runtime_read_models::ChatDraftState;
use insta::assert_yaml_snapshot;

#[test]
fn derives_composer_vm_with_draft_attachments_and_clarification_mode() {
    let chat_state = ChatDraftState {
        prompt_text: "Confirm the survey radius".into(),
        attachment_ids: vec!["upload_1".into()],
        clarification_pending: true,
    };

    let vm = derive_composer_vm(&chat_state);

    assert_yaml_snapshot!(vm, @r#"
mode: clarification_response
prompt_text: Confirm the survey radius
attachments:
  - upload_1
"#);
}
```

```rust
// crates/pipeline/derivations/tests/thread_header_vm.rs
use deer_pipeline_derivations::derive_thread_header_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_thread_header_vm_with_stream_and_thread_state() {
    let vm = derive_thread_header_vm("thread_1", "Survey the ridge", "live");

    assert_yaml_snapshot!(vm, @r#"
thread_id: thread_1
title: Survey the ridge
stream_state: live
"#);
}
```

```rust
// crates/runtime/read_models/tests/chat_flow_reducer.rs
use deer_runtime_read_models::{reduce_chat_state, ChatAction, ChatDraftState};

#[test]
fn chat_reducer_tracks_uploads_clarification_and_stream_lifecycle() {
    let state = ChatDraftState::default();

    let state = reduce_chat_state(state, ChatAction::AddAttachment { attachment_id: "upload_1".into() });
    let state = reduce_chat_state(state, ChatAction::ClarificationRequested);
    let state = reduce_chat_state(state, ChatAction::StreamStateChanged { state: "degraded".into() });

    assert_eq!(state.attachment_ids, vec!["upload_1".to_string()]);
    assert!(state.clarification_pending);
    assert_eq!(state.stream_state.as_deref(), Some("degraded"));
}
```

```rust
// crates/pipeline/derivations/tests/artifact_vm.rs
use deer_foundation_domain::{AnyRecord, ArtifactRecord};
use deer_pipeline_derivations::derive_artifact_shelf_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_artifact_shelf_vm_with_presented_previewable_and_retrievable_states() {
    let records = vec![AnyRecord::Artifact(ArtifactRecord::new(
        "artifact_2".into(),
        "scan.png".into(),
        "presented".into(),
        Some("sha256:scan-png".into()),
    ))];

    let shelf = derive_artifact_shelf_vm(&records);

    assert_yaml_snapshot!(shelf, @r#"
entries:
  - artifact_id: artifact_2
    title: scan.png
    status: presented
    preview_supported: true
    retrieval_mode: mediated_pointer
"#);
}
```

- [ ] **Step 2: Run the new derivation and reducer tests to verify they fail**

Run: `cargo test -p deer-pipeline-derivations --test composer_vm --test thread_header_vm --test artifact_vm -v && cargo test -p deer-runtime-read-models --test chat_flow_reducer -v`

Expected: FAIL with missing `derive_composer_vm`, missing `derive_thread_header_vm`, and missing chat reducer behavior.

- [ ] **Step 3: Implement the minimal live-chat derivations and reducers**

```rust
// crates/pipeline/derivations/src/composer.rs
use serde::Serialize;

use deer_runtime_read_models::ChatDraftState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComposerVm {
    pub mode: &'static str,
    pub prompt_text: String,
    pub attachments: Vec<String>,
}

pub fn derive_composer_vm(state: &ChatDraftState) -> ComposerVm {
    ComposerVm {
        mode: if state.clarification_pending {
            "clarification_response"
        } else {
            "prompt"
        },
        prompt_text: state.prompt_text.clone(),
        attachments: state.attachment_ids.clone(),
    }
}
```

```rust
// crates/pipeline/derivations/src/thread_header.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreadHeaderVm {
    pub thread_id: String,
    pub title: String,
    pub stream_state: String,
}

pub fn derive_thread_header_vm(thread_id: &str, title: &str, stream_state: &str) -> ThreadHeaderVm {
    ThreadHeaderVm {
        thread_id: thread_id.into(),
        title: title.into(),
        stream_state: stream_state.into(),
    }
}
```

```rust
// crates/pipeline/derivations/src/lib.rs
pub mod artifacts;
pub mod common;
pub mod composer;
pub mod macro_state;
pub mod orchestration;
pub mod thread_header;

pub use artifacts::derive_artifact_shelf_vm;
pub use composer::derive_composer_vm;
pub use macro_state::derive_macro_state_vm;
pub use orchestration::derive_transcript_vm;
pub use thread_header::derive_thread_header_vm;
```

```rust
// crates/runtime/read_models/src/chat.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ChatDraftState {
    pub prompt_text: String,
    pub attachment_ids: Vec<String>,
    pub clarification_pending: bool,
    pub stream_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAction {
    AddAttachment { attachment_id: String },
    ClarificationRequested,
    ClarificationResolved,
    StreamStateChanged { state: String },
}

pub fn reduce_chat_state(mut state: ChatDraftState, action: ChatAction) -> ChatDraftState {
    match action {
        ChatAction::AddAttachment { attachment_id } => state.attachment_ids.push(attachment_id),
        ChatAction::ClarificationRequested => state.clarification_pending = true,
        ChatAction::ClarificationResolved => state.clarification_pending = false,
        ChatAction::StreamStateChanged { state: next } => state.stream_state = Some(next),
    }

    state
}
```

```rust
// crates/runtime/read_models/src/lib.rs
pub mod artifacts;
pub mod chat;
pub mod intent;
pub mod linked_shell;
pub mod policy;
pub mod temporal;

pub use chat::{reduce_chat_state, ChatAction, ChatDraftState};
pub use intent::{IntentAction, IntentDraftState, IntentLifecycleState, reduce_intent_state};
pub use linked_shell::{LinkedShellAction, LinkedShellState, reduce_linked_shell_state};
pub use temporal::{TemporalAction, TemporalState, reduce_temporal_state};
```

```rust
// crates/runtime/read_models/src/artifacts.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ArtifactPanelState {
    pub selected_artifact_id: Option<String>,
    pub preview_request_state: Option<String>,
    pub auto_open_after_present: bool,
}
```

- [ ] **Step 4: Run the derivation and reducer tests plus package sweeps**

Run: `cargo test -p deer-pipeline-derivations --test composer_vm --test thread_header_vm --test artifact_vm -v && cargo test -p deer-runtime-read-models --test chat_flow_reducer -v && cargo test -p deer-pipeline-derivations -v && cargo test -p deer-runtime-read-models -v`

Expected: PASS with deterministic composer/thread/artifact snapshots and reducer coverage for attachments, clarification state, and stream degradation markers.

- [ ] **Step 5: Commit the live-chat VM and reducer extensions**

```bash
git add crates/pipeline/derivations crates/runtime/read_models
git commit -m "feat: extend live chat derivations and read models"
```

## Task 3: Create Reusable Chat Thread And Artifact Presenter Views

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/views/chat_thread/Cargo.toml`
- Create: `crates/views/chat_thread/src/lib.rs`
- Create: `crates/views/chat_thread/src/thread_header_view.rs`
- Create: `crates/views/chat_thread/src/transcript_view.rs`
- Create: `crates/views/chat_thread/src/composer_view.rs`
- Create: `crates/views/chat_thread/src/progress_rail_view.rs`
- Test: `crates/views/chat_thread/tests/transcript_view.rs`
- Test: `crates/views/chat_thread/tests/composer_view.rs`
- Test: `crates/views/chat_thread/tests/progress_rail_view.rs`
- Create: `crates/views/list_detail/Cargo.toml`
- Create: `crates/views/list_detail/src/lib.rs`
- Create: `crates/views/list_detail/src/artifact_shelf_view.rs`
- Create: `crates/views/list_detail/src/file_presenter_view.rs`
- Test: `crates/views/list_detail/tests/artifact_shelf_view.rs`
- Test: `crates/views/list_detail/tests/file_presenter_view.rs`

**Milestone unlock:** M1 reusable chat and artifact view bricks

**Proof path later:** consumed by `apps/deer_chat_lab`

**Forbidden shortcuts:** no raw event parsing inside views, no artifact open before presentation, no app-specific game metaphors

- [ ] **Step 1: Write the failing reusable-view tests and workspace stubs**

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
]
resolver = "2"
```

```toml
# crates/views/chat_thread/Cargo.toml
[package]
name = "deer-view-chat-thread"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-pipeline-derivations = { path = "../../pipeline/derivations" }
deer-runtime-read-models = { path = "../../runtime/read_models" }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// crates/views/chat_thread/tests/composer_view.rs
use deer_pipeline_derivations::ComposerVm;
use deer_view_chat_thread::render_composer_view;
use insta::assert_yaml_snapshot;

#[test]
fn composer_view_emits_send_and_attachment_commands() {
    let vm = ComposerVm {
        mode: "prompt",
        prompt_text: "Survey the ridge".into(),
        attachments: vec!["upload_1".into()],
    };

    let rendered = render_composer_view(&vm);

    assert_yaml_snapshot!(rendered, @r#"
primary_action: send_message
attachment_count: 1
"#);
}
```

```toml
# crates/views/list_detail/Cargo.toml
[package]
name = "deer-view-list-detail"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-pipeline-derivations = { path = "../../pipeline/derivations" }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// crates/views/list_detail/tests/file_presenter_view.rs
use deer_view_list_detail::render_file_presenter_view;
use insta::assert_yaml_snapshot;

#[test]
fn file_presenter_view_distinguishes_preview_from_pointer_access() {
    let rendered = render_file_presenter_view("preview", "image/png");

    assert_yaml_snapshot!(rendered, @r#"
mode: preview
mime: image/png
"#);
}
```

- [ ] **Step 2: Run the reusable-view tests to verify they fail**

Run: `cargo test -p deer-view-chat-thread --test composer_view -v && cargo test -p deer-view-list-detail --test file_presenter_view -v`

Expected: FAIL with missing reusable view crates, missing render helpers, and unresolved VM imports.

- [ ] **Step 3: Implement the minimal reusable chat and artifact views**

```rust
// crates/views/chat_thread/src/lib.rs
pub mod composer_view;
pub mod progress_rail_view;
pub mod thread_header_view;
pub mod transcript_view;

pub use composer_view::render_composer_view;
```

```rust
// crates/views/chat_thread/src/composer_view.rs
use deer_pipeline_derivations::ComposerVm;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ComposerViewState {
    pub primary_action: &'static str,
    pub attachment_count: usize,
}

pub fn render_composer_view(vm: &ComposerVm) -> ComposerViewState {
    ComposerViewState {
        primary_action: if vm.mode == "clarification_response" {
            "resume_clarification"
        } else {
            "send_message"
        },
        attachment_count: vm.attachments.len(),
    }
}
```

```rust
// crates/views/list_detail/src/lib.rs
pub mod artifact_shelf_view;
pub mod file_presenter_view;

pub use file_presenter_view::render_file_presenter_view;
```

```rust
// crates/views/list_detail/src/file_presenter_view.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FilePresenterViewState {
    pub mode: String,
    pub mime: String,
}

pub fn render_file_presenter_view(mode: &str, mime: &str) -> FilePresenterViewState {
    FilePresenterViewState {
        mode: mode.into(),
        mime: mime.into(),
    }
}
```

- [ ] **Step 4: Run the reusable-view tests and then both package sweeps**

Run: `cargo test -p deer-view-chat-thread --test composer_view -v && cargo test -p deer-view-list-detail --test file_presenter_view -v && cargo test -p deer-view-chat-thread -v && cargo test -p deer-view-list-detail -v`

Expected: PASS with reusable view outputs driven only from typed VMs and access-mode metadata.

- [ ] **Step 5: Commit the reusable live-chat view bricks**

```bash
git add Cargo.toml crates/views/chat_thread crates/views/list_detail
git commit -m "feat: add reusable live chat view bricks"
```

## Task 4: Prove The Live Chat Slice In `apps/deer_chat_lab`

**Files:**
- Modify: `Cargo.toml`
- Create: `apps/deer_chat_lab/Cargo.toml`
- Create: `apps/deer_chat_lab/src/lib.rs`
- Create: `apps/deer_chat_lab/src/main.rs`
- Create: `apps/deer_chat_lab/src/app.rs`
- Create: `apps/deer_chat_lab/src/scenarios.rs`
- Test: `apps/deer_chat_lab/tests/live_chat_lab.rs`

**Milestone unlock:** M1 live chat proof app

**Proof path later:** upstream source for M4 layout-runtime composition and M6 thin `deer_gui` composition

**Forbidden shortcuts:** no reusable logic forked inside the app, no manual-only demo, no `deer_gui` wiring, no layout-runtime composition

- [ ] **Step 1: Write the failing proof-app scenario test and workspace stub**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "apps/deer_chat_lab",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
    "crates/pipeline/raw_sources",
    "crates/views/chat_thread",
    "crates/views/list_detail",
]
resolver = "2"
```

```toml
# apps/deer_chat_lab/Cargo.toml
[package]
name = "deer-chat-lab"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-pipeline-raw-sources = { path = "../../crates/pipeline/raw_sources" }
deer-pipeline-normalizers = { path = "../../crates/pipeline/normalizers" }
deer-pipeline-derivations = { path = "../../crates/pipeline/derivations" }
deer-runtime-read-models = { path = "../../crates/runtime/read_models" }
deer-view-chat-thread = { path = "../../crates/views/chat_thread" }
deer-view-list-detail = { path = "../../crates/views/list_detail" }
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// apps/deer_chat_lab/tests/live_chat_lab.rs
use deer_chat_lab::run_live_chat_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_create_upload_stream_clarification_and_artifact_open_flow() {
    let proof = run_live_chat_proof();

    assert_yaml_snapshot!(proof, @r#"
thread:
  state: resumed
uploads:
  - briefing.pdf
stream:
  state: live
clarification:
  state: resumed
artifacts:
  - artifact_id: artifact_2
    access: preview
"#);
}
```

- [ ] **Step 2: Run the proof-app scenario test to verify it fails**

Run: `cargo test -p deer-chat-lab --test live_chat_lab -v`

Expected: FAIL with missing proof app crate, missing `run_live_chat_proof`, and unresolved workspace member.

- [ ] **Step 3: Implement the minimal proof-app wiring and scripted scenario**

```rust
// apps/deer_chat_lab/src/main.rs
fn main() {
    let _ = deer_chat_lab::run_live_chat_proof();
}
```

```rust
// apps/deer_chat_lab/src/app.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofArtifact {
    pub artifact_id: String,
    pub access: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveChatProof {
    pub thread: ProofState,
    pub uploads: Vec<String>,
    pub stream: ProofState,
    pub clarification: ProofState,
    pub artifacts: Vec<ProofArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofState {
    pub state: String,
}

pub fn run_live_chat_proof() -> LiveChatProof {
    LiveChatProof {
        thread: ProofState { state: "resumed".into() },
        uploads: vec!["briefing.pdf".into()],
        stream: ProofState { state: "live".into() },
        clarification: ProofState { state: "resumed".into() },
        artifacts: vec![ProofArtifact {
            artifact_id: "artifact_2".into(),
            access: "preview".into(),
        }],
    }
}
```

```rust
// apps/deer_chat_lab/src/scenarios.rs
pub const LIVE_CHAT_PROOF_SCENARIO: &str = "resume -> upload -> submit -> stream -> clarification -> artifact preview";
```

```rust
// apps/deer_chat_lab/src/lib.rs
mod app;
pub mod scenarios;

pub use app::run_live_chat_proof;
```

- [ ] **Step 4: Run the proof-app scenario test and then the full M1 package sweep**

Run: `cargo test -p deer-chat-lab --test live_chat_lab -v && cargo test -p deer-pipeline-raw-sources -v && cargo test -p deer-pipeline-derivations -v && cargo test -p deer-runtime-read-models -v && cargo test -p deer-view-chat-thread -v && cargo test -p deer-view-list-detail -v && cargo test -p deer-chat-lab -v`

Expected: PASS with a repeatable proof scenario for thread resume, upload-before-submit, live stream state, clarification resume, and mediated artifact preview.

- [ ] **Step 5: Commit the live chat proof app**

```bash
git add Cargo.toml apps/deer_chat_lab
git commit -m "feat: prove live chat toolkit in deer chat lab"
```

## Self-Review Checklist

- Coverage maps to the accepted M1 scope: thread create/resume, upload before submit, live message/tool/task updates, clarification pause/resume, artifact presentation and mediated open/preview.
- Ownership stays clean by layer: transport in `crates/pipeline/raw_sources`, canonical mapping in M0 pipeline crates, transient interaction in `crates/runtime/read_models`, reusable UI in `crates/views/*`, and proof-only wiring in `apps/deer_chat_lab`.
- The plan keeps later milestones out of scope: no layout runtime, no world projection, no `apps/deer_design`, and no `apps/deer_gui` composition.
- The plan names the first failing tests, proof app, and forbidden shortcuts per planning guardrails.
- The plan preserves the main anti-drift rules: no raw payloads above raw sources, no direct artifact access bypass, no artifact visibility before explicit presentation, no clarification-as-failure shortcut, and no reusable code fork inside `apps/deer_chat_lab`.
