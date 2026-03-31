# Workspace And Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the workspace, shared dependency policy, and minimal foundational crates required for all later toolkit slices.

**Architecture:** Start by widening the Cargo workspace, centralizing battle-tested dependencies, and creating small focused crates for contracts, canonical domain, replay, derivations, and read-model state. Keep reusable crates pure and app-agnostic; reuse the existing `apps/deer_gui` integration-test style for real, fixture-backed testing.

**Tech Stack:** Cargo workspace, Rust 2021, `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`, `tracing`, `camino`, `smallvec`, `bytes`, Bevy integration tests.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Modify | `Cargo.toml` | widen workspace members and centralize shared versions |
| Create | `crates/foundation/contracts/Cargo.toml` | shared contract crate manifest |
| Create | `crates/foundation/contracts/src/lib.rs` | IDs, traits, commands, event envelopes |
| Create | `crates/foundation/contracts/tests/contracts.rs` | serde roundtrip + trait fixture tests |
| Create | `crates/foundation/domain/Cargo.toml` | canonical domain manifest |
| Create | `crates/foundation/domain/src/lib.rs` | thread, message, artifact, agent, queue records |
| Create | `crates/foundation/domain/tests/domain.rs` | invariants and linkage tests |
| Create | `crates/foundation/replay/Cargo.toml` | replay manifest |
| Create | `crates/foundation/replay/src/lib.rs` | replay record and cursor structs |
| Create | `crates/foundation/replay/tests/replay.rs` | replay cursor tests |
| Create | `crates/pipeline/derivations/Cargo.toml` | derivation manifest |
| Create | `crates/pipeline/derivations/src/lib.rs` | base VM structs |
| Create | `crates/pipeline/derivations/tests/derivations.rs` | fixture-to-VM coverage |
| Create | `crates/runtime/read_models/Cargo.toml` | read-model manifest |
| Create | `crates/runtime/read_models/src/lib.rs` | transient UI state reducers |
| Create | `crates/runtime/read_models/tests/read_models.rs` | reducer and optimistic-state tests |

## Task 1: Widen The Cargo Workspace

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Write the failing workspace check by creating the target member list in the root manifest**

Replace the root file with:

```toml
[workspace]
members = [
    "apps/deer_gui",
    "apps/deer_chat_lab",
    "apps/deer_replay",
    "apps/deer_graph_lab",
    "apps/deer_design",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/raw_sources",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
    "crates/runtime/world_projection",
    "crates/views/chat_thread",
    "crates/views/timeline_view",
    "crates/views/graph_view",
    "crates/views/list_detail",
    "crates/views/telemetry_view",
    "crates/ui/panel_shells",
    "crates/ui/layout_runtime",
]
resolver = "2"

[workspace.dependencies]
bevy = { version = "0.18.1", default-features = false, features = [
    "bevy_asset", "bevy_audio", "bevy_core_pipeline", "bevy_gizmos",
    "bevy_log", "bevy_mesh", "bevy_pbr", "bevy_picking", "bevy_render",
    "bevy_scene", "bevy_state", "bevy_winit", "default_font",
    "multi_threaded", "png", "vorbis", "wayland", "x11",
] }
bevy_egui = "0.39"
bevy_hanabi = "0.18"
bevy_tweening = "0.15"
bytes = "1"
camino = "1"
chrono = { version = "0.4.44", features = ["clock", "serde"] }
crossbeam-channel = "0.5.15"
egui_dock = "0.14"
petgraph = "0.8"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
smallvec = "1"
thiserror = "2"
tracing = "0.1"
uuid = { version = "1.23.0", features = ["serde", "v4"] }
```

- [ ] **Step 2: Run the workspace check and verify it fails for missing members**

Run: `cargo metadata --format-version 1`

Expected: FAIL with missing workspace member errors for the crates and apps that do not exist yet.

- [ ] **Step 3: Commit the manifest expansion after the missing crates are added later in this file**

```bash
git add Cargo.toml
git commit -m "chore: prepare modular toolkit workspace"
```

## Task 2: Create `deer-foundation-contracts`

**Files:**
- Create: `crates/foundation/contracts/Cargo.toml`
- Create: `crates/foundation/contracts/src/lib.rs`
- Create: `crates/foundation/contracts/tests/contracts.rs`

- [ ] **Step 1: Write the failing integration test first**

```rust
use deer_foundation_contracts::{ArtifactId, MessageId, ThreadId};

#[test]
fn ids_roundtrip_through_serde() {
    let payload = serde_json::json!({
        "thread_id": ThreadId::new(),
        "message_id": MessageId::new(),
        "artifact_id": ArtifactId::new(),
    });

    let text = serde_json::to_string(&payload).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert!(value.get("thread_id").is_some());
    assert!(value.get("message_id").is_some());
    assert!(value.get("artifact_id").is_some());
}
```

- [ ] **Step 2: Add the crate manifest**

```toml
[package]
name = "deer-foundation-contracts"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
```

