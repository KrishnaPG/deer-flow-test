# Shell State And Linked Interaction Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the typed shell-state and linked-interaction foundation for `apps/deer_gui` so selection, focus, filters, pins, compare sets, and intent prefill are brokered outside the HUD and can drive all later shell modes.

**Architecture:** Add a dedicated `shell` module that owns shell-local, non-canonical interaction state and broker systems. Picking and world systems publish typed canonical references into shell brokers; HUD panels become thin render/input adapters that read synchronized shell state instead of owning cross-panel truth directly.

**Tech Stack:** Rust 2021, Bevy 0.18 resources/messages/schedules, `bevy_egui` 0.39 HUD rendering, existing single-binary integration tests under `apps/deer_gui/tests/integration.rs`.

---

## File Structure

- Create: `apps/deer_gui/src/shell/mod.rs` - module exports
- Create: `apps/deer_gui/src/shell/plugin.rs` - plugin and schedule registration
- Create: `apps/deer_gui/src/shell/state.rs` - shell resources, canonical refs, interaction slices
- Create: `apps/deer_gui/src/shell/panel_participation.rs` - panel IDs, roles, interaction kinds, defaults
- Create: `apps/deer_gui/src/shell/systems.rs` - broker systems and shell mutation flow
- Create: `apps/deer_gui/src/hud/shell_sync.rs` - shell-to-HUD synchronization systems
- Create: `apps/deer_gui/tests/integration/shell_state.rs` - foundation integration coverage
- Modify: `apps/deer_gui/src/lib.rs` - export `shell`
- Modify: `apps/deer_gui/src/main.rs` - register `ShellPlugin`
- Modify: `apps/deer_gui/src/hud/mod.rs` - expose shell sync module and reduce authority of `HudState`
- Modify: `apps/deer_gui/src/hud/plugin.rs` - schedule shell sync before render systems
- Modify: `apps/deer_gui/src/hud/state_systems.rs` - keep HUD mutation local to HUD-only concerns
- Modify: `apps/deer_gui/src/hud/right_inspector.rs` - consume shell-driven selection detail
- Modify: `apps/deer_gui/src/hud/bottom_console.rs` - consume shell-driven intent prefill
- Modify: `apps/deer_gui/src/hud/center_canvas.rs` - read shell mode/focus hooks
- Modify: `apps/deer_gui/src/world/components.rs` - canonical ref conversion from world entities
- Modify: `apps/deer_gui/src/picking/plugin.rs` - register shell-facing selection systems
- Modify: `apps/deer_gui/src/picking/systems.rs` - publish brokered selection/focus state
- Modify: `apps/deer_gui/tests/integration.rs` - add `shell_state` module
- Modify: `apps/deer_gui/tests/integration/hud.rs` - add HUD sync coverage
- Modify: `apps/deer_gui/tests/integration/picking.rs` - add picking-to-shell coverage
- Modify: `apps/deer_gui/tests/integration/world.rs` - add world-to-canonical-ref coverage

### Task 1: Add typed shell-state contracts

**Files:**
- Create: `apps/deer_gui/src/shell/mod.rs`
- Create: `apps/deer_gui/src/shell/state.rs`
- Modify: `apps/deer_gui/src/lib.rs`
- Create: `apps/deer_gui/tests/integration/shell_state.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Write the failing shell-state tests**

```rust
use bevy::prelude::*;

use deer_gui::shell::{CanonicalEntityRef, CanonicalRecordFamily, ShellPlugin, ShellState};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ShellPlugin);
    app
}

#[test]
fn t_shell_01_shell_state_defaults() {
    let mut app = test_app();
    app.update();

    let shell = app.world().resource::<ShellState>();
    assert!(shell.selection.primary.is_none());
    assert!(shell.selection.ordered.is_empty());
    assert!(shell.focus.target.is_none());
    assert!(shell.filters.global.is_empty());
    assert!(shell.pins.items.is_empty());
    assert!(shell.compare.items.is_empty());
    assert!(shell.intent_prefill.target.is_none());
}

