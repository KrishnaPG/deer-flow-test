# PR I — HUD Transclusion (Stateless Data-Only Fragments)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `HudFragmentRegistry` resource and `HudFragment` type so that any subsystem (scenes, swarm, etc.) can push content fragments into the Center Canvas without importing HUD internals. Integrate the registry into the existing center canvas render path as additional tabs alongside built-in modes.

**Architecture:** `HudFragmentRegistry` is a Bevy `Resource` holding a `Vec<HudFragment>`. Each fragment carries a provider ID, display title, and an `Arc<dyn Fn(&mut egui::Ui) + Send + Sync>` render callback. The `center_canvas_system` reads the registry and renders active fragments as additional tabs — purely additive, existing modes untouched.

**Tech Stack:** Rust, Bevy 0.18.1, bevy_egui

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `src/hud/transclusion.rs` | `HudFragment` struct + `HudFragmentRegistry` resource |
| Modify | `src/hud/mod.rs` | Add `pub mod transclusion` + re-export |
| Modify | `src/hud/plugin.rs` | Register `HudFragmentRegistry` resource |
| Modify | `src/hud/center_canvas.rs` | Read registry, render fragments as additional tabs |
| Create | `tests/integration/hud_transclusion.rs` | 4 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod hud_transclusion` |

All paths relative to `apps/deer_gui/`.

---

### Task 1: Create `transclusion.rs` module

**Files:**
- Create: `apps/deer_gui/src/hud/transclusion.rs`

- [ ] **Step 1: Write the module**

```rust
//! HUD transclusion — stateless data-only fragments for the Center Canvas.
//!
//! Any subsystem can push a [`HudFragment`] into the [`HudFragmentRegistry`]
//! resource. The `center_canvas_system` reads the registry each frame and
//! renders registered fragments as additional tabs alongside built-in modes.
//!
//! Fragments are stateless: they carry a render callback but never own state.
//! The registry is the only coupling point between producers and the HUD.

use std::sync::Arc;

use bevy::log::{debug, info, trace};
use bevy::prelude::*;
use bevy_egui::egui;

// ---------------------------------------------------------------------------
// HudFragment
// ---------------------------------------------------------------------------

/// A single content fragment pushed by any subsystem into the Center Canvas.
///
/// Fragments are stateless data definitions. The HUD reads and renders them;
/// fragments never own mutable state.
#[derive(Clone)]
pub struct HudFragment {
    /// Unique provider ID (e.g. "scene.tet", "swarm.monitor").
    pub provider: &'static str,
    /// Display title shown in the canvas tab.
    pub title: String,
    /// Render callback, called each egui frame when this fragment's tab is active.
    pub render: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>,
}

impl std::fmt::Debug for HudFragment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HudFragment")
            .field("provider", &self.provider)
            .field("title", &self.title)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// HudFragmentRegistry
// ---------------------------------------------------------------------------

/// Resource holding all currently registered fragments.
///
/// Subsystems call [`register`](Self::register) to push fragments and
/// [`unregister`](Self::unregister) to remove them. The `center_canvas_system`
/// reads [`fragments`](Self::fragments) each frame.
#[derive(Resource, Default)]
pub struct HudFragmentRegistry {
    fragments: Vec<HudFragment>,
}

impl HudFragmentRegistry {
    /// Register a content fragment.
    ///
    /// If a fragment with the same `provider` already exists, it is replaced.
    pub fn register(&mut self, fragment: HudFragment) {
        info!(
            "HudFragmentRegistry::register — provider='{}' title='{}'",
            fragment.provider, fragment.title
        );
        // Replace existing fragment from same provider (idempotent).
        self.fragments.retain(|f| f.provider != fragment.provider);
        self.fragments.push(fragment);
    }

    /// Remove the fragment registered by the given provider.
    pub fn unregister(&mut self, provider: &'static str) {
        let before = self.fragments.len();
        self.fragments.retain(|f| f.provider != provider);
        debug!(
            "HudFragmentRegistry::unregister — provider='{}' removed={}",
            provider,
            before - self.fragments.len()
        );
    }

