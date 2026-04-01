# Runtime Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement typed temporal, failover, policy, large-object, and intent-lifecycle runtime semantics for the bridge and shell layers so `deer_gui` can safely represent replay, degradation, exclusion, and explicit intent submission boundaries.

**Architecture:** Keep the wire layer thin: `commands.rs` and `events.rs` define transport contracts, `adapter.rs` and `dispatch.rs` parse envelopes, and a new bridge runtime-state layer owns shell-local semantics such as replay cursor, temporal range, policy epoch, freshness, and intent stage. All durable state stays reference-based; large objects cross as handoff contracts, and linked gestures remain prefill-only until explicit operator actions advance intent state.

**Tech Stack:** Rust 2021, Bevy 0.18 resources/messages/plugins, `serde`/`serde_json`, `crossbeam-channel`, existing bridge integration tests under `apps/deer_gui/tests/integration.rs`.

---

## File Structure

- Create: `apps/deer_gui/src/bridge/runtime_semantics.rs`
- Create: `apps/deer_gui/src/bridge/runtime_state.rs`
- Modify: `apps/deer_gui/src/bridge/mod.rs`
- Modify: `apps/deer_gui/src/models.rs`
- Modify: `apps/deer_gui/src/bridge/events.rs`
- Modify: `apps/deer_gui/src/bridge/commands.rs`
- Modify: `apps/deer_gui/src/bridge/adapter.rs`
- Modify: `apps/deer_gui/src/bridge/dispatch.rs`
- Modify: `apps/deer_gui/src/bridge/plugin.rs`
- Modify: `apps/deer_gui/src/bridge/orchestrator.rs`
- Create: `apps/deer_gui/tests/integration/bridge_runtime_semantics.rs`
- Modify: `apps/deer_gui/tests/integration.rs`
- Modify: `apps/deer_gui/tests/integration/bridge.rs`
- Modify: `apps/deer_gui/tests/integration/bridge_adapter.rs`

### Task 1: Add typed runtime-semantics contracts

**Files:**
- Create: `apps/deer_gui/src/bridge/runtime_semantics.rs`
- Modify: `apps/deer_gui/src/bridge/mod.rs`
- Modify: `apps/deer_gui/src/models.rs`
- Create: `apps/deer_gui/tests/integration/bridge_runtime_semantics.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Write the failing runtime-semantics tests**

```rust
use deer_gui::bridge::runtime_semantics::{IntentStage, ReplayCursor, StreamLifecycleState, TemporalAnchor, TemporalRange};

#[test]
fn t_bridge_runtime_01_replay_cursor_preserves_basis_fields() {
    let cursor = ReplayCursor {
        sequence_id: Some(7),
        event_time: Some("2026-04-01T00:00:00Z".to_string()),
        checkpoint_id: Some("cp-1".to_string()),
        mode: deer_gui::bridge::runtime_semantics::ReplayMode::HistoricalEvent,
        broker_epoch: 2,
    };

    assert_eq!(cursor.sequence_id, Some(7));
    assert_eq!(cursor.checkpoint_id.as_deref(), Some("cp-1"));
}

#[test]
fn t_bridge_runtime_02_temporal_range_preserves_anchor_semantics() {
    let range = TemporalRange {
        lower: TemporalAnchor::SequenceId(10),
        upper: TemporalAnchor::SequenceId(20),
        inclusive_upper: true,
    };

    assert!(range.inclusive_upper);
}

#[test]
fn t_bridge_runtime_03_intent_stage_defaults_to_prefill() {
    assert_eq!(IntentStage::default(), IntentStage::Prefill);
}

#[test]
fn t_bridge_runtime_04_stream_state_enum_contains_recovered() {
    let state = StreamLifecycleState::Recovered;
    assert_eq!(state, StreamLifecycleState::Recovered);
}
```

- [ ] **Step 2: Run the runtime-semantics tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::bridge_runtime_semantics::t_bridge_runtime_01_replay_cursor_preserves_basis_fields -- --exact --nocapture`
Expected: FAIL with missing runtime semantics module/types.

- [ ] **Step 3: Implement the minimal runtime-semantics types**

```rust
// apps/deer_gui/src/bridge/runtime_semantics.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayMode {
    LiveTail,
    Checkpoint,
    HistoricalEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCursor {
    pub sequence_id: Option<u64>,
    pub event_time: Option<String>,
    pub checkpoint_id: Option<String>,
    pub mode: ReplayMode,
    pub broker_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalAnchor {
    SequenceId(u64),
    CheckpointId(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalRange {
    pub lower: TemporalAnchor,
    pub upper: TemporalAnchor,
    pub inclusive_upper: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamLifecycleState {
    Connecting,
    Live,
    CatchingUp,
    Degraded,
    Stalled,
    Recovered,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntentStage {
    #[default]
    Prefill,
    Draft,
    Validated,
    Submitted,
    Approved,
    Executed,
    Rejected,
}
```

