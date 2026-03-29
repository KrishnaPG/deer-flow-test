# Design: Multi-Scene Themes — Shared Primitives, Precursors, Descent, Picking, HUD & Fonts

**Date:** 2026-03-29  
**Status:** Awaiting user review  
**Scope:** `apps/deer_gui`

---

## 1. Context & Goals

`apps/deer_gui` is a Bevy-based cinematic control center. The TET Orchestrator scene (starfield, monolith, data trails) is already fully implemented and managed by `SceneManager`. This design covers the remaining work to reach a fully multi-theme, production-quality state:

| PR | Deliverable | Status |
|----|-------------|--------|
| A | Shared scene primitives + TET refactor | New |
| B | Precursors scene | New |
| C | Descent scene | New |
| D | Two-phase picking unification | New |
| E | HUD transclusion plumbing | New |
| F | Font loading & typography | New |

**Non-negotiable constraints (AGENTS.md):**
- Files < 400 LOC, functions < 50 LOC.
- Zero-copy buffers; `&'static str` for all audio track paths.
- No unit tests, no mocks — integration tests using real Bevy `App`.
- All constants in `constants.rs`; never inline magic numbers.
- Dynamic tracing/debug logs in all methods.
- Very low coupling — every module reusable in isolation.

---

## 2. Stable Decisions (Already Implemented)

These APIs are frozen. New work must conform to them:

| Interface | Location | Notes |
|-----------|----------|-------|
| `SceneConfig` trait | `src/scene/traits.rs` | `spawn_environment(…) -> Entity`, `ambient_audio_track() -> &'static str`, `on_activate/deactivate` |
| `SceneManager` resource | `src/scene/manager.rs` | `register`, `activate`, `deactivate`, `current_name` |
| `SceneRoot` component | `src/scene/manager.rs` | Tags the single root entity per scene; recursive despawn |
| `SceneAudioState` resource | `src/scene/audio_bridge.rs` | `request_ambient(&'static str)`, `request_stop()`, `desired_track()` |
| `ThemeManager` resource | `src/theme/theme.rs` | `current() -> &Theme`; `generation: u64` dirty-check |
| `Theme` struct | `src/theme/theme.rs` | Includes `star_emissive`, `monolith_emissive`, `trail_emissive`, `trail_base_color`, `monolith_glow_channels: [f32; 3]` |
| Audio constants | `src/constants.rs` | `SCENE_AUDIO_FADE_IN_SECS=2.0`, `SCENE_AUDIO_FADE_OUT_SECS=1.5` |
| Test helper pattern | `tests/integration/scene_manager.rs` | `build_scene_mgr_app()` + nested `resource_scope` |

---

## 3. PR A — Shared Scene Primitives

### 3.1 Problem

`spawn_tet_environment` contains private helpers (`spawn_starfield`, `spawn_monolith`, `spawn_data_trails`, `extract_world_colors`, `pseudo_random_sphere_point`, `random_scale`) that will be re-used by Precursors and Descent. If each scene duplicates these, we accumulate diverging implementations.

### 3.2 New Module: `src/scene/primitives.rs`

A single, scene-independent file exposing reusable spawn helpers. All functions accept the minimal arguments needed and return nothing (they side-effect `Commands`/`Assets`). Each function is < 50 LOC.

#### Public API

```rust
/// Spawn a SceneRoot entity and return its Entity id.
pub fn spawn_root(commands: &mut Commands) -> Entity

/// Spawn `count` stars distributed on a Fibonacci sphere of `radius`.
/// Emissive color sourced from `emissive`. Each star gets `ParallaxLayer`.
/// All entities are `ChildOf(root)`.
pub fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: LinearRgba,
    count: usize,
    radius: f32,
)

/// Spawn a single ambient-light entity parented to root.
/// Brightness and color sourced from `brightness` / `color`.
pub fn spawn_scene_ambient_light(
    commands: &mut Commands,
    root: Entity,
    brightness: f32,
    color: LinearRgba,
)

/// Deterministic Fibonacci-sphere point at index `i` of `total`.
/// Public so each scene can call it directly for custom spawning.
pub fn fibonacci_sphere_point(index: usize, total: usize, radius: f32) -> Vec3

/// Deterministic per-entity scale in [0.3, 1.0] using the index.
pub fn entity_scale(index: usize) -> f32
```

