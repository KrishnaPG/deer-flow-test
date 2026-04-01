# Battle Command Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the world-primary `battle_command` shell for `apps/deer_gui` with synced tactical viewport/minimap behavior, tiered shell visibility, collapsible rails, and a compound bottom deck.

**Architecture:** Add a dedicated `hud::battle_command` slice driven by typed shell resources and world/camera contracts. Keep render systems thin: world/minimap projection and camera navigation live in `world` and `camera`, while HUD systems only present tiered shell state and issue explicit navigation or command-prefill requests.

**Tech Stack:** Rust 2021, Bevy 0.18 ECS/resources/schedules, `bevy_egui` 0.39, existing camera/world/HUD plugins, single-binary integration tests under `apps/deer_gui/tests/integration.rs`.

---

## File Structure

- Create: `apps/deer_gui/src/hud/battle_command/mod.rs`
- Create: `apps/deer_gui/src/hud/battle_command/contracts.rs`
- Create: `apps/deer_gui/src/hud/battle_command/state.rs`
- Create: `apps/deer_gui/src/hud/battle_command/world_viewport.rs`
- Create: `apps/deer_gui/src/hud/battle_command/minimap.rs`
- Create: `apps/deer_gui/src/hud/battle_command/rails.rs`
- Create: `apps/deer_gui/src/hud/battle_command/inspector.rs`
- Create: `apps/deer_gui/src/hud/battle_command/bottom_deck.rs`
- Create: `apps/deer_gui/src/hud/battle_command/overlays.rs`
- Create: `apps/deer_gui/src/camera/navigation.rs`
- Create: `apps/deer_gui/src/world/viewport.rs`
- Create: `apps/deer_gui/tests/integration/battle_command.rs`
- Modify: `apps/deer_gui/src/hud/mod.rs`
- Modify: `apps/deer_gui/src/hud/plugin.rs`
- Modify: `apps/deer_gui/src/camera/mod.rs`
- Modify: `apps/deer_gui/src/camera/components.rs`
- Modify: `apps/deer_gui/src/camera/plugin.rs`
- Modify: `apps/deer_gui/src/camera/systems.rs`
- Modify: `apps/deer_gui/src/world/mod.rs`
- Modify: `apps/deer_gui/src/world/components.rs`
- Modify: `apps/deer_gui/tests/integration.rs`
- Modify: `apps/deer_gui/tests/integration/hud.rs`
- Modify: `apps/deer_gui/tests/integration/camera.rs`
- Modify: `apps/deer_gui/tests/integration/world_extended.rs`

### Task 1: Add `battle_command` contracts and state

**Files:**
- Create: `apps/deer_gui/src/hud/battle_command/contracts.rs`
- Create: `apps/deer_gui/src/hud/battle_command/state.rs`
- Create: `apps/deer_gui/src/hud/battle_command/mod.rs`
- Modify: `apps/deer_gui/src/hud/mod.rs`
- Create: `apps/deer_gui/tests/integration/battle_command.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Write the failing battle-command state tests**

```rust
use bevy::prelude::*;

use deer_gui::hud::battle_command::{BattleCommandHudState, BottomDeckSection, RailCollapseState, ShellVisibilityTier};

#[test]
fn t_battle_01_shell_contract_defaults() {
    let state = BattleCommandHudState::default();
    assert_eq!(state.visibility_tier, ShellVisibilityTier::Tier1);
    assert_eq!(state.event_rail, RailCollapseState::Expanded);
    assert_eq!(state.fleet_rail, RailCollapseState::Expanded);
    assert_eq!(state.active_bottom_section, BottomDeckSection::SelectionSummary);
}
```

- [ ] **Step 2: Run the battle-command state test and verify it fails**

Run: `cargo test -p deer-gui --test integration integration::battle_command::t_battle_01_shell_contract_defaults -- --exact --nocapture`
Expected: FAIL with missing `battle_command` module.

- [ ] **Step 3: Add the minimal battle-command contract/state types**

```rust
// apps/deer_gui/src/hud/battle_command/state.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellVisibilityTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailCollapseState {
    Expanded,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomDeckSection {
    SelectionSummary,
    ActionDeck,
    QueueStatus,
}

