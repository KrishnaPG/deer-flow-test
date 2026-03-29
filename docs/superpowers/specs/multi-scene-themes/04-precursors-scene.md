# PR D — Precursors Scene

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** PR A, PR B, PR C

---

## Visual Design

From `docs/design/control-center.md` and `wireframes.md`:

> Weathered stone watchtower at edge of cliff, overlooking a vast river valley.

- **Sky layer:** gradient quad (warm amber/orange static gradient).
- **River layer:** wide horizontal band of flowing "barge" particles.
- **Path layer:** diagonal stream of traveller particles.
- **Distant city (State Server):** static glow cluster on the horizon.
- **Stars:** warm-toned Fibonacci sphere starfield.

Time-of-day mapping and seasonal palettes are deferred.

## Implementation: Descriptor-Driven

The Precursors scene is defined by `assets/scenes/precursors.scene.ron`:

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

## Components

These are defined by the generator factories (PR C) and are
scene-independent:

| Component   | Generator         | Fields                     |
| ----------- | ----------------- | -------------------------- |
| `Barge`     | `river_barges`    | `t: f32`, `index: usize`  |
| `Traveller` | `path_travellers` | `t: f32`, `index: usize`  |

## Per-Frame Systems

| System                        | Schedule | Behaviour                                       |
| ----------------------------- | -------- | ------------------------------------------------ |
| `barge_system`                | Update   | Advances `Barge.t`, wraps at 1.0, updates Transform |
| `traveller_system`            | Update   | Advances `Traveller.t`, updates Transform        |

Both systems are no-ops when no matching entities exist.

## Theme

File: `assets/themes/precursors.theme.ron`

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

A Rust factory `precursors_theme()` is also provided in
`src/theme/precursors_theme.rs` as a compile-time fallback (used when
descriptor loading is unavailable, e.g. in integration tests).

## Constants

Added to `constants.rs` under `visual`:

```rust
pub const PRECURSORS_BARGE_COUNT: usize = 60;
pub const PRECURSORS_TRAVELLER_COUNT: usize = 80;
pub const PRECURSORS_BARGE_SPEED: f32 = 3.0;
pub const PRECURSORS_TRAVELLER_SPEED: f32 = 2.0;
pub const PRECURSORS_RIVER_RADIUS: f32 = 300.0;
pub const PRECURSORS_PATH_RADIUS: f32 = 250.0;
```

## Tests

New file: `tests/integration/scene_precursors.rs`

| Test                            | Verifies                                                    |
| ------------------------------- | ----------------------------------------------------------- |
| `t_prec_01_activate_sets_audio` | `desired_track() == Some("audio/precursors_ambient.ogg")`   |
| `t_prec_02_root_spawned`        | `SceneRoot` count = 1 after activation                      |
| `t_prec_03_barge_count`         | `Barge` entity count matches descriptor                     |
| `t_prec_04_traveller_count`     | `Traveller` entity count matches descriptor                 |
| `t_prec_05_deactivate_clears`   | All entity counts = 0 after deactivate                      |
| `t_prec_06_switch_from_tet`     | TET entities gone, Precursors entities present after switch  |
