# PR E — Descent Scene + Theme

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the Descent scene (drop-ship observation deck descending through alien clouds) via data-driven descriptors, a Rust theme fallback, two per-frame animation systems (`cloud_system`, `drop_pod_system`), and Descent-specific constants.

**Architecture:** Scene and theme are defined in RON descriptors loaded by the existing `SceneLoader` + `GeneratorRegistry` pipeline (PRs A–C). `cloud_system` and `drop_pod_system` advance parametric `t` values each frame and update `Transform` positions. A `descent_theme()` factory provides a compile-time fallback identical to the RON theme descriptor. `GlowEntity` (from `static_glow.rs`) doubles as the colony-beacon tag.

**Tech Stack:** Rust, Bevy 0.18.1

**Depends on:** PR A (primitives), PR B (descriptors), PR C (generators)

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `assets/scenes/descent.scene.ron` | Descent scene descriptor |
| Create | `assets/themes/descent.theme.ron` | Descent theme descriptor |
| Create | `src/theme/descent_theme.rs` | `descent_theme() -> Theme` factory (compile-time fallback) |
| Modify | `src/theme/mod.rs` | Add `pub mod descent_theme` + re-export |
| Modify | `src/constants.rs` | Add Descent constants under `visual` |
| Create | `src/scene/generators/systems.rs` | Add `cloud_system` and `drop_pod_system` (appending to file if it exists from PR D, otherwise creating) |
| Modify | `src/scene/generators/mod.rs` | Add `pub mod systems` + re-exports for the new systems |
| Modify | `src/scene/plugin.rs` | Register `cloud_system`, `drop_pod_system` in Update; register Descent theme + scene |
| Create | `tests/integration/scene_descent.rs` | 6 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod scene_descent` |

---

### Task 1: Create Descent scene descriptor

**Files:**
- Create: `apps/deer_gui/assets/scenes/descent.scene.ron`

- [ ] **Step 1: Write the descriptor**

```ron
SceneDescriptor(
    name: "Descent",
    ambient_audio: "audio/descent_ambient.ogg",
    gltf_scene: None,
    theme: "Descent",
    generators: [
        (generator: "starfield", params: Starfield(
            count: 1000,
            radius: 900.0,
            emissive: (0.8, 0.9, 1.8, 1.0),
        )),
        (generator: "cloud_layer", params: CloudLayer(
            count: 120,
            speed: 6.0,
            radius: 400.0,
            emissive: (0.8, 0.9, 1.8, 1.0),
        )),
        (generator: "drop_pods", params: DropPods(
            count: 40,
            speed: 8.0,
            emissive: (0.3, 0.8, 0.9, 1.0),
        )),
        (generator: "static_glow_cluster", params: StaticGlowCluster(
            count: 20,
            emissive: (0.9, 0.4, 0.1, 1.0),
            position: (0.0, -600.0, 0.0),
            spread: 150.0,
        )),
    ],
)
```

---

### Task 2: Create Descent theme descriptor

**Files:**
- Create: `apps/deer_gui/assets/themes/descent.theme.ron`

- [ ] **Step 1: Write the descriptor**

```ron
ThemeDescriptor(
    name: "Descent",
    background: (0.05, 0.08, 0.12, 1.0),
    surface: (0.10, 0.15, 0.20, 1.0),
    accent: (0.90, 0.45, 0.10, 1.0),
    accent_secondary: (0.10, 0.70, 0.65, 1.0),
    text_primary: (0.95, 0.92, 0.88, 1.0),
    text_secondary: (0.65, 0.65, 0.70, 1.0),
    success: (0.3, 0.7, 0.3, 1.0),
    warning: (0.9, 0.7, 0.1, 1.0),
    error: (0.8, 0.2, 0.2, 1.0),
    panel_alpha: 0.75,
    panel_rounding: 8.0,
    star_emissive: (0.8, 0.9, 1.8, 1.0),
    monolith_emissive: (0.9, 0.4, 0.1, 1.0),
    trail_emissive: (0.3, 0.8, 0.9, 1.0),
    trail_base_color: (0.2, 0.6, 0.8, 1.0),
    monolith_glow_channels: (0.9, 0.4, 0.1),
    font_css_url: None,
)
```

---

### Task 3: Create Descent theme Rust fallback

**Files:**
- Create: `apps/deer_gui/src/theme/descent_theme.rs`

- [ ] **Step 1: Write the theme factory**

```rust
//! Descent theme — warm amber accents over deep blue-grey atmosphere.
//!
//! Compile-time fallback for the Descent scene. The RON descriptor
//! at `assets/themes/descent.theme.ron` is the authoritative source;
//! this factory provides identical values for use when the RON file
//! is unavailable.