#[derive(Debug, Resource)]
pub struct BattleCommandHudState {
    pub visibility_tier: ShellVisibilityTier,
    pub event_rail: RailCollapseState,
    pub fleet_rail: RailCollapseState,
    pub active_bottom_section: BottomDeckSection,
}

impl Default for BattleCommandHudState {
    fn default() -> Self {
        Self {
            visibility_tier: ShellVisibilityTier::Tier1,
            event_rail: RailCollapseState::Expanded,
            fleet_rail: RailCollapseState::Expanded,
            active_bottom_section: BottomDeckSection::SelectionSummary,
        }
    }
}
```

- [ ] **Step 4: Run the battle-command state test and verify it passes**

Run: `cargo test -p deer-gui --test integration integration::battle_command::t_battle_01_shell_contract_defaults -- --exact --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit the battle-command contract slice**

```bash
git add apps/deer_gui/src/hud/battle_command/contracts.rs apps/deer_gui/src/hud/battle_command/mod.rs apps/deer_gui/src/hud/battle_command/state.rs apps/deer_gui/src/hud/mod.rs apps/deer_gui/tests/integration.rs apps/deer_gui/tests/integration/battle_command.rs
git commit -m "feat: add battle command shell contracts and state"
```

### Task 2: Add world viewport, minimap, and camera sync/navigation

**Files:**
- Create: `apps/deer_gui/src/world/viewport.rs`
- Create: `apps/deer_gui/src/camera/navigation.rs`
- Create: `apps/deer_gui/src/hud/battle_command/world_viewport.rs`
- Create: `apps/deer_gui/src/hud/battle_command/minimap.rs`
- Modify: `apps/deer_gui/src/world/mod.rs`
- Modify: `apps/deer_gui/src/camera/mod.rs`
- Modify: `apps/deer_gui/src/camera/components.rs`
- Modify: `apps/deer_gui/src/camera/plugin.rs`
- Modify: `apps/deer_gui/src/camera/systems.rs`
- Modify: `apps/deer_gui/tests/integration/camera.rs`
- Modify: `apps/deer_gui/tests/integration/world_extended.rs`
- Modify: `apps/deer_gui/tests/integration/battle_command.rs`

- [ ] **Step 1: Write failing camera/minimap tests**

```rust
#[test]
fn t_cam_07_camera_sync_snapshot_tracks_position_zoom_and_frustum() {
    let snapshot = deer_gui::camera::navigation::CameraSyncSnapshot {
        camera_translation: Vec3::new(10.0, 20.0, 30.0),
        zoom: 1.5,
        frustum_center: Vec2::new(0.5, 0.5),
        frustum_size: Vec2::new(0.2, 0.3),
    };

    assert_eq!(snapshot.zoom, 1.5);
    assert_eq!(snapshot.frustum_size, Vec2::new(0.2, 0.3));
}

#[test]
fn t_battle_02_minimap_locate_repositions_camera_only() {
    let request = deer_gui::camera::navigation::ViewportNavigationRequest {
        target_center: Vec2::new(0.8, 0.1),
    };
    assert_eq!(request.target_center, Vec2::new(0.8, 0.1));
}
```