#[test]
fn t_shell_02_shell_state_tracks_typed_selection_focus_filter_pin_compare_prefill() {
    let mut app = test_app();
    app.update();

    let reference = CanonicalEntityRef {
        family: CanonicalRecordFamily::Mission,
        canonical_id: "mission-1".to_string(),
        correlation_id: Some("corr-1".to_string()),
        lineage_id: None,
    };

    let mut shell = app.world_mut().resource_mut::<ShellState>();
    shell.selection.primary = Some(reference.clone());
    shell.selection.ordered.push(reference.clone());
    shell.focus.target = Some(reference.clone());
    shell.pins.items.push(reference.clone());
    shell.compare.items.push(reference.clone());
    shell.intent_prefill.target = Some(reference.clone());

    assert_eq!(shell.selection.ordered.len(), 1);
    assert_eq!(shell.pins.items.len(), 1);
    assert_eq!(shell.compare.items.len(), 1);
    assert_eq!(shell.intent_prefill.target.as_ref().unwrap().canonical_id, "mission-1");
}
```

- [ ] **Step 2: Run the shell-state tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::shell_state::t_shell_01_shell_state_defaults -- --exact --nocapture`
Expected: FAIL with unresolved import or missing `shell` module/types.

- [ ] **Step 3: Add the minimal shell-state implementation**

```rust
// apps/deer_gui/src/shell/state.rs
use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalRecordFamily {
    Agent,
    Mission,
    Artifact,
    Session,
    ReplayCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEntityRef {
    pub family: CanonicalRecordFamily,
    pub canonical_id: String,
    pub correlation_id: Option<String>,
    pub lineage_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct SelectionState {
    pub primary: Option<CanonicalEntityRef>,
    pub ordered: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct FocusState {
    pub target: Option<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct FilterState {
    pub global: Vec<String>,
    pub panel: Vec<String>,
}

#[derive(Debug, Default)]
pub struct PinState {
    pub items: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct CompareState {
    pub items: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct IntentPrefillState {
    pub target: Option<CanonicalEntityRef>,
}

#[derive(Resource, Debug, Default)]
pub struct ShellState {
    pub selection: SelectionState,
    pub focus: FocusState,
    pub filters: FilterState,
    pub pins: PinState,
    pub compare: CompareState,
    pub intent_prefill: IntentPrefillState,
}
```

```rust
// apps/deer_gui/src/shell/plugin.rs
use bevy::prelude::*;

use super::state::ShellState;

pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShellState>();
    }
}
```

```rust
// apps/deer_gui/src/shell/mod.rs
mod plugin;
mod state;

pub use plugin::ShellPlugin;
pub use state::{CanonicalEntityRef, CanonicalRecordFamily, ShellState};
```

- [ ] **Step 4: Run the shell-state tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::shell_state -- --nocapture`
Expected: PASS for `t_shell_01_shell_state_defaults` and `t_shell_02_shell_state_tracks_typed_selection_focus_filter_pin_compare_prefill`.

- [ ] **Step 5: Commit the shell-state contract slice**

```bash
git add apps/deer_gui/src/lib.rs apps/deer_gui/src/shell/mod.rs apps/deer_gui/src/shell/plugin.rs apps/deer_gui/src/shell/state.rs apps/deer_gui/tests/integration.rs apps/deer_gui/tests/integration/shell_state.rs
git commit -m "feat: add typed shell state resources"
```

### Task 2: Add broker events and panel participation defaults

**Files:**
- Create: `apps/deer_gui/src/shell/panel_participation.rs`
- Create: `apps/deer_gui/src/shell/systems.rs`
- Modify: `apps/deer_gui/src/shell/mod.rs`
- Modify: `apps/deer_gui/src/shell/plugin.rs`
- Modify: `apps/deer_gui/src/main.rs`
- Test: `apps/deer_gui/tests/integration/shell_state.rs`

- [ ] **Step 1: Write failing broker and participation tests**

```rust
use deer_gui::shell::{InteractionKind, PanelId, PanelParticipationRegistry, PanelRole, ShellSelectionRequest};

