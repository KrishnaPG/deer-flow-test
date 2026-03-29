# PR D — Precursors Scene + Theme

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the Precursors scene and theme as a fully descriptor-driven scene with per-frame animation systems for river barges and path travellers, a Rust theme fallback, and 6 integration tests.

**Architecture:** Scene and theme defined by RON descriptor files (`assets/scenes/precursors.scene.ron`, `assets/themes/precursors.theme.ron`). A `PrecursorsSceneConfig` implements `SceneConfig` using `DescriptorSceneConfig` internally. Per-frame `barge_system` and `traveller_system` live in `src/scene/generators/systems.rs` (shared across all scenes). A compile-time `precursors_theme()` factory provides a fallback when descriptor loading is unavailable (e.g. integration tests).

**Tech Stack:** Rust, Bevy 0.18.1

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `assets/scenes/precursors.scene.ron` | Scene descriptor (starfield, barges, travellers, glow cluster) |
| Create | `assets/themes/precursors.theme.ron` | Theme descriptor (warm amber/earth palette) |
| Create | `src/theme/precursors_theme.rs` | `precursors_theme() -> Theme` compile-time fallback |
| Create | `src/scene/generators/systems.rs` | `barge_system`, `traveller_system` per-frame systems |
| Modify | `src/scene/generators/mod.rs` | Add `pub mod systems` + re-exports |
| Modify | `src/constants.rs` | Add Precursors-specific constants |
| Modify | `src/theme/mod.rs` | Add `pub mod precursors_theme` + re-export |
| Modify | `src/theme/plugin.rs` | Register Precursors theme in `ThemeManager` |
| Modify | `src/scene/plugin.rs` | Register `barge_system` and `traveller_system` in Update |
| Create | `tests/integration/scene_precursors.rs` | 6 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod scene_precursors` |

---

### Task 1: Add Precursors constants

**Files:**
- Modify: `apps/deer_gui/src/constants.rs`

- [ ] **Step 1: Add constants to the `visual` module**

In `src/constants.rs`, add the following constants inside the `pub mod visual` block, after the `DATA_TRAIL_COUNT` constant:

```rust
    /// Number of river barge entities in the Precursors scene.
    pub const PRECURSORS_BARGE_COUNT: usize = 60;
    /// Number of path traveller entities in the Precursors scene.
    pub const PRECURSORS_TRAVELLER_COUNT: usize = 80;
    /// Movement speed of Precursors river barges (units / sec).
    pub const PRECURSORS_BARGE_SPEED: f32 = 3.0;
    /// Movement speed of Precursors path travellers (units / sec).
    pub const PRECURSORS_TRAVELLER_SPEED: f32 = 2.0;
    /// Radius of the Precursors river barge orbit path.
    pub const PRECURSORS_RIVER_RADIUS: f32 = 300.0;
    /// Radius of the Precursors path traveller orbit.
    pub const PRECURSORS_PATH_RADIUS: f32 = 250.0;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation (possibly with existing warnings).

---

### Task 2: Create Precursors scene descriptor

**Files:**
- Create: `apps/deer_gui/assets/scenes/precursors.scene.ron`

- [ ] **Step 1: Write the descriptor file**

```ron
SceneDescriptor(
    name: "Precursors",
    ambient_audio: "audio/precursors_ambient.ogg",
    gltf_scene: None,
    theme: "Precursors",
    generators: [
        (generator: "starfield", params: Starfield(
            count: 1500,
            radius: 800.0,
            emissive: (1.8, 1.4, 0.8, 1.0),
        )),
        (generator: "river_barges", params: RiverBarges(
            count: 60,
            speed: 3.0,
            river_radius: 300.0,
            emissive: (0.6, 0.8, 0.4, 1.0),
        )),
        (generator: "path_travellers", params: PathTravellers(
            count: 80,
            speed: 2.0,
            path_radius: 250.0,
            emissive: (0.5, 0.7, 0.3, 1.0),
        )),
        (generator: "static_glow_cluster", params: StaticGlowCluster(
            count: 15,
            emissive: (0.8, 0.6, 0.3, 1.0),
            position: (800.0, -50.0, 0.0),
            spread: 100.0,
        )),
    ],
)
```

---

### Task 3: Create Precursors theme descriptor

**Files:**
- Create: `apps/deer_gui/assets/themes/precursors.theme.ron`

- [ ] **Step 1: Write the descriptor file**

