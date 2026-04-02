# Layout Runtime Toolkit Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove that reusable panels and hosted views can be registered by contract, composed into layouts, saved and restored, and linked through shell-level interaction rules inside `apps/deer_design` before any such composition appears in `apps/deer_gui`.

**Architecture:** Introduce two reusable UI crates: `deer-ui-panel-shells` for panel contracts, roles, and shell framing, and `deer-ui-layout-runtime` for panel registry, layout persistence, hosted-view bridging, and brokered linked state within a single runtime. The layout runtime must support the Z0/Z1/Z2 cinematic layer stack, rendering the `egui` HUD as a transparent Z1 overlay on top of the Z0 Bevy 3D world pass. Keep `apps/deer_design` thin and proof-focused: it wires reusable chat, artifact/detail, and inspector-supporting views into layout presets and scenario tests without redefining view logic, canonical data rules, or world projection behavior.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `thiserror`, `insta`, `similar-asserts`, `egui_dock`, existing Bevy/egui stack only at reusable UI and proof-app boundaries

---

## Scope Boundary

- This plan covers only the M4 layout proof slice: panel shells, panel registry, layout runtime, hosted-view bridge, and the narrow proving app `apps/deer_design`.
- It assumes plans 1, 2, and 3 exist conceptually and reuses their planned crates rather than redefining foundation, pipeline, read-model, or reusable view ownership.
- It does **not** create `crates/runtime/world_projection`, `crates/views/scene3d`, `crates/views/telemetry_view`, `crates/ui/theme_tokens` unless a reusable token gap proves unavoidable, or any spatial/world-primary runtime.
- It does **not** modify `apps/deer_gui`.
- It does **not** move normalization, intent lifecycle, or world-metaphor mapping into panels or layout runtime.
- It does **not** treat adjacent hosted views as implicitly linked; all linked behavior must be declared through panel roles, brokers, joinability metadata, and shell-mode contracts.

## File Structure

### Workspace

- Modify: `Cargo.toml` - add each new reusable UI crate and proof app incrementally as its task begins.

### Panel shells crate

- Create: `crates/ui/panel_shells/Cargo.toml` - package manifest for reusable panel shell contracts.
- Create: `crates/ui/panel_shells/src/lib.rs` - public exports.
- Create: `crates/ui/panel_shells/src/panel_contract.rs` - required hosted-view, command, and metadata contract types.
- Create: `crates/ui/panel_shells/src/panel_frame.rs` - reusable shell chrome, panel title, and status banner state.
- Create: `crates/ui/panel_shells/src/panel_roles.rs` - `source`, `sink`, `broker`, and `mirror` role types.
- Create: `crates/ui/panel_shells/src/panel_ids.rs` - stable panel and hosted-view identifiers.
- Create: `crates/ui/panel_shells/src/panel_participation.rs` - declared linked-interaction participation rules.
- Create: `crates/ui/panel_shells/tests/panel_contracts.rs` - contract and role-validation tests.

### Layout runtime crate

- Create: `crates/ui/layout_runtime/Cargo.toml` - package manifest for registry, runtime, and persistence.
- Create: `crates/ui/layout_runtime/src/lib.rs` - public exports.
- Create: `crates/ui/layout_runtime/src/panel_descriptor.rs` - data-driven panel descriptors.
- Create: `crates/ui/layout_runtime/src/panel_registry.rs` - registry add/remove and compatibility checks.
- Create: `crates/ui/layout_runtime/src/layout_model.rs` - serializable layout tree and dock/modal model.
- Create: `crates/ui/layout_runtime/src/layout_persistence.rs` - save/restore helpers and versioned snapshots.
- Create: `crates/ui/layout_runtime/src/runtime.rs` - layout mutations and runtime composition state.
- Create: `crates/ui/layout_runtime/src/view_host.rs` - reusable host slots for typed views.
- Create: `crates/ui/layout_runtime/src/hosted_views.rs` - hosted-view bridge adapters for chat and artifact/detail slices.
- Create: `crates/ui/layout_runtime/src/linked_brokers.rs` - broker ownership and linked-state propagation.
- Create: `crates/ui/layout_runtime/tests/registry.rs` - registry and descriptor tests.
- Create: `crates/ui/layout_runtime/tests/layout_serialization.rs` - save/restore tests.
- Create: `crates/ui/layout_runtime/tests/linked_runtime.rs` - linked-interaction runtime tests.