    /// Read-only access to all registered fragments.
    pub fn fragments(&self) -> &[HudFragment] {
        trace!(
            "HudFragmentRegistry::fragments — count={}",
            self.fragments.len()
        );
        &self.fragments
    }
}
```

---

### Task 2: Wire module into `hud/mod.rs`

**Files:**
- Modify: `apps/deer_gui/src/hud/mod.rs`

- [ ] **Step 1: Add `pub mod transclusion` and re-export**

After the existing `mod top_bar;` line (line 17), add:

```rust
pub mod transclusion;
```

In the `pub use` block (after line 24), add:

```rust
pub use transclusion::{HudFragment, HudFragmentRegistry};
```

The resulting module declaration block (lines 7–18) becomes:

```rust
pub mod adapters;
mod bottom_console;
mod center_canvas;
mod event_ticker;
mod left_panel;
mod modal;
mod plugin;
mod right_inspector;
pub mod state_systems;
mod styles;
mod top_bar;
pub mod transclusion;
```

And the re-export block (lines 20–25) becomes:

```rust
pub use plugin::HudPlugin;
pub use styles::{
    agent_state_color, draw_progress_bar, glass_panel_frame, mission_status_color,
    side_panel_frame, top_bar_frame,
};
pub use transclusion::{HudFragment, HudFragmentRegistry};
```

---

### Task 3: Register `HudFragmentRegistry` in `HudPlugin`

**Files:**
- Modify: `apps/deer_gui/src/hud/plugin.rs`

- [ ] **Step 1: Add import**

After the existing `use super::HudState;` import (line 19), add:

```rust
use super::transclusion::HudFragmentRegistry;
```

- [ ] **Step 2: Init the resource in `build`**

In `HudPlugin::build`, after `app.init_resource::<HudState>()` (line 49), add:

```rust
            .init_resource::<HudFragmentRegistry>()
```

The resulting build method becomes:

```rust
    fn build(&self, app: &mut App) {
        info!("HudPlugin::build — registering HUD resources and systems");

        app.init_resource::<HudState>()
            .init_resource::<HudFragmentRegistry>()
            // State-maintenance systems run in Update, before EguiPrimaryContextPass.
            .add_systems(
                Update,
                (event_ticker_maintenance_system, command_dispatch_system),
            )
            // Render systems run in EguiPrimaryContextPass, reading HudState.
            .add_systems(
                EguiPrimaryContextPass,
                (
                    top_bar_system,
                    bottom_console_system,
                    left_panel_system,
                    right_inspector_system,
                    center_canvas_system,
                    event_ticker_system,
                    modal_system,
                )
                    .chain(),
            );
    }