use bevy::log::debug;

use super::theme::Theme;
use crate::constants::visual::{HUD_PANEL_ALPHA, HUD_PANEL_ROUNDING};

// ---------------------------------------------------------------------------
// Theme factory
// ---------------------------------------------------------------------------

/// Returns the Descent theme.
pub fn descent_theme() -> Theme {
    debug!("descent_theme — constructing Descent theme");
    Theme {
        name: "Descent".to_string(),
        background: bevy::color::Color::srgba(0.05, 0.08, 0.12, 1.0),
        surface: bevy::color::Color::srgba(0.10, 0.15, 0.20, 1.0),
        accent: bevy::color::Color::srgba(0.90, 0.45, 0.10, 1.0),
        accent_secondary: bevy::color::Color::srgba(0.10, 0.70, 0.65, 1.0),
        text_primary: bevy::color::Color::srgba(0.95, 0.92, 0.88, 1.0),
        text_secondary: bevy::color::Color::srgba(0.65, 0.65, 0.70, 1.0),
        success: bevy::color::Color::srgba(0.3, 0.7, 0.3, 1.0),
        warning: bevy::color::Color::srgba(0.9, 0.7, 0.1, 1.0),
        error: bevy::color::Color::srgba(0.8, 0.2, 0.2, 1.0),
        panel_alpha: HUD_PANEL_ALPHA,
        panel_rounding: HUD_PANEL_ROUNDING,
        // World-colour roles
        star_emissive: bevy::color::LinearRgba::new(0.8, 0.9, 1.8, 1.0),
        monolith_emissive: bevy::color::LinearRgba::new(0.9, 0.4, 0.1, 1.0),
        trail_emissive: bevy::color::LinearRgba::new(0.3, 0.8, 0.9, 1.0),
        trail_base_color: bevy::color::Color::srgb(0.2, 0.6, 0.8),
        monolith_glow_channels: [0.9, 0.4, 0.1],
    }
}
```

---

### Task 4: Wire Descent theme into `theme/mod.rs`

**Files:**
- Modify: `apps/deer_gui/src/theme/mod.rs`

- [ ] **Step 1: Add module and re-export**

Add after the `pub mod tet_theme;` line:

```rust
pub mod descent_theme;
```

Add to the re-exports:

```rust
pub use descent_theme::descent_theme;
```

The resulting `mod.rs` should look like:

```rust
//! Theme engine — visual palette management for the Deer GUI.
//!
//! Re-exports the public API so that downstream modules can write
//! `use crate::theme::{ThemePlugin, ThemeManager, Theme};`.

mod plugin;
pub mod descent_theme;
pub mod tet_theme;
mod theme;

pub use descent_theme::descent_theme;
pub use plugin::ThemePlugin;
pub use tet_theme::tet_theme;
pub use theme::{Theme, ThemeManager};
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation (or warnings only from unrelated code).

---

### Task 5: Add Descent constants

**Files:**
- Modify: `apps/deer_gui/src/constants.rs`

- [ ] **Step 1: Add Descent constants to the `visual` module**

Add the following block at the end of the `pub mod visual` block, before its closing `}`:

```rust
    // -- Descent scene --

    /// Number of cloud particles in the Descent scene.
    pub const DESCENT_CLOUD_COUNT: usize = 120;
    /// Number of drop-pod particles in the Descent scene.
    pub const DESCENT_POD_COUNT: usize = 40;
    /// Number of colony beacon entities in the Descent scene.
    pub const DESCENT_BEACON_COUNT: usize = 20;
    /// Cloud particle drift speed (units / sec).
    pub const DESCENT_CLOUD_SPEED: f32 = 6.0;
    /// Drop-pod fall speed (units / sec).
    pub const DESCENT_POD_SPEED: f32 = 8.0;
    /// Radius of the cloud particle distribution sphere.
    pub const DESCENT_CLOUD_RADIUS: f32 = 400.0;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 6: Create per-frame animation systems

**Files:**
- Create (or append to): `apps/deer_gui/src/scene/generators/systems.rs`

If `systems.rs` already exists from PR D (with `barge_system` and `traveller_system`), append the two new systems. If it does not exist, create the file with all four systems. The code below shows the file as if created fresh with all four systems; if appending, add only `cloud_system` and `drop_pod_system`.

- [ ] **Step 1: Write (or append) the systems file**

If the file **does not exist**, create it with:

```rust
//! Per-frame animation systems for procedural generator entities.
//!
//! Each system advances a parametric `t` value, wraps at 1.0, and
//! updates the entity's `Transform` using the generator's position
//! function. All systems are no-ops when no matching entities exist.

use bevy::log::trace;
use bevy::prelude::*;

use super::cloud_layer::{cloud_position, CloudParticle};
use super::drop_pods::{pod_position, DropPod};

use crate::constants::visual::DESCENT_CLOUD_RADIUS;

// ---------------------------------------------------------------------------
// Cloud system
// ---------------------------------------------------------------------------

/// Advances `CloudParticle` entities each frame.
///
/// Updates `cloud.t += cloud.speed * dt * 0.01`, wraps at 1.0,
/// recalculates position via `cloud_position(t, index, radius)`.
pub fn cloud_system(
    time: Res<Time>,
    mut query: Query<(&mut CloudParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut cloud, mut transform) in &mut query {
        cloud.t += cloud.speed * dt * 0.01;
        if cloud.t >= 1.0 {
            cloud.t -= 1.0;
        }

        let pos = cloud_position(cloud.t, cloud.index, DESCENT_CLOUD_RADIUS);
        transform.translation = pos;
    }

    trace!("cloud_system — dt={dt:.4}");
}

// ---------------------------------------------------------------------------
// Drop-pod system
// ---------------------------------------------------------------------------

/// Advances `DropPod` entities each frame.
///
/// Updates `pod.t += pod.speed * dt * 0.01`, wraps at 1.0,
/// recalculates position via `pod_position(t, index)`.
pub fn drop_pod_system(
    time: Res<Time>,
    mut query: Query<(&mut DropPod, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut pod, mut transform) in &mut query {
        pod.t += pod.speed * dt * 0.01;
        if pod.t >= 1.0 {
            pod.t -= 1.0;
        }

        let pos = pod_position(pod.t, pod.index);
        transform.translation = pos;
    }

    trace!("drop_pod_system — dt={dt:.4}");
}
```

If the file **already exists** from PR D (with `barge_system` and `traveller_system`), add the following imports at the top of the file:

```rust
use super::cloud_layer::{cloud_position, CloudParticle};
use super::drop_pods::{pod_position, DropPod};
use crate::constants::visual::DESCENT_CLOUD_RADIUS;
```

Then append the `cloud_system` and `drop_pod_system` functions (the function bodies above) after the existing systems.

- [ ] **Step 2: Update `generators/mod.rs`**

Add `pub mod systems;` if not already present. Add re-exports:

```rust
pub use systems::{cloud_system, drop_pod_system};
```

If PR D already added `pub use systems::{barge_system, traveller_system};`, extend to:

```rust
pub use systems::{barge_system, cloud_system, drop_pod_system, traveller_system};
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 7: Wire Descent into the plugin

**Files:**
- Modify: `apps/deer_gui/src/scene/plugin.rs`

- [ ] **Step 1: Import descent theme and systems**