#[test]
fn t_shell_03_panel_participation_defaults_match_control_center_scaffolding() {
    let registry = PanelParticipationRegistry::default();
    let inspector = registry.get(&PanelId::InspectorPanel).unwrap();
    assert!(inspector.roles.contains(&PanelRole::Sink));
    assert!(inspector.interactions.contains(&InteractionKind::Selection));
}

#[test]
fn t_shell_04_brokers_apply_requests_and_update_sequences() {
    let mut app = test_app();
    app.world_mut().write_message(ShellSelectionRequest {
        next: CanonicalEntityRef {
            family: CanonicalRecordFamily::Mission,
            canonical_id: "mission-9".to_string(),
            correlation_id: None,
            lineage_id: None,
        },
        source: PanelId::MissionRailPanel,
    });

    app.update();

    let shell = app.world().resource::<ShellState>();
    assert_eq!(shell.selection.primary.as_ref().unwrap().canonical_id, "mission-9");
}
```

- [ ] **Step 2: Run the broker tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::shell_state::t_shell_03_panel_participation_defaults_match_control_center_scaffolding -- --exact --nocapture`
Expected: FAIL with missing broker types or message registration.

- [ ] **Step 3: Implement broker messages, participation registry, and broker systems**

```rust
// apps/deer_gui/src/shell/panel_participation.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    MissionRailPanel,
    InspectorPanel,
    CommandDeckPanel,
    CenterCanvasPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionKind {
    Selection,
    Focus,
    Filter,
    Pin,
    Compare,
    IntentPrefill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelRole {
    Source,
    Sink,
    Broker,
}

#[derive(Debug, Clone)]
pub struct PanelParticipation {
    pub roles: Vec<PanelRole>,
    pub interactions: Vec<InteractionKind>,
}

#[derive(Debug, Default)]
pub struct PanelParticipationRegistry {
    panels: HashMap<PanelId, PanelParticipation>,
}
```

```rust
// apps/deer_gui/src/shell/systems.rs
use bevy::prelude::*;

use super::panel_participation::PanelId;
use super::state::{CanonicalEntityRef, ShellState};

#[derive(Message, Debug, Clone)]
pub struct ShellSelectionRequest {
    pub next: CanonicalEntityRef,
    pub source: PanelId,
}

pub fn selection_broker_system(
    mut requests: MessageReader<ShellSelectionRequest>,
    mut shell: ResMut<ShellState>,
) {
    for request in requests.read() {
        shell.selection.primary = Some(request.next.clone());
        shell.selection.ordered.clear();
        shell.selection.ordered.push(request.next.clone());
    }
}
```

```rust
// apps/deer_gui/src/shell/plugin.rs
app.init_resource::<ShellState>()
    .add_message::<ShellSelectionRequest>()
    .add_systems(Update, selection_broker_system);
```