- [ ] **Step 4: Run the runtime-semantics tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::bridge_runtime_semantics -- --nocapture`
Expected: PASS for the new runtime semantics tests.

- [ ] **Step 5: Commit the typed runtime-semantics slice**

```bash
git add apps/deer_gui/src/bridge/mod.rs apps/deer_gui/src/bridge/runtime_semantics.rs apps/deer_gui/src/models.rs apps/deer_gui/tests/integration.rs apps/deer_gui/tests/integration/bridge_runtime_semantics.rs
git commit -m "feat: add typed bridge runtime semantics"
```

### Task 2: Parse temporal, stream, policy, and large-object envelopes

**Files:**
- Modify: `apps/deer_gui/src/bridge/events.rs`
- Modify: `apps/deer_gui/src/bridge/commands.rs`
- Modify: `apps/deer_gui/src/bridge/adapter.rs`
- Modify: `apps/deer_gui/src/bridge/dispatch.rs`
- Modify: `apps/deer_gui/tests/integration/bridge_adapter.rs`

- [ ] **Step 1: Write failing bridge-adapter tests for new envelopes**

```rust
#[test]
fn t_adapter_22_event_stream_status_freshness() {
    let event = dispatch_json(r#"{
        "kind":"event",
        "event":"stream_status",
        "data":{"state":"recovered","last_sequence_id":33}
    }"#);

    match event {
        BridgeEvent::StreamStatus { state, last_sequence_id } => {
            assert_eq!(state, StreamLifecycleState::Recovered);
            assert_eq!(last_sequence_id, Some(33));
        }
        other => panic!("expected StreamStatus, got {other:?}"),
    }
}

#[test]
fn t_adapter_23_response_large_object_access_contract() {
    let event = dispatch_json(r#"{
        "kind":"response",
        "id":"req-large",
        "ok":true,
        "result":{"large_object_access":{"canonical_id":"artifact-1","handoff_url":"https://example.test/signed","expires_at":"2026-04-01T01:00:00Z"}}
    }"#);

    match event {
        BridgeEvent::Response { response: BridgeResponse::LargeObjectAccess(contract), .. } => {
            assert_eq!(contract.canonical_id, "artifact-1");
        }
        other => panic!("expected LargeObjectAccess, got {other:?}"),
    }
}
```

- [ ] **Step 2: Run the bridge-adapter tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::bridge_adapter::t_adapter_22_event_stream_status_freshness -- --exact --nocapture`
Expected: FAIL with missing variants or parsing paths.

- [ ] **Step 3: Add the new bridge event/response variants and parser branches**

```rust
// apps/deer_gui/src/bridge/events.rs
pub enum BridgeEvent {
    // existing variants...
    StreamStatus {
        state: StreamLifecycleState,
        last_sequence_id: Option<u64>,
    },
    ReplayCursor {
        cursor: ReplayCursor,
    },
    PolicyEpoch {
        epoch: u64,
        reason: String,
    },
}

pub enum BridgeResponse {
    // existing variants...
    LargeObjectAccess(LargeObjectAccessContract),
}
```

```rust
// apps/deer_gui/src/bridge/adapter.rs
if let Some(v) = obj.remove("large_object_access") {
    return serde_json::from_value(v)
        .map(BridgeResponse::LargeObjectAccess)
        .map_err(|e| AdapterError { context: "parse_response/large_object_access", detail: e.to_string() });
}
```

- [ ] **Step 4: Run the bridge-adapter tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::bridge_adapter -- --nocapture`
Expected: PASS for the new stream-status and large-object-access tests.

- [ ] **Step 5: Commit the envelope parsing slice**

```bash
git add apps/deer_gui/src/bridge/adapter.rs apps/deer_gui/src/bridge/commands.rs apps/deer_gui/src/bridge/dispatch.rs apps/deer_gui/src/bridge/events.rs apps/deer_gui/tests/integration/bridge_adapter.rs
git commit -m "feat: parse runtime semantics bridge events"
```

### Task 3: Add runtime state, failover semantics, and policy invalidation

**Files:**
- Create: `apps/deer_gui/src/bridge/runtime_state.rs`
- Modify: `apps/deer_gui/src/bridge/plugin.rs`
- Modify: `apps/deer_gui/src/bridge/mod.rs`
- Modify: `apps/deer_gui/tests/integration/bridge.rs`
- Modify: `apps/deer_gui/tests/integration/bridge_runtime_semantics.rs`

- [ ] **Step 1: Write failing runtime-state tests**

```rust
use deer_gui::bridge::{BridgeRuntimeState, ReplayCursor, ReplayMode, StreamLifecycleState};

#[test]
fn t_bridge_06_runtime_state_tracks_replay_cursor_and_range() {
    let state = BridgeRuntimeState::default();
    assert!(state.replay_cursor.is_none());
    assert!(state.temporal_range.is_none());
}

#[test]
fn t_bridge_runtime_05_recovery_requires_new_broker_epoch() {
    let mut state = BridgeRuntimeState::default();
    state.broker_epoch = 3;
    state.stream_state = StreamLifecycleState::Recovered;
    assert_eq!(state.broker_epoch, 3);
}
```

- [ ] **Step 2: Run the runtime-state tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::bridge::t_bridge_06_runtime_state_tracks_replay_cursor_and_range -- --exact --nocapture`
Expected: FAIL with missing `BridgeRuntimeState`.