#### What stays in `tet/setup.rs`

`spawn_monolith`, `spawn_data_trails`, and `spiral_position` remain TET-specific — they use TET-specific constants and components (`TetMonolith`, `DataTrail`). TET's `spawn_starfield` call is replaced by the primitive.

#### What moves

`pseudo_random_sphere_point` → renamed `fibonacci_sphere_point` in `primitives.rs` (same algorithm, better name).  
`random_scale` → renamed `entity_scale` in `primitives.rs`.  
`Star` component → moved to `primitives.rs` (it is a generic "starfield entity" marker, not TET-specific). Re-exported from `tet/setup.rs` as `pub use crate::scene::primitives::Star` for backward compatibility with existing tests.  
Private `spawn_starfield` in `tet/setup.rs` → replaced with a call to `primitives::spawn_starfield`.

#### Export

`src/scene/mod.rs` adds:

```rust
pub mod primitives;
pub use primitives::{spawn_root, spawn_starfield, spawn_scene_ambient_light,
                     fibonacci_sphere_point, entity_scale};
```

### 3.3 TET Refactor

`tet/setup.rs::spawn_tet_environment` is updated to:
1. Call `primitives::spawn_root` instead of inline `commands.spawn((SceneRoot, …))`.
2. Call `primitives::spawn_starfield` instead of the private version.
3. Remove the now-redundant private `pseudo_random_sphere_point` and `random_scale`.

Behaviour is identical. All existing TET tests remain green.

### 3.4 Tests

New file: `tests/integration/scene_primitives.rs`

| Test | What it verifies |
|------|-----------------|
| `t_prim_01_spawn_root_creates_scene_root` | `spawn_root` returns entity with `SceneRoot` component |
| `t_prim_02_spawn_starfield_count` | Calling `spawn_starfield(…, count=50, …)` produces 50 `Star` entities with `ParallaxLayer` |
| `t_prim_03_fibonacci_sphere_within_radius` | 200 points all within `radius + ε` |
| `t_prim_04_entity_scale_in_range` | 200 indices all in `[0.29, 1.01]` |
| `t_prim_05_tet_unchanged_after_refactor` | After refactor, activating TET still produces expected `Star` count and `TetMonolith` |

---

## 4. PR B — Precursors Scene

### 4.1 Visual Design

From `docs/design/control-center.md` and `wireframes.md`:

> Weathered stone watchtower at edge of cliff, overlooking a vast river valley.

- **Sky layer:** gradient quad (dawn/dusk using `time_of_day` later; for now, warm amber/orange static gradient).
- **River layer:** wide horizontal band of flowing "barge" particles.
- **Path layer:** diagonal stream of traveller particles.
- **Distant city (State Server):** static glow cluster on the horizon.
- **Agents:** represented as barge/traveller dots; for MVP, use the same `ParallaxLayer` + sphere mesh approach as TET with earth-tone emissive.

**MVP scope (this PR):** Sky background quad, river barge particles, path travellers. City horizon is a static dim glow cluster. Seasonal/time-of-day mapping is deferred.

### 4.2 New Files

```
src/scene/precursors/
├── mod.rs       — pub re-exports
├── config.rs    — PrecursorsSceneConfig impl SceneConfig
├── setup.rs     — spawn_precursors_environment() -> Entity
└── systems.rs   — precursors_barge_system, precursors_traveller_system
```

### 4.3 Components