- [ ] **Step 4: Run the broker tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::shell_state -- --nocapture`
Expected: PASS for participation defaults and selection broker update flow.

- [ ] **Step 5: Commit the broker scaffolding**

```bash
git add apps/deer_gui/src/main.rs apps/deer_gui/src/shell/mod.rs apps/deer_gui/src/shell/panel_participation.rs apps/deer_gui/src/shell/plugin.rs apps/deer_gui/src/shell/systems.rs apps/deer_gui/tests/integration/shell_state.rs
git commit -m "feat: add shell brokers and panel participation scaffolding"
```

### Task 3: Route world entities and picking through canonical shell selection

**Files:**
- Modify: `apps/deer_gui/src/world/components.rs`
- Modify: `apps/deer_gui/src/picking/plugin.rs`
- Modify: `apps/deer_gui/src/picking/systems.rs`
- Modify: `apps/deer_gui/tests/integration/world.rs`
- Modify: `apps/deer_gui/tests/integration/picking.rs`

- [ ] **Step 1: Write failing world and picking integration tests**

```rust
#[test]
fn t_world_08_world_entity_maps_to_canonical_ref() {
    let world_entity = WorldEntity {
        entity_id: "mission-77".to_string(),
        entity_type: WorldEntityType::Mission { progress: 0.8 },
    };

    let canonical = world_entity.to_canonical_ref();
    assert_eq!(canonical.family, CanonicalRecordFamily::Mission);
    assert_eq!(canonical.canonical_id, "mission-77");
}

#[test]
fn t_pick_11_selection_updates_shell_selection_state() {
    let mut app = test_app();
    app.add_plugins(ShellPlugin);
    app.world_mut().write_message(EntityClicked(app.world_mut().spawn((
        Selectable,
        WorldEntity {
            entity_id: "agent-7".to_string(),
            entity_type: WorldEntityType::Agent(AgentState::Working),
        },
    )).id()));

    app.update();

    let shell = app.world().resource::<ShellState>();
    assert_eq!(shell.selection.primary.as_ref().unwrap().canonical_id, "agent-7");
}
```

- [ ] **Step 2: Run the world and picking tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::world::t_world_08_world_entity_maps_to_canonical_ref -- --exact --nocapture`
Expected: FAIL with missing `to_canonical_ref`.

- [ ] **Step 3: Implement canonical ref conversion and selection publication**

```rust
// apps/deer_gui/src/world/components.rs
impl WorldEntity {
    pub fn to_canonical_ref(&self) -> CanonicalEntityRef {
        let family = match self.entity_type {
            WorldEntityType::Agent(_) => CanonicalRecordFamily::Agent,
            WorldEntityType::Mission { .. } => CanonicalRecordFamily::Mission,
            WorldEntityType::Artifact { .. } => CanonicalRecordFamily::Artifact,
            WorldEntityType::Swarm { .. } => CanonicalRecordFamily::Mission,
        };

        CanonicalEntityRef {
            family,
            canonical_id: self.entity_id.clone(),
            correlation_id: None,
            lineage_id: None,
        }
    }
}
```

```rust
// apps/deer_gui/src/picking/systems.rs
pub fn selection_sync_system(
    mut shell_requests: MessageWriter<ShellSelectionRequest>,
    selected_query: Query<&WorldEntity, With<Selected>>,
) {
    if let Some(world_entity) = selected_query.iter().next() {
        shell_requests.write(ShellSelectionRequest {
            next: world_entity.to_canonical_ref(),
            source: PanelId::MissionRailPanel,
        });
    }
}
```

- [ ] **Step 4: Run the world and picking tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::world -- --nocapture && cargo test -p deer-gui --test integration integration::picking -- --nocapture`
Expected: PASS for the new world canonical-ref and picking-to-shell tests.

- [ ] **Step 5: Commit the world/picking routing slice**

```bash
git add apps/deer_gui/src/world/components.rs apps/deer_gui/src/picking/plugin.rs apps/deer_gui/src/picking/systems.rs apps/deer_gui/tests/integration/picking.rs apps/deer_gui/tests/integration/world.rs
git commit -m "refactor: route picking through brokered shell selection"
```

### Task 4: Mirror shell state into HUD panels and add regression coverage

**Files:**
- Create: `apps/deer_gui/src/hud/shell_sync.rs`
- Modify: `apps/deer_gui/src/hud/mod.rs`
- Modify: `apps/deer_gui/src/hud/plugin.rs`
- Modify: `apps/deer_gui/src/hud/right_inspector.rs`
- Modify: `apps/deer_gui/src/hud/bottom_console.rs`
- Modify: `apps/deer_gui/src/hud/center_canvas.rs`
- Modify: `apps/deer_gui/tests/integration/hud.rs`
- Modify: `apps/deer_gui/tests/integration/shell_state.rs`

- [ ] **Step 1: Write failing HUD sync tests**

```rust
#[test]
fn t_hud_07_hud_mirrors_shell_selection_into_inspector() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ShellPlugin);
    app.init_resource::<HudState>();
    app.add_systems(Update, deer_gui::hud::shell_sync::shell_selection_to_hud_system);

    {
        let mut shell = app.world_mut().resource_mut::<ShellState>();
        shell.selection.primary = Some(CanonicalEntityRef {
            family: CanonicalRecordFamily::Agent,
            canonical_id: "agent-99".to_string(),
            correlation_id: None,
            lineage_id: None,
        });
    }

    app.update();
    let hud = app.world().resource::<HudState>();
    assert_eq!(hud.selected_entity.as_ref().unwrap().entity_id, "agent-99");
}