- [ ] **Step 2: Run the camera/minimap tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::camera::t_cam_07_camera_sync_snapshot_tracks_position_zoom_and_frustum -- --exact --nocapture`
Expected: FAIL with missing navigation types.

- [ ] **Step 3: Implement camera sync and viewport navigation types/systems**

```rust
// apps/deer_gui/src/camera/navigation.rs
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSyncSnapshot {
    pub camera_translation: Vec3,
    pub zoom: f32,
    pub frustum_center: Vec2,
    pub frustum_size: Vec2,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ViewportNavigationRequest {
    pub target_center: Vec2,
}
```

```rust
// apps/deer_gui/src/camera/systems.rs
pub fn viewport_navigation_system(
    mut requests: MessageReader<ViewportNavigationRequest>,
    mut query: Query<&mut CinematicCamera>,
) {
    for request in requests.read() {
        for mut cam in &mut query {
            cam.focus_target = Some(Vec3::new(request.target_center.x, 0.0, request.target_center.y));
        }
    }
}
```

- [ ] **Step 4: Run the camera/world/battle tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::camera -- --nocapture && cargo test -p deer-gui --test integration integration::world_extended -- --nocapture && cargo test -p deer-gui --test integration integration::battle_command -- --nocapture`
Expected: PASS for new camera sync and viewport navigation tests.

- [ ] **Step 5: Commit the world/minimap navigation slice**

```bash
git add apps/deer_gui/src/camera/mod.rs apps/deer_gui/src/camera/navigation.rs apps/deer_gui/src/camera/plugin.rs apps/deer_gui/src/camera/systems.rs apps/deer_gui/src/hud/battle_command/minimap.rs apps/deer_gui/src/hud/battle_command/world_viewport.rs apps/deer_gui/src/world/mod.rs apps/deer_gui/src/world/viewport.rs apps/deer_gui/tests/integration/battle_command.rs apps/deer_gui/tests/integration/camera.rs apps/deer_gui/tests/integration/world_extended.rs
git commit -m "feat: sync tactical viewport and minimap navigation"
```

### Task 3: Add tiered shell visibility and collapsible rails

**Files:**
- Create: `apps/deer_gui/src/hud/battle_command/rails.rs`
- Create: `apps/deer_gui/src/hud/battle_command/inspector.rs`
- Create: `apps/deer_gui/src/hud/battle_command/overlays.rs`
- Modify: `apps/deer_gui/src/hud/plugin.rs`
- Modify: `apps/deer_gui/src/hud/left_panel.rs`
- Modify: `apps/deer_gui/src/hud/right_inspector.rs`
- Modify: `apps/deer_gui/src/hud/center_canvas.rs`
- Modify: `apps/deer_gui/tests/integration/hud.rs`
- Modify: `apps/deer_gui/tests/integration/battle_command.rs`

- [ ] **Step 1: Write failing tier and rail tests**

```rust
#[test]
fn t_battle_04_collapsed_event_rail_keeps_badges() {
    let mut state = BattleCommandHudState::default();
    state.event_rail = RailCollapseState::Compact;
    state.event_badge_count = 3;
    assert_eq!(state.event_badge_count, 3);
}

#[test]
fn t_battle_06_tier3_overlay_pauses_world_navigation_but_keeps_context() {
    let mut state = BattleCommandHudState::default();
    state.visibility_tier = ShellVisibilityTier::Tier3;
    state.overlay_blocks_world_navigation = true;
    assert!(state.overlay_blocks_world_navigation);
}
```

- [ ] **Step 2: Run the tier/rail tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::battle_command::t_battle_04_collapsed_event_rail_keeps_badges -- --exact --nocapture`
Expected: FAIL with missing fields or rail state behavior.

- [ ] **Step 3: Implement visibility tiers, compact rails, and legacy panel gating**

```rust
// apps/deer_gui/src/hud/battle_command/state.rs
pub struct BattleCommandHudState {
    pub visibility_tier: ShellVisibilityTier,
    pub event_rail: RailCollapseState,
    pub fleet_rail: RailCollapseState,
    pub event_badge_count: u32,
    pub fleet_badge_count: u32,
    pub overlay_blocks_world_navigation: bool,
    pub active_bottom_section: BottomDeckSection,
}
```

```rust
// apps/deer_gui/src/hud/plugin.rs
// register battle-command update systems before EguiPrimaryContextPass and
// short-circuit legacy left/right/center renderers when battle mode is active.
```

- [ ] **Step 4: Run the HUD and battle-command tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::hud -- --nocapture && cargo test -p deer-gui --test integration integration::battle_command -- --nocapture`
Expected: PASS for battle visibility, rail compaction, and legacy panel coexistence tests.

- [ ] **Step 5: Commit the visibility-tier and rail slice**

```bash
git add apps/deer_gui/src/hud/battle_command/inspector.rs apps/deer_gui/src/hud/battle_command/overlays.rs apps/deer_gui/src/hud/battle_command/rails.rs apps/deer_gui/src/hud/battle_command/state.rs apps/deer_gui/src/hud/center_canvas.rs apps/deer_gui/src/hud/left_panel.rs apps/deer_gui/src/hud/plugin.rs apps/deer_gui/src/hud/right_inspector.rs apps/deer_gui/tests/integration/battle_command.rs apps/deer_gui/tests/integration/hud.rs
git commit -m "feat: add battle command visibility tiers and collapsible rails"
```

### Task 4: Add the compound bottom deck and end-to-end shell flow tests

**Files:**
- Create: `apps/deer_gui/src/hud/battle_command/bottom_deck.rs`
- Modify: `apps/deer_gui/src/hud/bottom_console.rs`
- Modify: `apps/deer_gui/src/hud/plugin.rs`
- Modify: `apps/deer_gui/tests/integration/hud.rs`
- Modify: `apps/deer_gui/tests/integration/battle_command.rs`

- [ ] **Step 1: Write failing bottom-deck tests**

```rust
#[test]
fn t_battle_07_bottom_deck_sections_share_context_but_keep_distinct_state() {
    let state = BattleCommandHudState::default();
    assert_eq!(state.active_bottom_section, BottomDeckSection::SelectionSummary);
}

#[test]
fn t_battle_08_action_deck_hotkey_slots_stay_stable_across_selection_updates() {
    let slots = deer_gui::hud::battle_command::bottom_deck::default_action_slots();
    assert_eq!(slots.len(), 15);
    assert_eq!(slots[14].label, "Cancel");
}
```

- [ ] **Step 2: Run the bottom-deck tests and verify they fail**

Run: `cargo test -p deer-gui --test integration integration::battle_command::t_battle_08_action_deck_hotkey_slots_stay_stable_across_selection_updates -- --exact --nocapture`
Expected: FAIL with missing bottom-deck helpers.

- [ ] **Step 3: Implement the bottom deck and stable action slots**

```rust
// apps/deer_gui/src/hud/battle_command/bottom_deck.rs
pub struct ActionSlot {
    pub label: &'static str,
    pub hotkey: &'static str,
}

pub fn default_action_slots() -> [ActionSlot; 15] {
    [
        ActionSlot { label: "Move", hotkey: "Q" },
        ActionSlot { label: "Hold", hotkey: "W" },
        ActionSlot { label: "Patrol", hotkey: "E" },
        ActionSlot { label: "Focus", hotkey: "R" },
        ActionSlot { label: "Scan", hotkey: "A" },
        ActionSlot { label: "Dock", hotkey: "S" },
        ActionSlot { label: "Escort", hotkey: "D" },
        ActionSlot { label: "Split", hotkey: "F" },
        ActionSlot { label: "Merge", hotkey: "Z" },
        ActionSlot { label: "Queue", hotkey: "X" },
        ActionSlot { label: "Intel", hotkey: "C" },
        ActionSlot { label: "Ping", hotkey: "V" },
        ActionSlot { label: "Repair", hotkey: "1" },
        ActionSlot { label: "Upgrade", hotkey: "2" },
        ActionSlot { label: "Cancel", hotkey: "Esc" },
    ]
}
```

- [ ] **Step 4: Run the bottom-deck and full battle-command tests and verify they pass**

Run: `cargo test -p deer-gui --test integration integration::battle_command -- --nocapture && cargo test -p deer-gui --test integration integration::hud -- --nocapture`
Expected: PASS for bottom-deck, stable hotkey slot, and selection-to-inspection shell flow tests.

- [ ] **Step 5: Commit the compound bottom-deck slice**

```bash
git add apps/deer_gui/src/hud/battle_command/bottom_deck.rs apps/deer_gui/src/hud/bottom_console.rs apps/deer_gui/src/hud/plugin.rs apps/deer_gui/tests/integration/battle_command.rs apps/deer_gui/tests/integration/hud.rs
git commit -m "feat: add battle command compound bottom deck"
```