```rust
// In precursors/setup.rs
#[derive(Component, Debug, Clone)]
pub struct Barge { pub t: f32 }   // parametric pos along river [0,1)

#[derive(Component, Debug, Default)]
pub struct Traveller { pub t: f32 } // parametric pos along path [0,1)

#[derive(Component, Debug, Default)]
pub struct PrecursorsCityGlow; // tag for horizon glow entities
```

### 4.4 Config

```rust
impl SceneConfig for PrecursorsSceneConfig {
    fn name(&self) -> &str { "Precursors" }
    fn ambient_audio_track(&self) -> &'static str { "audio/precursors_ambient.ogg" }
    fn spawn_environment(&self, …, theme) -> Entity { spawn_precursors_environment(…, theme) }
}
```

### 4.5 Theme

New file: `src/theme/precursors_theme.rs`

```rust
pub fn precursors_theme() -> Theme {
    Theme {
        name: "Precursors".into(),
        // Earth tones: warm amber/ochre backgrounds, stone grey surface
        background: Color::srgb(0.22, 0.16, 0.10),
        surface: Color::srgb(0.30, 0.25, 0.18),
        accent: Color::srgb(0.85, 0.60, 0.20),       // amber
        accent_secondary: Color::srgb(0.35, 0.55, 0.30), // muted green
        text_primary: Color::srgb(0.95, 0.90, 0.80),
        text_secondary: Color::srgb(0.70, 0.65, 0.55),
        // World colors
        star_emissive: LinearRgba::new(1.8, 1.4, 0.8, 1.0),  // warm stars
        monolith_emissive: LinearRgba::new(0.8, 0.6, 0.3, 1.0), // city glow
        trail_emissive: LinearRgba::new(0.6, 0.8, 0.4, 1.0),   // river/path barges
        trail_base_color: Color::srgb(0.5, 0.7, 0.3),
        monolith_glow_channels: [0.8, 0.6, 0.3],
        // Standard UI fields
        success: Color::srgb(0.3, 0.7, 0.3),
        warning: Color::srgb(0.9, 0.7, 0.1),
        error: Color::srgb(0.8, 0.2, 0.2),
        panel_alpha: 0.75,
        panel_rounding: 8.0,
    }
}
```

`ThemePlugin` is updated to register `precursors_theme()` alongside `tet_theme()`.

### 4.6 Constants

New constants in `constants.rs` under `visual` module:

```rust
pub const PRECURSORS_BARGE_COUNT: usize = 60;
pub const PRECURSORS_TRAVELLER_COUNT: usize = 80;
pub const PRECURSORS_BARGE_SPEED: f32 = 3.0;
pub const PRECURSORS_TRAVELLER_SPEED: f32 = 2.0;
pub const PRECURSORS_RIVER_RADIUS: f32 = 300.0;
pub const PRECURSORS_PATH_RADIUS: f32 = 250.0;
```

### 4.7 Systems

`precursors_barge_system` — advances each `Barge.t` by `PRECURSORS_BARGE_SPEED * dt * 0.01`, wraps at 1.0, updates `Transform` using `barge_position(t, index)` helper.

`precursors_traveller_system` — same pattern for `Traveller` using `traveller_position(t, index)`.

Both systems are registered in `ScenePlugin` (Update schedule). They are no-ops if no `Barge`/`Traveller` entities exist (when TET or Descent is active).

### 4.8 Plugin Wiring

`ScenePlugin::build`:
1. `manager.register(Box::new(PrecursorsSceneConfig))`.
2. `app.add_systems(Update, (precursors_barge_system, precursors_traveller_system))`.

### 4.9 Tests

New file: `tests/integration/scene_precursors.rs`