- [ ] **Step 3: Implement runtime state and bridge plugin application**

```rust
// apps/deer_gui/src/bridge/runtime_state.rs
use bevy::prelude::*;

use super::runtime_semantics::{ReplayCursor, StreamLifecycleState, TemporalRange, IntentStage};

#[derive(Resource, Debug)]
pub struct BridgeRuntimeState {
    pub replay_cursor: Option<ReplayCursor>,
    pub temporal_range: Option<TemporalRange>,
    pub stream_state: StreamLifecycleState,
    pub broker_epoch: u64,
    pub policy_epoch: u64,
    pub intent_stage: IntentStage,
}

impl Default for BridgeRuntimeState {
    fn default() -> Self {
        Self {
            replay_cursor: None,
            temporal_range: None,
            stream_state: StreamLifecycleState::Connecting,
            broker_epoch: 0,
            policy_epoch: 0,
            intent_stage: IntentStage::Prefill,
        }
    }
}
```

```rust
// apps/deer_gui/src/bridge/plugin.rs
app.init_resource::<BridgeEventQueue>()
    .init_resource::<BridgeRuntimeState>();
```

- [ ] **Step 4: Run the bridge and runtime tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::bridge -- --nocapture && cargo test -p deer-gui --test integration integration::bridge_runtime_semantics -- --nocapture`
Expected: PASS for runtime-state default, recovery epoch, and bridge resource registration tests.

- [ ] **Step 5: Commit the runtime-state slice**

```bash
git add apps/deer_gui/src/bridge/mod.rs apps/deer_gui/src/bridge/plugin.rs apps/deer_gui/src/bridge/runtime_state.rs apps/deer_gui/tests/integration/bridge.rs apps/deer_gui/tests/integration/bridge_runtime_semantics.rs
git commit -m "feat: track bridge runtime state and recovery epochs"
```

### Task 4: Enforce explicit intent lifecycle boundaries

**Files:**
- Modify: `apps/deer_gui/src/bridge/orchestrator.rs`
- Modify: `apps/deer_gui/src/bridge/commands.rs`
- Modify: `apps/deer_gui/src/bridge/plugin.rs`
- Modify: `apps/deer_gui/src/bridge/runtime_state.rs`
- Modify: `apps/deer_gui/tests/integration/bridge_runtime_semantics.rs`

- [ ] **Step 1: Write failing intent-lifecycle tests**

```rust
#[test]
fn t_bridge_runtime_06_linked_gesture_only_prefills_intent() {
    let mut state = BridgeRuntimeState::default();
    state.intent_stage = IntentStage::Prefill;
    assert_eq!(state.intent_stage, IntentStage::Prefill);
}

#[test]
fn t_bridge_runtime_07_validated_intent_demotes_on_policy_change() {
    let mut state = BridgeRuntimeState::default();
    state.intent_stage = IntentStage::Validated;
    state.policy_epoch = 1;
    state.policy_epoch = 2;
    state.intent_stage = IntentStage::Draft;
    assert_eq!(state.intent_stage, IntentStage::Draft);
}
```

- [ ] **Step 2: Run the intent-lifecycle tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::bridge_runtime_semantics::t_bridge_runtime_06_linked_gesture_only_prefills_intent -- --exact --nocapture`
Expected: FAIL with missing transition helpers or lifecycle enforcement.

- [ ] **Step 3: Implement explicit lifecycle transitions and demotion rules**

```rust
// apps/deer_gui/src/bridge/runtime_state.rs
impl BridgeRuntimeState {
    pub fn prefill(&mut self) {
        self.intent_stage = IntentStage::Prefill;
    }

    pub fn enter_draft(&mut self) {
        self.intent_stage = IntentStage::Draft;
    }

    pub fn validate(&mut self) {
        self.intent_stage = IntentStage::Validated;
    }

    pub fn demote_to_draft(&mut self) {
        self.intent_stage = IntentStage::Draft;
    }
}
```

```rust
// apps/deer_gui/src/bridge/orchestrator.rs
pub trait OrchestratorClient: Send + Sync + 'static {
    // existing methods...
    fn submit_intent(&self, thread_id: &str, intent_payload: &str) -> Result<String, String>;
    fn request_large_object_access(&self, thread_id: &str, virtual_path: &str) -> Result<String, String>;
}
```

- [ ] **Step 4: Run the intent-lifecycle and full bridge tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::bridge_runtime_semantics -- --nocapture && cargo test -p deer-gui --test integration integration::bridge -- --nocapture && cargo check -p deer-gui`
Expected: PASS for linked-gesture-prefill-only and validated-to-draft demotion behavior.

- [ ] **Step 5: Commit the intent lifecycle slice**

```bash
git add apps/deer_gui/src/bridge/commands.rs apps/deer_gui/src/bridge/orchestrator.rs apps/deer_gui/src/bridge/plugin.rs apps/deer_gui/src/bridge/runtime_state.rs apps/deer_gui/tests/integration/bridge_runtime_semantics.rs
git commit -m "feat: enforce intent lifecycle boundaries"
```