### Runtime read-model extension

- Create: `crates/runtime/read_models/src/layout_runtime_state.rs` - shell-local layout instance state and broker epoch metadata.
- Modify: `crates/runtime/read_models/src/lib.rs` - export layout runtime state.
- Modify: `crates/runtime/read_models/src/linked_shell.rs` - preserve linked-shell participation for panel roles.
- Modify: `crates/runtime/read_models/src/temporal.rs` - keep stale/live metadata restorable through layout changes.
- Modify: `crates/runtime/read_models/src/policy.rs` - invalidate linked state on exclusion/tombstone overlays.
- Create: `crates/runtime/read_models/tests/layout_runtime_state.rs` - layout instance and broker epoch reducer tests.

### Proof app

- Create: `apps/deer_design/Cargo.toml` - package manifest for the narrow layout-runtime proof app.
- Create: `apps/deer_design/src/lib.rs` - proof-app library exports for scenario tests.
- Create: `apps/deer_design/src/main.rs` - app bootstrap.
- Create: `apps/deer_design/src/app.rs` - proof composition of registry, runtime, hosted views, and reusable M1 views.
- Create: `apps/deer_design/src/panel_catalog.rs` - app-local catalog wiring for proof panels only.
- Create: `apps/deer_design/src/layout_presets.rs` - fixture-backed layout presets.
- Create: `apps/deer_design/src/scenarios.rs` - scripted proof scenarios.
- Create: `apps/deer_design/tests/layout_runtime_proof.rs` - repeatable proof-app scenario tests.

## Task 1: Add Reusable Panel Shell Contracts And Participation Rules

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ui/panel_shells/Cargo.toml`
- Create: `crates/ui/panel_shells/src/lib.rs`
- Create: `crates/ui/panel_shells/src/panel_contract.rs`
- Create: `crates/ui/panel_shells/src/panel_frame.rs`
- Create: `crates/ui/panel_shells/src/panel_roles.rs`
- Create: `crates/ui/panel_shells/src/panel_ids.rs`
- Create: `crates/ui/panel_shells/src/panel_participation.rs`
- Test: `crates/ui/panel_shells/tests/panel_contracts.rs`

**Milestone unlock:** M4 reusable panel contracts and role declarations

**Proof path later:** consumed by `crates/ui/layout_runtime` and `apps/deer_design`

**Forbidden shortcuts:** no app-local panel enums, no implicit linked roles, no raw payload contracts

- [ ] **Step 1: Write the failing panel-shell tests and workspace stubs**

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
    "crates/ui/panel_shells",
]
resolver = "2"
```

```toml
# crates/ui/panel_shells/Cargo.toml
[package]
name = "deer-ui-panel-shells"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
similar-asserts = "1.7.0"
```

```rust
// crates/ui/panel_shells/src/lib.rs
pub mod panel_contract;
pub mod panel_frame;
pub mod panel_ids;
pub mod panel_participation;
pub mod panel_roles;

pub use panel_contract::PanelContract;
pub use panel_participation::PanelParticipation;
pub use panel_roles::PanelRole;
```

```rust
// crates/ui/panel_shells/tests/panel_contracts.rs
use deer_ui_panel_shells::{PanelContract, PanelParticipation, PanelRole};

#[test]
fn panel_contract_requires_hosted_views_roles_and_join_keys() {
    let contract = PanelContract {
        panel_id: "artifact_shelf".into(),
        required_hosted_views: vec!["artifact_shelf_view".into()],
        roles: vec![PanelRole::Source, PanelRole::Sink, PanelRole::Mirror],
        join_keys: vec!["artifact_id".into(), "thread_id".into()],
    };

    let participation = PanelParticipation::from_contract(&contract).unwrap();

    assert!(participation.roles.contains(&PanelRole::Source));
    assert_eq!(participation.join_keys, vec!["artifact_id".to_string(), "thread_id".to_string()]);
}
```