```

---

### Task 4: Integrate fragment tabs into center canvas

**Files:**
- Modify: `apps/deer_gui/src/hud/center_canvas.rs`

- [ ] **Step 1: Add import for `HudFragmentRegistry`**

After the existing `use super::{CenterCanvasMode, HudState};` import (line 15), add:

```rust
use super::transclusion::HudFragmentRegistry;
```

- [ ] **Step 2: Update `center_canvas_system` signature to read the registry**

Change the system signature from:

```rust
pub fn center_canvas_system(mut contexts: EguiContexts, hud: Res<HudState>) {
```

to:

```rust
pub fn center_canvas_system(
    mut contexts: EguiContexts,
    hud: Res<HudState>,
    fragment_registry: Res<HudFragmentRegistry>,
) {
```

- [ ] **Step 3: Render fragment tabs after built-in content**

After the `match hud.center_mode { ... }` block (after line 51, inside the `CentralPanel` closure), add fragment rendering:

```rust
        // -- Transcluded fragments --
        if !fragment_registry.fragments().is_empty() {
            ui.separator();
            render_fragment_tabs(ui, &fragment_registry);
        }
```

- [ ] **Step 4: Add the `render_fragment_tabs` helper function**

At the end of the file, before the closing, add:

```rust
/// Renders tabs for all transcluded HUD fragments from the registry.
fn render_fragment_tabs(ui: &mut egui::Ui, registry: &HudFragmentRegistry) {
    for fragment in registry.fragments() {
        trace!("center_canvas — rendering fragment '{}'", fragment.provider);
        ui.collapsing(&fragment.title, |ui| {
            (fragment.render)(ui);
        });
    }
}
```

The complete modified file becomes:

```rust
//! Center canvas HUD panel — switchable content area.
//!
//! Renders one of several views in the central area:
//! - WorldView (passthrough — no panel drawn)
//! - ExpertsMeeting (multi-agent conversation log)
//! - SwarmMonitor (aggregated particle/hex grid)
//! - ArtifactGraph (artifact lineage DAG)
//! - Forensics (debug/trace view)
//!
//! Transcluded fragments from [`HudFragmentRegistry`] are rendered as
//! additional collapsible sections below the active built-in mode.

use bevy::log::{debug, trace};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::styles::glass_panel_frame;
use super::transclusion::HudFragmentRegistry;
use super::{CenterCanvasMode, HudState};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the center canvas with mode-specific content.
///
/// In `WorldView` mode, no panel is drawn — the 3D world shows through.
/// Transcluded fragments from the registry are rendered as additional
/// collapsible sections when the canvas is visible.
pub fn center_canvas_system(
    mut contexts: EguiContexts,
    hud: Res<HudState>,
    fragment_registry: Res<HudFragmentRegistry>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // WorldView = passthrough, nothing to render
    if hud.center_mode == CenterCanvasMode::WorldView {
        trace!("center_canvas_system — WorldView mode, skipping");
        return;
    }

    trace!("center_canvas_system — rendering {:?}", hud.center_mode);

    let frame = glass_panel_frame(ctx);

    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        // Mode selector tabs
        render_mode_tabs(ui, &hud);
        ui.separator();

        // Mode-specific content
        match hud.center_mode {
            CenterCanvasMode::WorldView => unreachable!(), // handled above
            CenterCanvasMode::ExpertsMeeting => render_experts_meeting(ui, &hud),
            CenterCanvasMode::SwarmMonitor => render_swarm_monitor(ui),
            CenterCanvasMode::ArtifactGraph => render_artifact_graph(ui),
            CenterCanvasMode::Forensics => render_forensics(ui),
        }

        // -- Transcluded fragments --
        if !fragment_registry.fragments().is_empty() {
            ui.separator();
            render_fragment_tabs(ui, &fragment_registry);
        }
    });
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Renders the mode selector tabs at the top of the center canvas.
fn render_mode_tabs(ui: &mut egui::Ui, hud: &HudState) {
    ui.horizontal(|ui| {
        let modes = [
            (CenterCanvasMode::WorldView, "World"),
            (CenterCanvasMode::ExpertsMeeting, "Experts"),
            (CenterCanvasMode::SwarmMonitor, "Swarm"),
            (CenterCanvasMode::ArtifactGraph, "Artifacts"),
            (CenterCanvasMode::Forensics, "Forensics"),
        ];
        for (mode, label) in &modes {
            let is_active = hud.center_mode == *mode;
            let text = if is_active {
                egui::RichText::new(*label).strong()
            } else {
                egui::RichText::new(*label).color(egui::Color32::from_rgb(140, 145, 150))
            };
            if ui.selectable_label(is_active, text).clicked() {
                debug!("center_canvas — tab clicked: {:?}", mode);
                // Mode switch is deferred — the HudState is read-only here.
                // In a real implementation, this would send an event.
            }
        }
    });
}

/// Renders the Experts Meeting view — multi-agent conversation log.
fn render_experts_meeting(ui: &mut egui::Ui, hud: &HudState) {
    ui.heading("Experts Meeting");

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if let Some(thread) = &hud.thread_cache {
                for msg in &thread.messages {
                    render_chat_message(ui, &msg.role, &msg.content);
                }
            } else {
                ui.label(
                    egui::RichText::new("No thread selected")
                        .color(egui::Color32::from_rgb(128, 128, 128))
                        .italics(),
                );
            }
        });
}

/// Renders a single chat message bubble.
fn render_chat_message(ui: &mut egui::Ui, role: &str, content: &str) {
    let color = match role {
        "user" => egui::Color32::from_rgb(0, 204, 255),
        "assistant" => egui::Color32::from_rgb(51, 230, 102),
        _ => egui::Color32::from_rgb(180, 190, 200),
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(format!("[{role}]"))
                .color(color)
                .strong(),
        );
        ui.label(content);
    });
    ui.add_space(4.0);
}