- [ ] **Step 3: Write the minimal contract module**

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }
    };
}

id_type!(ThreadId);
id_type!(MessageId);
id_type!(ArtifactId);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamCheckpoint {
    pub thread_id: ThreadId,
    pub sequence: u64,
}
```

- [ ] **Step 4: Run the contracts test and verify it passes**

Run: `cargo test -p deer-foundation-contracts --test contracts`

Expected: PASS

## Task 3: Create `deer-foundation-domain` And `deer-foundation-replay`

**Files:**
- Create: `crates/foundation/domain/Cargo.toml`
- Create: `crates/foundation/domain/src/lib.rs`
- Create: `crates/foundation/domain/tests/domain.rs`
- Create: `crates/foundation/replay/Cargo.toml`
- Create: `crates/foundation/replay/src/lib.rs`
- Create: `crates/foundation/replay/tests/replay.rs`

- [ ] **Step 1: Write failing domain and replay tests**

```rust
use deer_foundation_domain::{ArtifactRecord, MessageKind, MessageRecord, ThreadSession};

#[test]
fn thread_links_messages_and_artifacts() {
    let session = ThreadSession::fixture();
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.artifacts.len(), 1);
    assert_eq!(session.messages[0].kind, MessageKind::Human);
}
```

```rust
use deer_foundation_replay::ReplayCursor;

#[test]
fn replay_cursor_moves_without_skipping_bounds() {
    let mut cursor = ReplayCursor::new(3);
    cursor.advance();
    cursor.advance();
    cursor.advance();
    assert_eq!(cursor.index(), 2);
}
```

- [ ] **Step 2: Write the minimal domain models**

```rust
use deer_foundation_contracts::{ArtifactId, MessageId, ThreadId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageKind {
    Human,
    Assistant,
    ToolCall,
    ToolResult,
    Clarification,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: MessageId,
    pub thread_id: ThreadId,
    pub kind: MessageKind,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: ArtifactId,
    pub thread_id: ThreadId,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadSession {
    pub id: ThreadId,
    pub title: String,
    pub messages: Vec<MessageRecord>,
    pub artifacts: Vec<ArtifactRecord>,
}
```

- [ ] **Step 3: Write the minimal replay model**

```rust
#[derive(Clone, Debug)]
pub struct ReplayCursor {
    len: usize,
    index: usize,
}

impl ReplayCursor {
    pub fn new(len: usize) -> Self {
        Self { len, index: 0 }
    }

    pub fn advance(&mut self) {
        if self.index + 1 < self.len {
            self.index += 1;
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
```

- [ ] **Step 4: Run the foundation tests**

Run: `cargo test -p deer-foundation-domain --test domain && cargo test -p deer-foundation-replay --test replay`

Expected: PASS

## Task 4: Create Base Derivations And Read Models

**Files:**
- Create: `crates/pipeline/derivations/Cargo.toml`
- Create: `crates/pipeline/derivations/src/lib.rs`
- Create: `crates/pipeline/derivations/tests/derivations.rs`
- Create: `crates/runtime/read_models/Cargo.toml`
- Create: `crates/runtime/read_models/src/lib.rs`
- Create: `crates/runtime/read_models/tests/read_models.rs`

- [ ] **Step 1: Write failing tests for a transcript VM and a draft reducer**

```rust
use deer_pipeline_derivations::TranscriptVm;

#[test]
fn transcript_vm_preserves_message_order() {
    let vm = TranscriptVm::fixture();
    assert_eq!(vm.rows.len(), 2);
    assert!(vm.rows[0].timestamp <= vm.rows[1].timestamp);
}
```

```rust
use deer_runtime_read_models::ComposerState;

#[test]
fn composer_state_tracks_draft_text() {
    let mut state = ComposerState::default();
    state.set_draft("research agents".into());
    assert_eq!(state.draft_text(), "research agents");
}
```

- [ ] **Step 2: Write the minimal VM and reducer code**

```rust
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct TranscriptRowVm {
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct TranscriptVm {
    pub rows: Vec<TranscriptRowVm>,
}
```

```rust
#[derive(Default, Clone, Debug)]
pub struct ComposerState {
    draft_text: String,
}

impl ComposerState {
    pub fn set_draft(&mut self, value: String) {
        self.draft_text = value;
    }

    pub fn draft_text(&self) -> &str {
        &self.draft_text
    }
}
```

- [ ] **Step 3: Run the foundation verification**

Run: `cargo test -p deer-pipeline-derivations --test derivations && cargo test -p deer-runtime-read-models --test read_models`

Expected: PASS

## Success Checks

- `cargo metadata --format-version 1` succeeds
- all foundation crates compile independently
- every later plan file can depend on typed IDs, canonical records, replay cursor, and basic transient state
- no crate imports `apps/deer_gui`

## Done State

This file is done only when the workspace has real crate boundaries, shared dependency policy, passing tests, and no missing-member errors.
