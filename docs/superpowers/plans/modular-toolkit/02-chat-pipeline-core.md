# Chat Pipeline Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the reusable DeerFlow chat/artifact ingestion pipeline from existing bridge/client transport through canonical domain and typed view models.

**Architecture:** Reuse the existing DeerFlow Python client and current bridge transport instead of inventing a new protocol stack. Add raw-source and normalizer crates that translate DeerFlow stream events into canonical records, then derive transcript, artifact-shelf, run-status, and task-progress view models.

**Tech Stack:** Existing `apps/deer_gui/src/bridge/*`, DeerFlow Python client, `serde_json`, `crossbeam-channel`, `camino`, `bytes`, `smallvec`, `tracing`.

---

## File Map

| Action | Path | Responsibility |
| --- | --- | --- |
| Create | `crates/pipeline/raw_sources/Cargo.toml` | raw-source crate manifest |
| Create | `crates/pipeline/raw_sources/src/lib.rs` | DeerFlow source trait + bridge-backed source |
| Create | `crates/pipeline/raw_sources/tests/raw_sources.rs` | fixture-backed source tests |
| Create | `crates/pipeline/normalizers/Cargo.toml` | normalizer crate manifest |
| Create | `crates/pipeline/normalizers/src/lib.rs` | raw event -> canonical record translation |
| Create | `crates/pipeline/normalizers/tests/normalizers.rs` | stream fixture normalization tests |
| Modify | `crates/pipeline/derivations/src/lib.rs` | add transcript/artifact/task/status VMs |
| Create | `crates/pipeline/derivations/tests/chat_vms.rs` | VM shaping tests |
| Create | `crates/pipeline/raw_sources/tests/fixtures/deerflow/thread_values.json` | snapshot fixture |
| Create | `crates/pipeline/raw_sources/tests/fixtures/deerflow/task_running.json` | custom event fixture |

## Task 1: Add `deer-pipeline-raw-sources`

**Files:**
- Create: `crates/pipeline/raw_sources/Cargo.toml`
- Create: `crates/pipeline/raw_sources/src/lib.rs`
- Create: `crates/pipeline/raw_sources/tests/raw_sources.rs`

- [ ] **Step 1: Write the failing raw-source fixture test**

```rust
use deer_pipeline_raw_sources::{DeerFlowRawEnvelope, FixtureRawSource, RawEventSource};

#[test]
fn fixture_source_emits_values_then_custom_event() {
    let source = FixtureRawSource::from_files(&[
        "tests/fixtures/deerflow/thread_values.json",
        "tests/fixtures/deerflow/task_running.json",
    ]);

    let events = source.collect().unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], DeerFlowRawEnvelope::Values { .. }));
    assert!(matches!(events[1], DeerFlowRawEnvelope::Custom { .. }));
}
```

- [ ] **Step 2: Write the crate manifest and minimal source interface**

```toml
[package]
name = "deer-pipeline-raw-sources"
version = "0.1.0"
edition = "2021"

[dependencies]
camino = { workspace = true }
crossbeam-channel = { workspace = true }
deer-foundation-contracts = { path = "../../foundation/contracts" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```
```rust
use serde::{Deserialize, Serialize};

pub trait RawEventSource {
    fn collect(&self) -> Result<Vec<DeerFlowRawEnvelope>, RawSourceError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeerFlowRawEnvelope {
    Values { payload: serde_json::Value },
    MessageTuple { payload: serde_json::Value },
    Custom { payload: serde_json::Value },
}
```

- [ ] **Step 3: Reuse the existing bridge transport concept in a bridge-backed source stub**

```rust
pub struct BridgeRawSource<T> {
    transport: T,
}

impl<T> BridgeRawSource<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}
```

- [ ] **Step 4: Run the raw-source tests**

Run: `cargo test -p deer-pipeline-raw-sources --test raw_sources`

Expected: PASS

## Task 2: Add `deer-pipeline-normalizers`

**Files:**
- Create: `crates/pipeline/normalizers/Cargo.toml`
- Create: `crates/pipeline/normalizers/src/lib.rs`
- Create: `crates/pipeline/normalizers/tests/normalizers.rs`