| Test | What it verifies |
|------|-----------------|
| `t_prec_01_activate_sets_audio` | Activate Precursors → `SceneAudioState.desired_track() == Some("audio/precursors_ambient.ogg")` |
| `t_prec_02_root_spawned` | `SceneRoot` count = 1 after activation |
| `t_prec_03_barge_count` | `Barge` entity count = `PRECURSORS_BARGE_COUNT` |
| `t_prec_04_traveller_count` | `Traveller` entity count = `PRECURSORS_TRAVELLER_COUNT` |
| `t_prec_05_deactivate_clears` | After deactivate: `SceneRoot=0`, `Barge=0`, `Traveller=0` |
| `t_prec_06_switch_from_tet` | Activate TET → activate Precursors: `Star=0`, `TetMonolith=0`, `Barge=PRECURSORS_BARGE_COUNT` |

---

## 5. PR C — Descent Scene

### 5.1 Visual Design

From design docs:

> Drop-ship observation deck descending through clouds of an alien world.

- **Cloud layer:** horizontal translucent quad particles rushing past (upward motion).
- **Drop-pods:** fast-falling particles (processing agents).
- **Colony beacons:** static dim-glow cluster at bottom (artifacts on ground).

**MVP scope:** Cloud particles (rising), drop-pod particles (falling), colony beacons (static glow cluster). Procedural terrain and cockpit-overlay details deferred.

### 5.2 New Files

```
src/scene/descent/
├── mod.rs
├── config.rs    — DescentSceneConfig impl SceneConfig
├── setup.rs     — spawn_descent_environment() -> Entity
└── systems.rs   — descent_cloud_system, descent_pod_system
```

### 5.3 Components

```rust
#[derive(Component, Debug, Clone)]
pub struct CloudParticle { pub t: f32 }  // parametric pos [0,1), upward motion

#[derive(Component, Debug, Clone)]
pub struct DropPod { pub t: f32 }        // parametric pos [0,1), downward motion

#[derive(Component, Debug, Default)]
pub struct ColonyBeacon;                 // static ground glow tag
```

### 5.4 Config

```rust
impl SceneConfig for DescentSceneConfig {
    fn name(&self) -> &str { "Descent" }
    fn ambient_audio_track(&self) -> &'static str { "audio/descent_ambient.ogg" }
    fn spawn_environment(&self, …, theme) -> Entity { spawn_descent_environment(…, theme) }
}
```

### 5.5 Theme

New file: `src/theme/descent_theme.rs`

```rust
pub fn descent_theme() -> Theme {
    Theme {
        name: "Descent".into(),
        // Oranges/teals — dynamic, dramatic
        background: Color::srgb(0.05, 0.08, 0.12),
        surface: Color::srgb(0.10, 0.15, 0.20),
        accent: Color::srgb(0.90, 0.45, 0.10),       // orange
        accent_secondary: Color::srgb(0.10, 0.70, 0.65), // teal
        text_primary: Color::srgb(0.95, 0.92, 0.88),
        text_secondary: Color::srgb(0.65, 0.65, 0.70),
        // World colors
        star_emissive: LinearRgba::new(0.8, 0.9, 1.8, 1.0),   // teal-white clouds
        monolith_emissive: LinearRgba::new(0.9, 0.4, 0.1, 1.0), // orange beacon
        trail_emissive: LinearRgba::new(0.3, 0.8, 0.9, 1.0),   // teal drop-pods
        trail_base_color: Color::srgb(0.2, 0.6, 0.8),
        monolith_glow_channels: [0.9, 0.4, 0.1],
        success: Color::srgb(0.3, 0.7, 0.3),
        warning: Color::srgb(0.9, 0.7, 0.1),
        error: Color::srgb(0.8, 0.2, 0.2),
        panel_alpha: 0.75,
        panel_rounding: 8.0,
    }
}
```

### 5.6 Constants

New in `constants.rs` under `visual`:

```rust
pub const DESCENT_CLOUD_COUNT: usize = 120;
pub const DESCENT_POD_COUNT: usize = 40;
pub const DESCENT_BEACON_COUNT: usize = 20;
pub const DESCENT_CLOUD_SPEED: f32 = 6.0;
pub const DESCENT_POD_SPEED: f32 = 8.0;
pub const DESCENT_CLOUD_RADIUS: f32 = 400.0;
```