/// Renders the Swarm Monitor view — placeholder for particle/hex grid.
fn render_swarm_monitor(ui: &mut egui::Ui) {
    ui.heading("Swarm Monitor");
    ui.label(
        egui::RichText::new("Aggregated swarm view — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

/// Renders the Artifact Graph view — placeholder for DAG visualization.
fn render_artifact_graph(ui: &mut egui::Ui) {
    ui.heading("Artifact Graph");
    ui.label(
        egui::RichText::new("Artifact lineage DAG — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

/// Renders the Forensics view — placeholder for debug trace output.
fn render_forensics(ui: &mut egui::Ui) {
    ui.heading("Forensics");
    ui.label(
        egui::RichText::new("Debug trace output — coming soon")
            .color(egui::Color32::from_rgb(128, 128, 128))
            .italics(),
    );
}

/// Renders tabs for all transcluded HUD fragments from the registry.
fn render_fragment_tabs(ui: &mut egui::Ui, registry: &HudFragmentRegistry) {
    for fragment in registry.fragments() {
        trace!("center_canvas — rendering fragment '{}'", fragment.provider);
        ui.collapsing(&fragment.title, |ui| {
            (fragment.render)(ui);
        });
    }
}
```

---

### Task 5: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/hud_transclusion.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to `integration.rs`**

After the existing `pub mod hud;` line (line 14), add:

```rust
    pub mod hud_transclusion;
```

The resulting `integration.rs` becomes:

```rust
//! Single integration-test binary for deer_gui.
//!
//! All test modules live under `integration/` and are compiled into one
//! binary to avoid the massive link-time overhead of Bevy test binaries
//! (each ~hundreds of MB). This keeps total disk usage manageable on the
//! RAM disk.

mod integration {
    pub mod audio;
    pub mod bridge;
    pub mod bridge_adapter;
    pub mod camera;
    pub mod constants_diagnostics;
    pub mod hud;
    pub mod hud_transclusion;
    pub mod models_pipeline;
    pub mod picking;
    pub mod scene_audio;
    pub mod scene_manager;
    pub mod scene_tet;
    pub mod scene_weather;
    pub mod theme;
    pub mod theme_world_colors;
    pub mod world;
    pub mod world_extended;
}
```

- [ ] **Step 2: Write test file**

```rust
//! Integration tests for HUD transclusion (fragment registry).
//!
//! Covers registry lifecycle: default state, register, unregister,
//! and multiple-provider coexistence. Does NOT require bevy_egui —
//! tests exercise the data-only API of `HudFragmentRegistry`.

use std::sync::Arc;

use deer_gui::hud::{HudFragment, HudFragmentRegistry};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a test fragment with a no-op render callback.
fn test_fragment(provider: &'static str, title: &str) -> HudFragment {
    HudFragment {
        provider,
        title: title.to_string(),
        render: Arc::new(|_ui| {}),
    }
}

// ---------------------------------------------------------------------------
// T-TRANS-01  Registry default is empty
// ---------------------------------------------------------------------------

#[test]
fn t_trans_01_registry_default_empty() {
    let registry = HudFragmentRegistry::default();
    assert!(
        registry.fragments().is_empty(),
        "Default registry must have 0 fragments"
    );
}

// ---------------------------------------------------------------------------
// T-TRANS-02  Register a fragment
// ---------------------------------------------------------------------------

#[test]
fn t_trans_02_register_fragment() {
    let mut registry = HudFragmentRegistry::default();
    registry.register(test_fragment("scene.tet", "Tetrahedron"));

    assert_eq!(
        registry.fragments().len(),
        1,
        "Registry must contain 1 fragment after register"
    );
    assert_eq!(registry.fragments()[0].provider, "scene.tet");
    assert_eq!(registry.fragments()[0].title, "Tetrahedron");
}

// ---------------------------------------------------------------------------
// T-TRANS-03  Unregister removes the fragment
// ---------------------------------------------------------------------------

#[test]
fn t_trans_03_unregister_removes() {
    let mut registry = HudFragmentRegistry::default();
    registry.register(test_fragment("swarm.monitor", "Swarm"));

    assert_eq!(registry.fragments().len(), 1);

    registry.unregister("swarm.monitor");
    assert!(
        registry.fragments().is_empty(),
        "Registry must be empty after unregister"
    );
}

// ---------------------------------------------------------------------------
// T-TRANS-04  Multiple providers coexist
// ---------------------------------------------------------------------------

#[test]
fn t_trans_04_multiple_providers() {
    let mut registry = HudFragmentRegistry::default();
    registry.register(test_fragment("scene.tet", "Tetrahedron"));
    registry.register(test_fragment("swarm.monitor", "Swarm Monitor"));

    assert_eq!(
        registry.fragments().len(),
        2,
        "Registry must hold both providers"
    );

    let providers: Vec<&str> = registry.fragments().iter().map(|f| f.provider).collect();
    assert!(providers.contains(&"scene.tet"));
    assert!(providers.contains(&"swarm.monitor"));
}
```

---

### Task 6: Build and test

- [ ] **Step 1: Run `cargo check`**

```bash
cargo check -p deer-gui 2>&1
```

Expected: No errors.

- [ ] **Step 2: Run all tests**

```bash
cargo test -p deer-gui 2>&1
```

Expected: All tests pass, including the 4 new `t_trans_*` tests.

- [ ] **Step 3: Verify LOC constraints**

```bash
wc -l apps/deer_gui/src/hud/transclusion.rs
wc -l apps/deer_gui/src/hud/center_canvas.rs
```

Expected: `transclusion.rs` < 100 LOC, `center_canvas.rs` < 200 LOC (was 153, adding ~20 lines).

---

### Task 7: Commit

- [ ] **Step 1: Stage and commit**

```bash
git add apps/deer_gui/src/hud/transclusion.rs \
       apps/deer_gui/src/hud/mod.rs \
       apps/deer_gui/src/hud/plugin.rs \
       apps/deer_gui/src/hud/center_canvas.rs \
       apps/deer_gui/tests/integration/hud_transclusion.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(hud): add HudFragmentRegistry for stateless content transclusion into Center Canvas"
```