- [ ] **Step 1: Write the failing normalizer test first**

```rust
use deer_pipeline_normalizers::normalize_envelopes;
use deer_pipeline_raw_sources::FixtureRawSource;

#[test]
fn normalizer_produces_thread_session_and_progress_records() {
    let raw = FixtureRawSource::from_files(&[
        "../raw_sources/tests/fixtures/deerflow/thread_values.json",
        "../raw_sources/tests/fixtures/deerflow/task_running.json",
    ]);

    let result = normalize_envelopes(&raw.collect().unwrap()).unwrap();
    assert_eq!(result.threads.len(), 1);
    assert_eq!(result.progress.len(), 1);
}
```

- [ ] **Step 2: Write the minimal normalizer implementation**

```rust
use deer_foundation_domain::{ArtifactRecord, MessageKind, MessageRecord, ThreadSession};
use deer_pipeline_raw_sources::DeerFlowRawEnvelope;

pub struct NormalizedBatch {
    pub threads: Vec<ThreadSession>,
    pub progress: Vec<String>,
}

pub fn normalize_envelopes(
    raw: &[DeerFlowRawEnvelope],
) -> Result<NormalizedBatch, NormalizerError> {
    let mut threads = Vec::new();
    let mut progress = Vec::new();

    for envelope in raw {
        match envelope {
            DeerFlowRawEnvelope::Values { payload } => {
                threads.push(ThreadSession::from_values_payload(payload)?);
            }
            DeerFlowRawEnvelope::Custom { payload } => {
                progress.push(payload.to_string());
            }
            DeerFlowRawEnvelope::MessageTuple { .. } => {}
        }
    }

    Ok(NormalizedBatch { threads, progress })
}
```

- [ ] **Step 3: Add canonical constructors in `deer-foundation-domain` as needed and rerun tests**

Run: `cargo test -p deer-pipeline-normalizers --test normalizers`

Expected: PASS

## Task 3: Extend Derivations For Chat VMs

**Files:**
- Modify: `crates/pipeline/derivations/src/lib.rs`
- Create: `crates/pipeline/derivations/tests/chat_vms.rs`

- [ ] **Step 1: Write the failing chat VM test**

```rust
use deer_pipeline_derivations::build_chat_surface_vms;
use deer_pipeline_normalizers::normalize_envelopes;
use deer_pipeline_raw_sources::FixtureRawSource;

#[test]
fn derivations_build_transcript_artifact_and_progress_vms() {
    let raw = FixtureRawSource::from_files(&[
        "../raw_sources/tests/fixtures/deerflow/thread_values.json",
        "../raw_sources/tests/fixtures/deerflow/task_running.json",
    ]);
    let batch = normalize_envelopes(&raw.collect().unwrap()).unwrap();
    let vms = build_chat_surface_vms(&batch).unwrap();

    assert!(!vms.transcript.rows.is_empty());
    assert_eq!(vms.progress.items.len(), 1);
}
```

- [ ] **Step 2: Add the minimal chat VM bundle**

```rust
#[derive(Clone, Debug)]
pub struct ArtifactItemVm {
    pub label: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct ArtifactShelfVm {
    pub items: Vec<ArtifactItemVm>,
}

#[derive(Clone, Debug)]
pub struct TaskProgressVm {
    pub items: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ChatSurfaceVms {
    pub transcript: TranscriptVm,
    pub artifacts: ArtifactShelfVm,
    pub progress: TaskProgressVm,
}
```

- [ ] **Step 3: Run the derivation verification**

Run: `cargo test -p deer-pipeline-derivations --test chat_vms`

Expected: PASS

## Success Checks

- the chat pipeline uses existing DeerFlow bridge/client concepts instead of a new Rust protocol stack
- raw DeerFlow events stop at `raw_sources` and `normalizers`
- canonical records and VMs are sufficient for transcript, artifacts, and task progress
- all tests are fixture-backed and repeatable

## Done State

This file is done only when a DeerFlow fixture sequence can flow from raw events to canonical records to chat surface VMs with passing tests.