### 5.7 Systems

`descent_cloud_system` — advances `CloudParticle.t` upward, wraps at 1.0, updates `Transform` using `cloud_position(t, index)`.

`descent_pod_system` — same pattern for `DropPod`, downward motion, using `pod_position(t, index)`.

### 5.8 Tests

New file: `tests/integration/scene_descent.rs`

| Test | What it verifies |
|------|-----------------|
| `t_desc_01_activate_sets_audio` | `desired_track() == Some("audio/descent_ambient.ogg")` |
| `t_desc_02_root_spawned` | `SceneRoot` count = 1 |
| `t_desc_03_cloud_count` | `CloudParticle` count = `DESCENT_CLOUD_COUNT` |
| `t_desc_04_pod_count` | `DropPod` count = `DESCENT_POD_COUNT` |
| `t_desc_05_beacon_count` | `ColonyBeacon` count = `DESCENT_BEACON_COUNT` |
| `t_desc_06_deactivate_clears` | All counts = 0 after deactivate |

---

## 6. PR D — Two-Phase Picking Unification

### 6.1 Current State

`src/picking/` has four systems chained in `Update`:
1. `spatial_index_rebuild_system` — rebuilds `SpatialIndex` from all `Transform + Selectable` entities.
2. `selection_update_system` — processes `EntityClicked` messages; adds/removes `Selected`.
3. `deselection_system` — clears selection on unoccupied click.
4. `selection_sync_system` — reads `Selected + WorldEntity`, writes to `HudState::selected_entity`.

This is functional but the two phases (coarse spatial query vs precise picking) are conflated in `spatial_index_rebuild_system`. "Two-phase" means:

- **Phase 1 (Coarse):** Spatial query — given a screen point, find candidate entities within a configurable screen-space radius. Result: `Vec<Entity>` candidates. Uses `SpatialIndex`.
- **Phase 2 (Precise):** From candidates, select the closest entity to the click point. Result: single `Entity` (or none). Emits `EntityClicked`.

The current code does this implicitly inside the observer `on_entity_clicked`. This PR makes it explicit, testable, and decoupled.

### 6.2 Design

#### New types in `picking/systems.rs`

```rust
/// Result of phase-1 coarse query — candidates near a screen point.
#[derive(Resource, Default)]
pub struct PickingCandidates {
    pub screen_pos: Vec2,
    pub candidates: Vec<Entity>,
}
```

#### System split

| System | Phase | Input | Output |
|--------|-------|-------|--------|
| `spatial_index_rebuild_system` | Pre-phase | ECS query | `SpatialIndex` (unchanged) |
| `coarse_picking_system` | Phase 1 | `PickingCandidates.screen_pos` (set by input handler) | `PickingCandidates.candidates` |
| `precise_picking_system` | Phase 2 | `PickingCandidates` | Emits `EntityClicked` message if best candidate found |
| `selection_update_system` | Post | `EntityClicked` messages | `Selected` component changes + `SelectionChanged` |
| `deselection_system` | Post | Input (no click) | Clears `Selected` |
| `selection_sync_system` | Post | `Selected + WorldEntity` | `HudState::selected_entity` |

The `on_entity_clicked` observer is removed; picking is now fully system-driven and testable.

#### Input flow

The existing Bevy `Pointer<Click>` observer is replaced by a thin observer that only writes `PickingCandidates.screen_pos`, then the chained systems run as before. This keeps Bevy's input detection while making the pipeline testable.

### 6.3 Constants

New in `constants.rs` under `visual`:

```rust
pub const PICKING_COARSE_RADIUS_PX: f32 = 24.0;
```

### 6.4 Tests

Extend `tests/integration/picking.rs`:

| Test | What it verifies |
|------|-----------------|
| `t_pick_07_coarse_candidates_populated` | Set `PickingCandidates.screen_pos` near a `Selectable` entity; run `coarse_picking_system`; assert candidates non-empty |
| `t_pick_08_precise_selects_closest` | Two candidates at different distances; `precise_picking_system` emits `EntityClicked` for the closer one |
| `t_pick_09_no_candidates_no_click` | Empty candidates → no `EntityClicked` message |
| `t_pick_10_deselection_clears_selected` | `deselection_system` removes `Selected` when no click this frame |

---

## 7. PR E — HUD Transclusion Plumbing

### 7.1 Problem

The Center Canvas (`center_canvas.rs`) has four stub modes (SwarmMonitor, ArtifactGraph, Forensics, ExpertsMeeting's thread_cache). Scenes and other subsystems need a way to push content fragments into the center canvas without directly calling HUD internals.

### 7.2 Design

#### `src/hud/transclusion.rs` (new file, < 100 LOC)

```rust
/// A single content fragment pushed by any subsystem into the Center Canvas.
#[derive(Debug, Clone)]
pub struct HudFragment {
    /// Unique provider ID (e.g. "scene.tet", "swarm.monitor").
    pub provider: &'static str,
    /// Display title shown in the canvas tab.
    pub title: String,
    /// The rendered content callback, called each egui frame when this fragment is active.
    /// Uses `Arc<dyn Fn(&mut egui::Ui) + Send + Sync>` for Send safety.
    pub render: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>,
}

/// Resource holding all currently registered fragments.
/// Populated by any system; consumed by `center_canvas_system`.
#[derive(Resource, Default)]
pub struct HudFragmentRegistry {
    fragments: Vec<HudFragment>,
}

impl HudFragmentRegistry {
    pub fn register(&mut self, fragment: HudFragment) { … }
    pub fn unregister(&mut self, provider: &'static str) { … }
    pub fn fragments(&self) -> &[HudFragment] { … }
}
```

`center_canvas_system` reads `HudFragmentRegistry` and renders registered fragments as tabs (alongside built-in modes). This is purely additive — existing modes are untouched.

### 7.3 Integration

`HudPlugin::build` calls `app.init_resource::<HudFragmentRegistry>()`.

No scene directly imports from `hud`; scenes push fragments via a thin public API.

### 7.4 Tests

New file: `tests/integration/hud_transclusion.rs`

| Test | What it verifies |
|------|-----------------|
| `t_trans_01_registry_default_empty` | `HudFragmentRegistry` starts with 0 fragments |
| `t_trans_02_register_fragment` | After `register(fragment)`, `fragments().len() == 1` |
| `t_trans_03_unregister_removes` | After `unregister("scene.tet")`, count drops to 0 |
| `t_trans_04_multiple_providers` | Two fragments with different providers both present |

---

## 8. PR F — Font Loading & Typography

### 8.1 Goals

The design calls for IBM Plex Sans (body) + JetBrains Mono (code/data). Fonts must:
1. Load deterministically in tests (no network, no GPU).
2. Be bundled in the repo under `apps/deer_gui/assets/fonts/`.
3. Be applied via `ThemePlugin` at startup (egui font definitions).

### 8.2 Fonts to bundle

| File | Usage |
|------|-------|
| `assets/fonts/IBMPlexSans-Regular.ttf` | Body text, labels |
| `assets/fonts/IBMPlexSans-Medium.ttf` | Subheadings |
| `assets/fonts/IBMPlexSans-Bold.ttf` | Headings, panel titles |
| `assets/fonts/JetBrainsMono-Regular.ttf` | Code, data values, IDs |

Source: Open source / SIL OFL licensed. Bundled for determinism.

### 8.3 New Module: `src/ui/fonts.rs`

```
src/ui/
├── mod.rs
└── fonts.rs   — font loader
```

```rust
/// Builds egui FontDefinitions with IBM Plex Sans + JetBrains Mono.
/// Called once by ThemePlugin on startup.
pub fn build_font_definitions() -> egui::FontDefinitions { … }
```

`ThemePlugin::build` calls `ctx.set_fonts(build_font_definitions())` during the first `EguiPrimaryContextPass` frame.

### 8.4 Tests

New test in existing `tests/integration/theme.rs` or new `tests/integration/fonts.rs`:

| Test | What it verifies |
|------|-----------------|
| `t_font_01_definitions_build` | `build_font_definitions()` returns non-empty `FontDefinitions` with expected font families |

---

## 9. Data Flow Summary

```
ScenePlugin::build
  ├── register TET, Precursors, Descent into SceneManager
  ├── add systems: tet_glow, data_trail, precursors_barge, precursors_traveller,
  │                descent_cloud, descent_pod, scene_audio_bridge, weather/atmosphere
  └── Startup: activate "TET" via SceneManager

Runtime scene switch (e.g. user selects "Precursors" in Settings modal):
  SceneManager::activate("Precursors", …, theme, audio_state)
    ├── deactivate TET: on_deactivate(), despawn(tet_root), audio_state.request_stop()
    └── activate Precursors: spawn_precursors_environment(…, theme), audio_state.request_ambient("audio/precursors_ambient.ogg")
  → next frame: scene_audio_bridge_system sends AudioCommand::StopAmbient + PlayAmbient

HUD transclusion example:
  TetGlowSystem (or any system) registers HudFragment{provider:"scene.tet", title:"TET Status", render:…}
  → center_canvas_system reads HudFragmentRegistry, renders "TET Status" tab
```

---

## 10. File Size & LOC Budget

| File | Estimated LOC |
|------|--------------|
| `scene/primitives.rs` | ~120 |
| `scene/precursors/config.rs` | ~45 |
| `scene/precursors/setup.rs` | ~180 |
| `scene/precursors/systems.rs` | ~80 |
| `scene/precursors/mod.rs` | ~15 |
| `scene/descent/config.rs` | ~45 |
| `scene/descent/setup.rs` | ~160 |
| `scene/descent/systems.rs` | ~80 |
| `scene/descent/mod.rs` | ~15 |
| `theme/precursors_theme.rs` | ~80 |
| `theme/descent_theme.rs` | ~80 |
| `hud/transclusion.rs` | ~90 |
| `ui/fonts.rs` | ~60 |
| `ui/mod.rs` | ~10 |
| `picking/` changes | ~+60 diff |
| `constants.rs` additions | ~+20 diff |

All within the < 400 LOC per file constraint.

---

## 11. Merge Order & Dependencies

```
PR A (primitives + TET refactor) — no deps, merge first
  ↓
PR B (Precursors) ─┐  both depend on PR A's primitives
PR C (Descent)    ─┘  can be developed and merged in parallel
  ↓
PR D (Picking)    — independent of scenes; can merge any time after PR A
PR E (HUD)        — independent of scenes; can merge any time after PR A
PR F (Fonts)      — fully independent, can merge any time
```

---

## 12. Open Questions (None Blocking)

- **Time-of-day mapping for Precursors sky:** Not in MVP scope; sky gradient is static amber. Revisit post-MVP.
- **Seasonal palette changes:** Not in scope for this plan.
- **Settings UI for scene/theme switching:** `Settings` modal stub exists (`hud/modal.rs`). Wiring scene switching to the Settings modal is a follow-up task after scene implementations are complete.
- **Audio files:** `audio/precursors_ambient.ogg` and `audio/descent_ambient.ogg` must exist in `assets/audio/` before scenes can play audio. For now, the asset paths are registered correctly; the audio system will gracefully no-op if files are absent. Asset creation is out of scope for this design.
- **Parallax scroll fix:** `PreviousCameraPosition` is never updated after the parallax system reads it. This is a pre-existing bug; fixing it is deferred to a separate PR.