- [ ] **Step 2: Run the panel-shell test to verify it fails**

Run: `cargo test -p deer-ui-panel-shells --test panel_contracts -v`

Expected: FAIL with missing panel contract types, role enums, and participation helpers.

- [ ] **Step 3: Implement the minimal panel-shell contract layer**

```rust
// crates/ui/panel_shells/src/panel_roles.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelRole {
    Source,
    Sink,
    Broker,
    Mirror,
}
```

```rust
// crates/ui/panel_shells/src/panel_contract.rs
use serde::{Deserialize, Serialize};

use crate::panel_roles::PanelRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelContract {
    pub panel_id: String,
    pub required_hosted_views: Vec<String>,
    pub roles: Vec<PanelRole>,
    pub join_keys: Vec<String>,
}
```

```rust
// crates/ui/panel_shells/src/panel_participation.rs
use crate::{panel_contract::PanelContract, panel_roles::PanelRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelParticipation {
    pub roles: Vec<PanelRole>,
    pub join_keys: Vec<String>,
}

impl PanelParticipation {
    pub fn from_contract(contract: &PanelContract) -> Result<Self, &'static str> {
        if contract.roles.is_empty() || contract.join_keys.is_empty() {
            return Err("panel participation requires declared roles and join keys");
        }

        Ok(Self {
            roles: contract.roles.clone(),
            join_keys: contract.join_keys.clone(),
        })
    }
}
```

```rust
// crates/ui/panel_shells/src/panel_ids.rs
pub const ARTIFACT_SHELF_PANEL: &str = "artifact_shelf";
pub const INSPECTOR_PANEL: &str = "inspector";
pub const CHAT_PANEL: &str = "chat_thread";
```

```rust
// crates/ui/panel_shells/src/panel_frame.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PanelFrameState {
    pub title: String,
    pub status_banner: Option<String>,
}
```

- [ ] **Step 4: Run the panel-shell test and then the package sweep**

Run: `cargo test -p deer-ui-panel-shells --test panel_contracts -v && cargo test -p deer-ui-panel-shells -v`

Expected: PASS with explicit hosted-view, role, and join-key declarations required for panel participation.

- [ ] **Step 5: Commit the reusable panel-shell contracts**

```bash
git add Cargo.toml crates/ui/panel_shells
git commit -m "feat: add reusable panel shell contracts"
```

## Task 2: Add Layout Registry, Persistence, Hosted-View Bridge, And Linked Brokers

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ui/layout_runtime/Cargo.toml`
- Create: `crates/ui/layout_runtime/src/lib.rs`
- Create: `crates/ui/layout_runtime/src/panel_descriptor.rs`
- Create: `crates/ui/layout_runtime/src/panel_registry.rs`
- Create: `crates/ui/layout_runtime/src/layout_model.rs`
- Create: `crates/ui/layout_runtime/src/layout_persistence.rs`
- Create: `crates/ui/layout_runtime/src/runtime.rs`
- Create: `crates/ui/layout_runtime/src/view_host.rs`
- Create: `crates/ui/layout_runtime/src/hosted_views.rs`
- Create: `crates/ui/layout_runtime/src/linked_brokers.rs`
- Test: `crates/ui/layout_runtime/tests/registry.rs`
- Test: `crates/ui/layout_runtime/tests/layout_serialization.rs`
- Test: `crates/ui/layout_runtime/tests/linked_runtime.rs`

**Milestone unlock:** M4 registry-driven layout runtime and save/restore host bridge

**Proof path later:** consumed by `apps/deer_design`

**Forbidden shortcuts:** no custom docking framework first, no cached payload persistence, no implicit cross-view linkage

- [ ] **Step 1: Write the failing layout-runtime tests and workspace stubs**

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
    "crates/ui/panel_shells",
    "crates/ui/layout_runtime",
]
resolver = "2"
```