Add to the imports:

```rust
use super::generators::systems::{cloud_system, drop_pod_system};
use crate::theme::descent_theme;
```

If PR D already imported `barge_system` / `traveller_system`, extend that import line.

- [ ] **Step 2: Register Descent theme in `ScenePlugin::build`**

In the `ThemeManager::new(...)` call (or wherever themes are registered), add `descent_theme()` to the themes vector:

```rust
// After tet_theme():
let theme_mgr = ThemeManager::new(vec![
    tet_theme(),
    descent_theme(),
]);
```

If themes are registered individually rather than in a vector, add:

```rust
// Register Descent theme as compile-time fallback.
// The RON descriptor is authoritative; this ensures the theme is
// available even if the asset directory is missing.
```

- [ ] **Step 3: Add per-frame systems to the Update schedule**

In the `.add_systems(Update, (...))` tuple, add:

```rust
cloud_system,
drop_pod_system,
```

The full Update tuple should now include (at minimum):

```rust
.add_systems(
    Update,
    (
        tet_glow_system,
        data_trail_system,
        cloud_system,
        drop_pod_system,
        atmosphere_transition_system,
        weather_update_system,
        weather_transition_system,
        scene_audio_bridge_system,
    ),
);
```

If PR D added `barge_system` and `traveller_system`, those should also be present.

- [ ] **Step 4: Register Descent scene config (DescriptorSceneConfig)**

After registering TET, add the Descent scene via `DescriptorSceneConfig`:

```rust
use super::descriptor_config::DescriptorSceneConfig;
use super::descriptor::SceneDescriptor;

// In ScenePlugin::build, after manager.register(Box::new(TetSceneConfig)):
let descent_ron = include_str!("../../assets/scenes/descent.scene.ron");
let descent_desc: SceneDescriptor = ron::from_str(descent_ron)
    .expect("descent.scene.ron should parse");
manager.register(Box::new(DescriptorSceneConfig::new(descent_desc)));
```

Alternatively, if the `SceneLoader` is already wired to scan `assets/scenes/` at startup and auto-register discovered descriptors, skip this step.

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 8: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/scene_descent.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to `integration.rs`**

Add inside the `mod integration { ... }` block:

```rust
pub mod scene_descent;
```

- [ ] **Step 2: Write the test file**