#[test]
fn t_hud_08_command_deck_reads_intent_prefill_context() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ShellPlugin);
    app.init_resource::<HudState>();
    app.add_systems(Update, deer_gui::hud::shell_sync::shell_prefill_to_hud_system);

    app.world_mut().resource_mut::<ShellState>().intent_prefill.target = Some(CanonicalEntityRef {
        family: CanonicalRecordFamily::Mission,
        canonical_id: "mission-42".to_string(),
        correlation_id: None,
        lineage_id: None,
    });

    app.update();
    let hud = app.world().resource::<HudState>();
    assert!(hud.command_input.contains("mission-42"));
}
```

- [ ] **Step 2: Run the HUD sync tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::hud::t_hud_07_hud_mirrors_shell_selection_into_inspector -- --exact --nocapture`
Expected: FAIL with missing `shell_sync` systems.

- [ ] **Step 3: Implement shell-to-HUD sync systems**

```rust
// apps/deer_gui/src/hud/shell_sync.rs
use bevy::prelude::*;

use crate::shell::ShellState;

use super::{HudState, InspectorDetails, EntityInspectorData};

pub fn shell_selection_to_hud_system(mut hud: ResMut<HudState>, shell: Res<ShellState>) {
    hud.selected_entity = shell.selection.primary.as_ref().map(|selection| EntityInspectorData {
        entity_id: selection.canonical_id.clone(),
        display_name: selection.canonical_id.clone(),
        details: InspectorDetails::Mission {
            progress: 0.0,
            assigned_agents: 0,
            description: String::new(),
        },
    });
}

pub fn shell_prefill_to_hud_system(mut hud: ResMut<HudState>, shell: Res<ShellState>) {
    if let Some(target) = &shell.intent_prefill.target {
        hud.command_input = format!("target:{}", target.canonical_id);
    }
}
```

```rust
// apps/deer_gui/src/hud/plugin.rs
.add_systems(
    Update,
    (
        event_ticker_maintenance_system,
        command_dispatch_system,
        shell_selection_to_hud_system,
        shell_prefill_to_hud_system,
    ),
)
```

- [ ] **Step 4: Run the full foundation tests and verify they pass**

Run: `cargo test -p deer-gui --test integration -- --nocapture`
Expected: PASS for new shell-state, HUD, picking, and world integration coverage.

- [ ] **Step 5: Commit the HUD sync foundation**

```bash
git add apps/deer_gui/src/hud/bottom_console.rs apps/deer_gui/src/hud/center_canvas.rs apps/deer_gui/src/hud/mod.rs apps/deer_gui/src/hud/plugin.rs apps/deer_gui/src/hud/right_inspector.rs apps/deer_gui/src/hud/shell_sync.rs apps/deer_gui/tests/integration/hud.rs apps/deer_gui/tests/integration/shell_state.rs
git commit -m "feat: hook hud panels into brokered shell state"
```