```ron
ThemeDescriptor(
    name: "Precursors",
    background: (0.22, 0.16, 0.10, 1.0),
    surface: (0.30, 0.25, 0.18, 1.0),
    accent: (0.85, 0.60, 0.20, 1.0),
    accent_secondary: (0.35, 0.55, 0.30, 1.0),
    text_primary: (0.95, 0.90, 0.80, 1.0),
    text_secondary: (0.70, 0.65, 0.55, 1.0),
    success: (0.3, 0.7, 0.3, 1.0),
    warning: (0.9, 0.7, 0.1, 1.0),
    error: (0.8, 0.2, 0.2, 1.0),
    panel_alpha: 0.75,
    panel_rounding: 8.0,
    star_emissive: (1.8, 1.4, 0.8, 1.0),
    monolith_emissive: (0.8, 0.6, 0.3, 1.0),
    trail_emissive: (0.6, 0.8, 0.4, 1.0),
    trail_base_color: (0.5, 0.7, 0.3, 1.0),
    monolith_glow_channels: (0.8, 0.6, 0.3),
    font_css_url: None,
)
```

---

### Task 4: Create Precursors theme Rust fallback

**Files:**
- Create: `apps/deer_gui/src/theme/precursors_theme.rs`

- [ ] **Step 1: Write the theme factory**

```rust
//! Precursors theme — warm amber and earth tones evoking ancient river valleys.
//!
//! Compile-time fallback for when descriptor loading is unavailable
//! (e.g. integration tests without asset I/O).

use bevy::log::debug;

use super::theme::Theme;
use crate::constants::visual::{HUD_PANEL_ALPHA, HUD_PANEL_ROUNDING};

// ---------------------------------------------------------------------------
// Theme factory
// ---------------------------------------------------------------------------

/// Returns the Precursors theme.
///
/// Warm amber palette with earth-toned accents. Matches the values in
/// `assets/themes/precursors.theme.ron`.
pub fn precursors_theme() -> Theme {
    debug!("precursors_theme — constructing Precursors theme");
    Theme {
        name: "Precursors".to_string(),
        background: bevy::color::Color::srgba(0.22, 0.16, 0.10, 1.0),
        surface: bevy::color::Color::srgba(0.30, 0.25, 0.18, 1.0),
        accent: bevy::color::Color::srgba(0.85, 0.60, 0.20, 1.0),
        accent_secondary: bevy::color::Color::srgba(0.35, 0.55, 0.30, 1.0),
        text_primary: bevy::color::Color::srgba(0.95, 0.90, 0.80, 1.0),
        text_secondary: bevy::color::Color::srgba(0.70, 0.65, 0.55, 1.0),
        success: bevy::color::Color::srgba(0.3, 0.7, 0.3, 1.0),
        warning: bevy::color::Color::srgba(0.9, 0.7, 0.1, 1.0),
        error: bevy::color::Color::srgba(0.8, 0.2, 0.2, 1.0),
        panel_alpha: HUD_PANEL_ALPHA,
        panel_rounding: HUD_PANEL_ROUNDING,
        // World material colours
        star_emissive: bevy::color::LinearRgba::new(1.8, 1.4, 0.8, 1.0),
        monolith_emissive: bevy::color::LinearRgba::new(0.8, 0.6, 0.3, 1.0),
        trail_emissive: bevy::color::LinearRgba::new(0.6, 0.8, 0.4, 1.0),
        trail_base_color: bevy::color::Color::srgb(0.5, 0.7, 0.3),
        monolith_glow_channels: [0.8, 0.6, 0.3],
    }
}
```

---

### Task 5: Wire Precursors theme into theme module and plugin

**Files:**
- Modify: `apps/deer_gui/src/theme/mod.rs`
- Modify: `apps/deer_gui/src/theme/plugin.rs`

- [ ] **Step 1: Add `precursors_theme` to `theme/mod.rs`**

Add after `pub mod tet_theme;`:
```rust
pub mod precursors_theme;
```

Add to the re-exports:
```rust
pub use precursors_theme::precursors_theme;
```

The resulting `theme/mod.rs` should look like:
```rust
//! Theme engine — visual palette management for the Deer GUI.
//!
//! Re-exports the public API so that downstream modules can write
//! `use crate::theme::{ThemePlugin, ThemeManager, Theme};`.

mod plugin;
pub mod precursors_theme;
pub mod tet_theme;
mod theme;

pub use plugin::ThemePlugin;
pub use precursors_theme::precursors_theme;
pub use tet_theme::tet_theme;
pub use theme::{Theme, ThemeManager};
```