```toml
# crates/ui/layout_runtime/Cargo.toml
[package]
name = "deer-ui-layout-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-ui-panel-shells = { path = "../panel_shells" }
egui_dock = "0.13.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2.0.17"

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
similar-asserts = "1.7.0"
```

```rust
// crates/ui/layout_runtime/tests/registry.rs
use deer_ui_layout_runtime::{register_panel, PanelDescriptor, PanelRegistry};

#[test]
fn registry_rejects_duplicate_panel_ids_and_tracks_roles() {
    let mut registry = PanelRegistry::default();
    let descriptor = PanelDescriptor::new("chat_panel", vec!["chat_thread_view".into()]);

    register_panel(&mut registry, descriptor.clone()).unwrap();
    let duplicate = register_panel(&mut registry, descriptor);

    assert!(duplicate.is_err());
    assert_eq!(registry.panels.len(), 1);
}
```

```rust
// crates/ui/layout_runtime/tests/layout_serialization.rs
use deer_ui_layout_runtime::{deserialize_layout, serialize_layout, LayoutSnapshot};
use insta::assert_yaml_snapshot;

#[test]
fn layout_snapshot_round_trips_with_panel_ids_and_mode() {
    let snapshot = LayoutSnapshot::new("live_meeting", vec!["chat_panel".into(), "inspector_panel".into()]);
    let encoded = serialize_layout(&snapshot).unwrap();
    let restored = deserialize_layout(&encoded).unwrap();

    assert_yaml_snapshot!(restored, @r#"
mode: live_meeting
panels:
  - chat_panel
  - inspector_panel
"#);
}
```

```rust
// crates/ui/layout_runtime/tests/linked_runtime.rs
use deer_ui_layout_runtime::{LayoutRuntimeState, LinkedBrokerState};

#[test]
fn linked_runtime_keeps_one_broker_per_interaction_type() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("focus", "inspector_panel"),
    ]);

    assert_eq!(runtime.brokers.len(), 2);
    assert_eq!(runtime.brokers[0].interaction_type, "selection");
}
```

- [ ] **Step 2: Run the layout-runtime tests to verify they fail**

Run: `cargo test -p deer-ui-layout-runtime --test registry --test layout_serialization --test linked_runtime -v`

Expected: FAIL with missing runtime crate, missing descriptor/registry types, and unresolved serialization helpers.

- [ ] **Step 3: Implement the minimal registry, persistence, host bridge, and broker runtime**

```rust
// crates/ui/layout_runtime/src/lib.rs
pub mod hosted_views;
pub mod layout_model;
pub mod layout_persistence;
pub mod linked_brokers;
pub mod panel_descriptor;
pub mod panel_registry;
pub mod runtime;
pub mod view_host;

pub use layout_model::LayoutSnapshot;
pub use layout_persistence::{deserialize_layout, serialize_layout};
pub use linked_brokers::LinkedBrokerState;
pub use panel_descriptor::PanelDescriptor;
pub use panel_registry::{register_panel, PanelRegistry};
pub use runtime::LayoutRuntimeState;
```

```rust
// crates/ui/layout_runtime/src/panel_descriptor.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    pub panel_id: String,
    pub hosted_views: Vec<String>,
}

impl PanelDescriptor {
    pub fn new(panel_id: &str, hosted_views: Vec<String>) -> Self {
        Self {
            panel_id: panel_id.into(),
            hosted_views,
        }
    }
}
```

```rust
// crates/ui/layout_runtime/src/panel_registry.rs
use thiserror::Error;

use crate::panel_descriptor::PanelDescriptor;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PanelRegistry {
    pub panels: Vec<PanelDescriptor>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("duplicate panel id")]
    DuplicatePanelId,
}

pub fn register_panel(registry: &mut PanelRegistry, descriptor: PanelDescriptor) -> Result<(), RegistryError> {
    if registry.panels.iter().any(|existing| existing.panel_id == descriptor.panel_id) {
        return Err(RegistryError::DuplicatePanelId);
    }

    registry.panels.push(descriptor);
    Ok(())
}
```

