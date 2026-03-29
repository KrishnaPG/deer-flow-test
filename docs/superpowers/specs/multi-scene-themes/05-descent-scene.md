# PR E — Descent Scene

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** PR A, PR B, PR C

---

## Visual Design

From design docs:

> Drop-ship observation deck descending through clouds of an alien world.

- **Cloud layer:** horizontal translucent particles rushing upward.
- **Drop-pods:** fast-falling particles (processing agents).
- **Colony beacons:** static dim-glow cluster at bottom (artifacts on ground).
- **Stars:** teal-white Fibonacci sphere starfield (above cloud layer).

Procedural terrain and cockpit-overlay details are deferred.

## Implementation: Descriptor-Driven

File: `assets/scenes/descent.scene.ron`

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

## Components

| Component        | Generator       | Fields                      |
| ---------------- | --------------- | --------------------------- |
| `CloudParticle`  | `cloud_layer`   | `t: f32`, `index: usize`   |
| `DropPod`        | `drop_pods`     | `t: f32`, `index: usize`   |
| `ColonyBeacon`   | `static_glow`   | (tag only)                  |

## Per-Frame Systems

| System                | Schedule | Behaviour                                       |
| --------------------- | -------- | ------------------------------------------------ |
| `cloud_system`        | Update   | Advances `CloudParticle.t` upward, wraps, updates Transform |
| `drop_pod_system`     | Update   | Advances `DropPod.t` downward, wraps, updates Transform     |

Both systems are no-ops when no matching entities exist.

## Theme

File: `assets/themes/descent.theme.ron`

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

A Rust factory `descent_theme()` in `src/theme/descent_theme.rs` serves as
compile-time fallback.

## Constants

Added to `constants.rs` under `visual`:

```rust
pub const DESCENT_CLOUD_COUNT: usize = 120;
pub const DESCENT_POD_COUNT: usize = 40;
pub const DESCENT_BEACON_COUNT: usize = 20;
pub const DESCENT_CLOUD_SPEED: f32 = 6.0;
pub const DESCENT_POD_SPEED: f32 = 8.0;
pub const DESCENT_CLOUD_RADIUS: f32 = 400.0;
```

## Tests

New file: `tests/integration/scene_descent.rs`

| Test                            | Verifies                                       |
| ------------------------------- | ---------------------------------------------- |
| `t_desc_01_activate_sets_audio` | `desired_track() == Some("audio/descent_ambient.ogg")` |
| `t_desc_02_root_spawned`        | `SceneRoot` count = 1                          |
| `t_desc_03_cloud_count`         | `CloudParticle` count matches descriptor       |
| `t_desc_04_pod_count`           | `DropPod` count matches descriptor             |
| `t_desc_05_beacon_count`        | `ColonyBeacon` count matches descriptor        |
| `t_desc_06_deactivate_clears`   | All counts = 0 after deactivate                |