- [ ] **Step 2: Register Precursors theme in `ThemePlugin`**

In `src/theme/plugin.rs`, update the `ThemeManager::new(...)` call inside `ThemePlugin::build` to include the Precursors theme:

Change:
```rust
let manager = ThemeManager::new(vec![tet_theme()]);
```
to:
```rust
let manager = ThemeManager::new(vec![tet_theme(), super::precursors_theme::precursors_theme()]);
```

Also add the import at the top of `plugin.rs`:
```rust
use super::precursors_theme::precursors_theme;
```

Then the line becomes:
```rust
let manager = ThemeManager::new(vec![tet_theme(), precursors_theme()]);
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 6: Create per-frame generator systems

**Files:**
- Create: `apps/deer_gui/src/scene/generators/systems.rs`

- [ ] **Step 1: Write the systems file**

```rust
//! Per-frame systems for procedural generator entities.
//!
//! These systems advance parametric `t` values and update transforms
//! for entities spawned by generator factories. They are no-ops when
//! no matching entities exist in the world.

use bevy::log::trace;
use bevy::prelude::{Query, Res, Time, Transform};

use super::path_travellers::{self, Traveller};
use super::river_barges::{self, Barge};

// ---------------------------------------------------------------------------
// Barge system
// ---------------------------------------------------------------------------

/// Advances [`Barge`] entities along their river paths.
///
/// Each frame: `barge.t += barge.speed * dt * 0.01`, wraps at 1.0,
/// then updates the transform via [`river_barges::barge_position`].
///
/// No-op when no `Barge` entities exist.
pub fn barge_system(
    time: Res<Time>,
    mut query: Query<(&mut Barge, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut barge, mut tf) in query.iter_mut() {
        barge.t = (barge.t + barge.speed * dt * 0.01) % 1.0;
        tf.translation = river_barges::barge_position(barge.t, barge.index, barge.river_radius);

        trace!(
            "barge_system: index={} t={:.4} pos=({:.1},{:.1},{:.1})",
            barge.index,
            barge.t,
            tf.translation.x,
            tf.translation.y,
            tf.translation.z,
        );
    }
}

// ---------------------------------------------------------------------------
// Traveller system
// ---------------------------------------------------------------------------

/// Advances [`Traveller`] entities along their diagonal paths.
///
/// Each frame: `traveller.t += traveller.speed * dt * 0.01`, wraps at 1.0,
/// then updates the transform via [`path_travellers::traveller_position`].
///
/// No-op when no `Traveller` entities exist.
pub fn traveller_system(
    time: Res<Time>,
    mut query: Query<(&mut Traveller, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut traveller, mut tf) in query.iter_mut() {
        traveller.t = (traveller.t + traveller.speed * dt * 0.01) % 1.0;
        tf.translation = path_travellers::traveller_position(
            traveller.t,
            traveller.index,
            traveller.path_radius,
        );

        trace!(
            "traveller_system: index={} t={:.4} pos=({:.1},{:.1},{:.1})",
            traveller.index,
            traveller.t,
            tf.translation.x,
            tf.translation.y,
            tf.translation.z,
        );
    }
}
```

---

### Task 7: Wire generator systems into module and plugin

**Files:**
- Modify: `apps/deer_gui/src/scene/generators/mod.rs`
- Modify: `apps/deer_gui/src/scene/plugin.rs`

- [ ] **Step 1: Add `systems` to `generators/mod.rs`**

Add after the existing module declarations:
```rust
pub mod systems;
```

Add to the re-exports:
```rust
pub use systems::{barge_system, traveller_system};
```

- [ ] **Step 2: Register systems in `ScenePlugin`**

In `src/scene/plugin.rs`, in the `ScenePlugin::build` method, add the per-frame systems to the `Update` schedule:

```rust
use super::generators::systems::{barge_system, traveller_system};
```

Then in the `app.add_systems(...)` block:
```rust
app.add_systems(Update, (barge_system, traveller_system));
```

If the plugin already has an `Update` systems registration, append `barge_system` and `traveller_system` to the existing tuple.

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 8: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/scene_precursors.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to `integration.rs`**

Add inside the `mod integration` block:
```rust
pub mod scene_precursors;
```

- [ ] **Step 2: Write the test file**

```rust
//! Integration tests for the Precursors scene and theme.
//!
//! Verifies scene activation, entity counts, audio wiring,
//! deactivation cleanup, and scene switching from TET.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::constants::visual::{PRECURSORS_BARGE_COUNT, PRECURSORS_TRAVELLER_COUNT};
use deer_gui::scene::audio_bridge::SceneAudioState;
use deer_gui::scene::descriptor::{GeneratorParams, GeneratorSpec, SceneDescriptor};
use deer_gui::scene::descriptor_config::DescriptorSceneConfig;
use deer_gui::scene::generators::{Barge, Traveller};
use deer_gui::scene::primitives::Star;
use deer_gui::scene::tet::config::TetSceneConfig;
use deer_gui::scene::tet::setup::TetMonolith;
use deer_gui::scene::{SceneConfig, SceneManager, SceneRoot};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with asset support.
fn build_precursors_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