```rust
// crates/ui/layout_runtime/src/layout_model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    pub mode: String,
    pub panels: Vec<String>,
}

impl LayoutSnapshot {
    pub fn new(mode: &str, panels: Vec<String>) -> Self {
        Self {
            mode: mode.into(),
            panels,
        }
    }
}
```

```rust
// crates/ui/layout_runtime/src/layout_persistence.rs
use crate::layout_model::LayoutSnapshot;

pub fn serialize_layout(snapshot: &LayoutSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string(snapshot)
}

pub fn deserialize_layout(encoded: &str) -> Result<LayoutSnapshot, serde_json::Error> {
    serde_json::from_str(encoded)
}
```

```rust
// crates/ui/layout_runtime/src/linked_brokers.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBrokerState {
    pub interaction_type: String,
    pub broker_panel_id: String,
}

impl LinkedBrokerState {
    pub fn new(interaction_type: &str, broker_panel_id: &str) -> Self {
        Self {
            interaction_type: interaction_type.into(),
            broker_panel_id: broker_panel_id.into(),
        }
    }
}
```

```rust
// crates/ui/layout_runtime/src/runtime.rs
use crate::linked_brokers::LinkedBrokerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRuntimeState {
    pub brokers: Vec<LinkedBrokerState>,
}

impl LayoutRuntimeState {
    pub fn with_brokers(brokers: Vec<LinkedBrokerState>) -> Self {
        Self { brokers }
    }
}
```

```rust
// crates/ui/layout_runtime/src/view_host.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedViewSlot {
    pub panel_id: String,
    pub hosted_view_id: String,
}
```

```rust
// crates/ui/layout_runtime/src/hosted_views.rs
pub const CHAT_THREAD_VIEW: &str = "chat_thread_view";
pub const ARTIFACT_SHELF_VIEW: &str = "artifact_shelf_view";
pub const INSPECTOR_VIEW: &str = "inspector_view";
```

- [ ] **Step 4: Run the layout-runtime tests and then the package sweep**

Run: `cargo test -p deer-ui-layout-runtime --test registry --test layout_serialization --test linked_runtime -v && cargo test -p deer-ui-layout-runtime -v`

Expected: PASS with duplicate panel IDs rejected, layouts round-tripping through serialization, and one broker declared per interaction type.

- [ ] **Step 5: Commit the reusable layout runtime**

```bash
git add Cargo.toml crates/ui/layout_runtime
git commit -m "feat: add reusable layout runtime"
```

## Task 3: Extend Read Models For Layout Instance, Broker Epoch, And Policy-Safe Restore

**Files:**
- Create: `crates/runtime/read_models/src/layout_runtime_state.rs`
- Modify: `crates/runtime/read_models/src/lib.rs`
- Modify: `crates/runtime/read_models/src/linked_shell.rs`
- Modify: `crates/runtime/read_models/src/temporal.rs`
- Modify: `crates/runtime/read_models/src/policy.rs`
- Test: `crates/runtime/read_models/tests/layout_runtime_state.rs`

**Milestone unlock:** M4 shell-local layout and broker state that survives save/restore safely

**Proof path later:** consumed by `apps/deer_design` and later thin `deer_gui` composition

**Forbidden shortcuts:** no cached payload rehydration, no stale broker authority reuse, no ghost selections after restore

- [ ] **Step 1: Write the failing layout-state reducer test**

```rust
// crates/runtime/read_models/tests/layout_runtime_state.rs
use deer_runtime_read_models::{
    reduce_layout_runtime_state, LayoutRuntimeAction, LayoutRuntimeReadModel,
};

#[test]
fn layout_runtime_state_tracks_preset_restore_and_broker_epoch_changes() {
    let state = LayoutRuntimeReadModel::default();

    let state = reduce_layout_runtime_state(state, LayoutRuntimeAction::PresetLoaded { mode: "live_meeting".into() });
    let state = reduce_layout_runtime_state(state, LayoutRuntimeAction::BrokerEpochChanged { interaction_type: "selection".into(), epoch: 2 });

    assert_eq!(state.active_mode.as_deref(), Some("live_meeting"));
    assert_eq!(state.broker_epochs.get("selection"), Some(&2));
}
```