```rust
//! Integration tests for the Descent scene.
//!
//! Verifies scene activation, entity counts, audio wiring, and
//! clean deactivation via the `SceneManager` + `DescriptorSceneConfig`
//! pipeline.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::constants::visual::{
    DESCENT_BEACON_COUNT, DESCENT_CLOUD_COUNT, DESCENT_POD_COUNT,
};
use deer_gui::scene::audio_bridge::SceneAudioState;
use deer_gui::scene::descriptor::SceneDescriptor;
use deer_gui::scene::descriptor_config::DescriptorSceneConfig;
use deer_gui::scene::generators::{CloudParticle, DropPod, GlowEntity};
use deer_gui::scene::manager::SceneRoot;
use deer_gui::scene::SceneManager;
use deer_gui::theme::{descent_theme, ThemeManager};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse the Descent scene descriptor from the embedded RON.
fn descent_descriptor() -> SceneDescriptor {
    let ron_str = include_str!("../../assets/scenes/descent.scene.ron");
    ron::from_str(ron_str).expect("descent.scene.ron should parse")
}

/// Build a minimal Bevy app with asset support for scene testing.
fn build_descent_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

/// Count entities with a specific component.
fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

/// Set up a SceneManager with the Descent scene registered and activate it.
fn activate_descent(app: &mut App) {
    let desc = descent_descriptor();
    let mut manager = SceneManager::new();
    manager.register(Box::new(DescriptorSceneConfig::new(desc)));

    let theme_mgr = ThemeManager::new(vec![descent_theme()]);
    let mut audio_state = SceneAudioState::default();

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    manager.activate(
                        "Descent",
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Some(&theme_mgr),
                        Some(&mut audio_state),
                    );
                },
            );
        });
    app.world_mut().flush();

    app.insert_resource(manager);
    app.insert_resource(audio_state);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn t_desc_01_activate_sets_audio() {
    let mut app = build_descent_app();

    let desc = descent_descriptor();
    let mut manager = SceneManager::new();
    manager.register(Box::new(DescriptorSceneConfig::new(desc)));

    let theme_mgr = ThemeManager::new(vec![descent_theme()]);
    let mut audio_state = SceneAudioState::default();

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    manager.activate(
                        "Descent",
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Some(&theme_mgr),
                        Some(&mut audio_state),
                    );
                },
            );
        });

    assert_eq!(
        audio_state.desired_track(),
        Some("audio/descent_ambient.ogg"),
        "Descent activation should set desired audio track",
    );
}

#[test]
fn t_desc_02_root_spawned() {
    let mut app = build_descent_app();
    activate_descent(&mut app);

    assert_eq!(
        count::<SceneRoot>(&mut app),
        1,
        "Descent should spawn exactly one SceneRoot",
    );
}

#[test]
fn t_desc_03_cloud_count() {
    let mut app = build_descent_app();
    activate_descent(&mut app);

    assert_eq!(
        count::<CloudParticle>(&mut app),
        DESCENT_CLOUD_COUNT,
        "Descent should spawn DESCENT_CLOUD_COUNT={DESCENT_CLOUD_COUNT} CloudParticle entities",
    );
}

#[test]
fn t_desc_04_pod_count() {
    let mut app = build_descent_app();
    activate_descent(&mut app);

    assert_eq!(
        count::<DropPod>(&mut app),
        DESCENT_POD_COUNT,
        "Descent should spawn DESCENT_POD_COUNT={DESCENT_POD_COUNT} DropPod entities",
    );
}

#[test]
fn t_desc_05_beacon_count() {
    let mut app = build_descent_app();
    activate_descent(&mut app);

    assert_eq!(
        count::<GlowEntity>(&mut app),
        DESCENT_BEACON_COUNT,
        "Descent should spawn DESCENT_BEACON_COUNT={DESCENT_BEACON_COUNT} GlowEntity (beacon) entities",
    );
}

#[test]
fn t_desc_06_deactivate_clears() {
    let mut app = build_descent_app();
    activate_descent(&mut app);

    // Verify entities exist before deactivation.
    assert!(count::<SceneRoot>(&mut app) > 0, "pre-check: root exists");
    assert!(count::<CloudParticle>(&mut app) > 0, "pre-check: clouds exist");

    // Deactivate.
    app.world_mut()
        .resource_scope(|world: &mut World, mut manager: Mut<SceneManager>| {
            let mut commands = world.commands();
            manager.deactivate(&mut commands, None);
        });
    app.world_mut().flush();

    assert_eq!(count::<SceneRoot>(&mut app), 0, "SceneRoot should be despawned");
    assert_eq!(count::<CloudParticle>(&mut app), 0, "CloudParticle should be despawned");
    assert_eq!(count::<DropPod>(&mut app), 0, "DropPod should be despawned");
    assert_eq!(count::<GlowEntity>(&mut app), 0, "GlowEntity should be despawned");
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All existing tests pass plus 6 new Descent tests.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p deer-gui -- -D warnings 2>&1`
Expected: No errors.

---

### Task 9: Commit

- [ ] **Step 1: Stage and commit all changes**

```bash
git add apps/deer_gui/assets/scenes/descent.scene.ron \
       apps/deer_gui/assets/themes/descent.theme.ron \
       apps/deer_gui/src/theme/descent_theme.rs \
       apps/deer_gui/src/theme/mod.rs \
       apps/deer_gui/src/constants.rs \
       apps/deer_gui/src/scene/generators/systems.rs \
       apps/deer_gui/src/scene/generators/mod.rs \
       apps/deer_gui/src/scene/plugin.rs \
       apps/deer_gui/tests/integration/scene_descent.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(scene): add Descent scene + theme with cloud and drop-pod animation systems"
```