/// Creates a `PrecursorsSceneConfig` (DescriptorSceneConfig) matching the
/// RON descriptor values, for use in tests without file I/O.
fn precursors_scene_config() -> DescriptorSceneConfig {
    let descriptor = SceneDescriptor {
        name: "Precursors".to_string(),
        ambient_audio: "audio/precursors_ambient.ogg".to_string(),
        gltf_scene: None,
        theme: "Precursors".to_string(),
        generators: vec![
            GeneratorSpec {
                generator: "starfield".to_string(),
                params: GeneratorParams::Starfield {
                    count: 1500,
                    radius: 800.0,
                    emissive: [1.8, 1.4, 0.8, 1.0],
                },
            },
            GeneratorSpec {
                generator: "river_barges".to_string(),
                params: GeneratorParams::RiverBarges {
                    count: PRECURSORS_BARGE_COUNT,
                    speed: 3.0,
                    river_radius: 300.0,
                    emissive: [0.6, 0.8, 0.4, 1.0],
                },
            },
            GeneratorSpec {
                generator: "path_travellers".to_string(),
                params: GeneratorParams::PathTravellers {
                    count: PRECURSORS_TRAVELLER_COUNT,
                    speed: 2.0,
                    path_radius: 250.0,
                    emissive: [0.5, 0.7, 0.3, 1.0],
                },
            },
            GeneratorSpec {
                generator: "static_glow_cluster".to_string(),
                params: GeneratorParams::StaticGlowCluster {
                    count: 15,
                    emissive: [0.8, 0.6, 0.3, 1.0],
                    position: [800.0, -50.0, 0.0],
                    spread: 100.0,
                },
            },
        ],
    };
    DescriptorSceneConfig::new(descriptor)
}

/// Activates a scene by name within the app's world.
fn activate_scene(app: &mut App, name: &'static str) -> bool {
    let mut result = false;
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(
                    |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                        let mut commands = world.commands();
                        result = mgr.activate(
                            name,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            None,
                            None,
                        );
                    },
                );
            });
        });
    app.world_mut().flush();
    result
}

/// Activates a scene by name with audio state wired up.
fn activate_scene_with_audio(app: &mut App, name: &'static str) -> bool {
    let mut result = false;
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(
                    |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                        world.resource_scope(
                            |world: &mut World, mut audio_state: Mut<SceneAudioState>| {
                                let mut commands = world.commands();
                                result = mgr.activate(
                                    name,
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    None,
                                    Some(&mut audio_state),
                                );
                            },
                        );
                    },
                );
            });
        });
    app.world_mut().flush();
    result
}

/// Deactivates the current scene.
fn deactivate_scene(app: &mut App) {
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            let mut commands = world.commands();
            mgr.deactivate(&mut commands, None);
        });
    app.world_mut().flush();
}

/// Counts entities with a given component.
fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