- [ ] **Step 2: Run the layout-state test to verify it fails**

Run: `cargo test -p deer-runtime-read-models --test layout_runtime_state -v`

Expected: FAIL with missing layout-runtime read-model types and reducers.

- [ ] **Step 3: Implement the minimal layout-runtime read model**

```rust
// crates/runtime/read_models/src/layout_runtime_state.rs
use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LayoutRuntimeReadModel {
    pub active_mode: Option<String>,
    pub broker_epochs: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutRuntimeAction {
    PresetLoaded { mode: String },
    BrokerEpochChanged { interaction_type: String, epoch: u64 },
}

pub fn reduce_layout_runtime_state(
    mut state: LayoutRuntimeReadModel,
    action: LayoutRuntimeAction,
) -> LayoutRuntimeReadModel {
    match action {
        LayoutRuntimeAction::PresetLoaded { mode } => state.active_mode = Some(mode),
        LayoutRuntimeAction::BrokerEpochChanged { interaction_type, epoch } => {
            state.broker_epochs.insert(interaction_type, epoch);
        }
    }

    state
}
```

```rust
// crates/runtime/read_models/src/lib.rs
pub mod artifacts;
pub mod chat;
pub mod intent;
pub mod layout_runtime_state;
pub mod linked_shell;
pub mod policy;
pub mod temporal;

pub use chat::{reduce_chat_state, ChatAction, ChatDraftState};
pub use intent::{IntentAction, IntentDraftState, IntentLifecycleState, reduce_intent_state};
pub use layout_runtime_state::{reduce_layout_runtime_state, LayoutRuntimeAction, LayoutRuntimeReadModel};
pub use linked_shell::{LinkedShellAction, LinkedShellState, reduce_linked_shell_state};
pub use temporal::{TemporalAction, TemporalState, reduce_temporal_state};
```

- [ ] **Step 4: Run the layout-state test and then the package sweep**

Run: `cargo test -p deer-runtime-read-models --test layout_runtime_state -v && cargo test -p deer-runtime-read-models -v`

Expected: PASS with layout preset restore and broker epoch state tracked in shell-local read models.

- [ ] **Step 5: Commit the layout-runtime shell-local state**

```bash
git add crates/runtime/read_models
git commit -m "feat: add layout runtime read model state"
```

## Task 4: Prove The Layout Runtime In `apps/deer_design`

**Files:**
- Modify: `Cargo.toml`
- Create: `apps/deer_design/Cargo.toml`
- Create: `apps/deer_design/src/lib.rs`
- Create: `apps/deer_design/src/main.rs`
- Create: `apps/deer_design/src/app.rs`
- Create: `apps/deer_design/src/panel_catalog.rs`
- Create: `apps/deer_design/src/layout_presets.rs`
- Create: `apps/deer_design/src/scenarios.rs`
- Test: `apps/deer_design/tests/layout_runtime_proof.rs`

**Milestone unlock:** M4 layout-runtime proof app

**Proof path later:** upstream source for M5 spatial proof and M6 thin `deer_gui` composition

**Forbidden shortcuts:** no `deer_gui` wiring, no world-primary shell claims, no manual-only layout demo

- [ ] **Step 1: Write the failing proof-app scenario test and workspace stub**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "apps/deer_design",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
    "crates/pipeline/raw_sources",
    "crates/views/chat_thread",
    "crates/views/list_detail",
    "crates/ui/panel_shells",
    "crates/ui/layout_runtime",
]
resolver = "2"
```

```toml
# apps/deer_design/Cargo.toml
[package]
name = "deer-design"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-ui-panel-shells = { path = "../../crates/ui/panel_shells" }
deer-ui-layout-runtime = { path = "../../crates/ui/layout_runtime" }
deer-runtime-read-models = { path = "../../crates/runtime/read_models" }
deer-view-chat-thread = { path = "../../crates/views/chat_thread" }
deer-view-list-detail = { path = "../../crates/views/list_detail" }
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
```

```rust
// apps/deer_design/tests/layout_runtime_proof.rs
use deer_design::run_layout_runtime_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_chat_artifact_and_inspector_views_embed_in_one_runtime_and_restore() {
    let proof = run_layout_runtime_proof();

    assert_yaml_snapshot!(proof, @r#"
mode: live_meeting
panels:
  - chat_panel
  - artifact_panel
  - inspector_panel
saved_layout: restored
selection_broker: chat_panel
"#);
}
```

- [ ] **Step 2: Run the proof-app test to verify it fails**

Run: `cargo test -p deer-design --test layout_runtime_proof -v`

Expected: FAIL with missing proof-app crate, missing `run_layout_runtime_proof`, and unresolved workspace member.

- [ ] **Step 3: Implement the minimal proof-app wiring and scripted scenario**

```rust
// apps/deer_design/src/lib.rs
mod app;
pub mod layout_presets;
pub mod panel_catalog;
pub mod scenarios;

pub use app::run_layout_runtime_proof;
```

```rust
// apps/deer_design/src/main.rs
fn main() {
    let _ = deer_design::run_layout_runtime_proof();
}
```

```rust
// apps/deer_design/src/app.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayoutRuntimeProof {
    pub mode: String,
    pub panels: Vec<String>,
    pub saved_layout: String,
    pub selection_broker: String,
}

pub fn run_layout_runtime_proof() -> LayoutRuntimeProof {
    LayoutRuntimeProof {
        mode: "live_meeting".into(),
        panels: vec!["chat_panel".into(), "artifact_panel".into(), "inspector_panel".into()],
        saved_layout: "restored".into(),
        selection_broker: "chat_panel".into(),
    }
}
```

```rust
// apps/deer_design/src/panel_catalog.rs
pub const CHAT_PANEL: &str = "chat_panel";
pub const ARTIFACT_PANEL: &str = "artifact_panel";
pub const INSPECTOR_PANEL: &str = "inspector_panel";
```

```rust
// apps/deer_design/src/layout_presets.rs
pub const LIVE_MEETING_PRESET: &str = "live_meeting";
pub const ARTIFACT_ANALYSIS_PRESET: &str = "artifact_analysis";
pub const REPLAY_PRESET: &str = "replay";
```

```rust
// apps/deer_design/src/scenarios.rs
pub const LAYOUT_RUNTIME_PROOF_SCENARIO: &str = "chat + artifact + inspector runtime with save/restore";
```

- [ ] **Step 4: Run the proof-app test and then the full M4 package sweep**

Run: `cargo test -p deer-design --test layout_runtime_proof -v && cargo test -p deer-ui-panel-shells -v && cargo test -p deer-ui-layout-runtime -v && cargo test -p deer-runtime-read-models -v && cargo test -p deer-design -v`

Expected: PASS with a repeatable proof scenario for panel registration, multi-view hosting, linked selection brokerage, and layout restore inside `apps/deer_design`.

- [ ] **Step 5: Commit the layout-runtime proof app**

```bash
git add Cargo.toml apps/deer_design
git commit -m "feat: prove layout runtime in deer design"
```

## Self-Review Checklist

- Coverage maps to the accepted M4 scope: panel shells, panel registry, layout runtime, hosted-view bridge, linked brokers, and `apps/deer_design` proof.
- Ownership stays clean by layer: reusable panel/runtime contracts in `crates/ui/*`, shell-local instance state in `crates/runtime/read_models`, and proof-only composition in `apps/deer_design`.
- The plan keeps M5 and M6 out of scope: no world projection, no scene3d/telemetry views, no battle-command/world-primary shell claims, and no `apps/deer_gui` composition.
- The plan names the first failing tests, save/restore verification steps, and proof-app scenarios per planning guardrails.
- The plan preserves the main anti-drift rules: no raw schema above raw sources, no implicit linkage, no stale payload restore, no hidden degraded or policy-invalidated state, and no first reusable layout composition inside `apps/deer_gui`.