/// Builds an app with both TET and Precursors scenes registered.
fn build_multi_scene_app() -> App {
    let mut app = build_precursors_app();
    let mut mgr = SceneManager::new();
    mgr.register(Box::new(TetSceneConfig));
    mgr.register(Box::new(precursors_scene_config()));
    app.insert_resource(mgr);
    app.insert_resource(SceneAudioState::default());
    app
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// t_prec_01: Activating Precursors sets the desired audio track.
#[test]
fn t_prec_01_activate_sets_audio() {
    let mut app = build_multi_scene_app();

    let activated = activate_scene_with_audio(&mut app, "Precursors");
    assert!(activated, "Precursors activation should succeed");

    let state = app.world().resource::<SceneAudioState>();
    assert_eq!(
        state.desired_track(),
        Some("audio/precursors_ambient.ogg"),
        "desired track should be Precursors ambient after activation"
    );
}

/// t_prec_02: Activating Precursors spawns exactly one SceneRoot.
#[test]
fn t_prec_02_root_spawned() {
    let mut app = build_multi_scene_app();

    activate_scene(&mut app, "Precursors");

    let root_count = count::<SceneRoot>(&mut app);
    assert_eq!(root_count, 1, "should have exactly one SceneRoot entity");
}

/// t_prec_03: Barge entity count matches descriptor value.
#[test]
fn t_prec_03_barge_count() {
    let mut app = build_multi_scene_app();

    activate_scene(&mut app, "Precursors");

    let barge_count = count::<Barge>(&mut app);
    assert_eq!(
        barge_count, PRECURSORS_BARGE_COUNT,
        "should spawn exactly {PRECURSORS_BARGE_COUNT} Barge entities"
    );
}

/// t_prec_04: Traveller entity count matches descriptor value.
#[test]
fn t_prec_04_traveller_count() {
    let mut app = build_multi_scene_app();

    activate_scene(&mut app, "Precursors");

    let traveller_count = count::<Traveller>(&mut app);
    assert_eq!(
        traveller_count, PRECURSORS_TRAVELLER_COUNT,
        "should spawn exactly {PRECURSORS_TRAVELLER_COUNT} Traveller entities"
    );
}

/// t_prec_05: Deactivating Precursors clears all scene entities.
#[test]
fn t_prec_05_deactivate_clears() {
    let mut app = build_multi_scene_app();

    activate_scene(&mut app, "Precursors");
    assert_eq!(count::<SceneRoot>(&mut app), 1);
    assert!(count::<Barge>(&mut app) > 0);
    assert!(count::<Traveller>(&mut app) > 0);

    deactivate_scene(&mut app);

    assert_eq!(count::<SceneRoot>(&mut app), 0, "SceneRoot should be despawned");
    assert_eq!(count::<Barge>(&mut app), 0, "Barge entities should be despawned");
    assert_eq!(count::<Traveller>(&mut app), 0, "Traveller entities should be despawned");
    assert_eq!(count::<Star>(&mut app), 0, "Star entities should be despawned");
}

/// t_prec_06: Switching from TET to Precursors removes TET entities
/// and spawns Precursors entities.
#[test]
fn t_prec_06_switch_from_tet() {
    let mut app = build_multi_scene_app();

    // Activate TET first.
    activate_scene(&mut app, "TET");
    assert_eq!(count::<SceneRoot>(&mut app), 1);
    assert!(count::<TetMonolith>(&mut app) > 0, "TET should have monolith");
    assert!(count::<Star>(&mut app) > 0, "TET should have stars");

    // Switch to Precursors.
    activate_scene(&mut app, "Precursors");

    // TET-specific entities should be gone.
    assert_eq!(
        count::<TetMonolith>(&mut app), 0,
        "TetMonolith should be despawned after switching to Precursors"
    );

    // Precursors entities should be present.
    assert_eq!(count::<SceneRoot>(&mut app), 1, "should have one SceneRoot");
    assert_eq!(
        count::<Barge>(&mut app), PRECURSORS_BARGE_COUNT,
        "Precursors barges should be spawned"
    );
    assert_eq!(
        count::<Traveller>(&mut app), PRECURSORS_TRAVELLER_COUNT,
        "Precursors travellers should be spawned"
    );

    // Verify manager state.
    let mgr = app.world().resource::<SceneManager>();
    assert_eq!(mgr.current_name(), Some("Precursors"));
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All existing tests pass plus the 6 new `t_prec_*` tests:
```
t_prec_01_activate_sets_audio ... ok
t_prec_02_root_spawned ... ok
t_prec_03_barge_count ... ok
t_prec_04_traveller_count ... ok
t_prec_05_deactivate_clears ... ok
t_prec_06_switch_from_tet ... ok
```

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p deer-gui -- -D warnings 2>&1`
Expected: No warnings.

- [ ] **Step 5: Commit**

```bash
git add apps/deer_gui/src/constants.rs \
       apps/deer_gui/assets/scenes/precursors.scene.ron \
       apps/deer_gui/assets/themes/precursors.theme.ron \
       apps/deer_gui/src/theme/precursors_theme.rs \
       apps/deer_gui/src/theme/mod.rs \
       apps/deer_gui/src/theme/plugin.rs \
       apps/deer_gui/src/scene/generators/systems.rs \
       apps/deer_gui/src/scene/generators/mod.rs \
       apps/deer_gui/src/scene/plugin.rs \
       apps/deer_gui/tests/integration/scene_precursors.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(scene): add Precursors scene, theme, per-frame systems, and 6 integration tests"
```
