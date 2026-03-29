# Design: Multi-Scene Themes — Index

**Date:** 2026-03-29 (revised 2026-03-29)
**Status:** Approved
**Scope:** `apps/deer_gui`

---

## Overview

`apps/deer_gui` is a Bevy 0.18.1 cinematic game-engine GUI for AI research
orchestration. The TET Orchestrator scene is fully implemented and managed by
`SceneManager`. This design covers the remaining work to reach a fully
data-driven, multi-theme, production-quality state.

## Design Principles

These override earlier drafts:

1. **Data-driven** — Scenes and themes are defined by descriptors
   (RON/JSON/TOML) loaded at runtime. No recompilation to add a scene.
2. **No Bevy wrappers** — Use Bevy's native APIs directly. If Bevy supports
   something (glTF, PBR, animation, picking, audio), use it without
   abstraction layers.
3. **Production grade** — Every feature implemented must be production quality.
   If it can't be done well, leave it out.
4. **Battle-tested libraries** — For procedural generation, particles, physics,
   use existing open-source crates. Custom code only when nothing pre-exists.
5. **Stateless presentation** — Data clearly separated from presentation
   everywhere. HUD code reads data, never owns it.
6. **CSS font standards** — Fonts loaded via Google Fonts CSS URLs at runtime.
   No bundling, no custom URI schemes.

## Deliverables

| Spec File                      | PR  | Deliverable                                      |
| ------------------------------ | --- | ------------------------------------------------ |
| `01-shared-primitives.md`      | A   | Shared scene primitives + TET refactor           |
| `02-scene-descriptors.md`      | B   | Scene/theme descriptor schema + loader           |
| `03-procedural-generators.md`  | C   | Built-in procedural generator factories          |
| `04-precursors-scene.md`       | D   | Precursors scene (descriptor + generators)       |
| `05-descent-scene.md`          | E   | Descent scene (descriptor + generators)          |
| `06-gltf-runtime-loading.md`   | F   | glTF runtime scene loading integration           |
| `07-font-loading.md`           | G   | CSS font loading from Google Fonts               |
| `08-picking-unification.md`    | H   | Two-phase picking unification                    |
| `09-hud-transclusion.md`       | I   | HUD transclusion (stateless data-only fragments) |

## Stable Decisions (Already Implemented — Frozen)

New work must conform to these APIs:

| Interface                  | Location                             | Notes                                                                 |
| -------------------------- | ------------------------------------ | --------------------------------------------------------------------- |
| `SceneConfig` trait        | `src/scene/traits.rs`                | `spawn_environment(…) -> Entity`, `ambient_audio_track() -> &'static str` |
| `SceneManager` resource    | `src/scene/manager.rs`               | `register`, `activate`, `deactivate`, `current_name`                  |
| `SceneRoot` component      | `src/scene/manager.rs`               | Tags single root entity; recursive despawn                            |
| `SceneAudioState` resource | `src/scene/audio_bridge.rs`          | `request_ambient(&'static str)`, `request_stop()`                    |
| `ThemeManager` resource    | `src/theme/theme.rs`                 | `current() -> &Theme`; `generation: u64` dirty-check                  |
| `Theme` struct             | `src/theme/theme.rs`                 | UI palette + world-colour fields                                      |
| Audio constants            | `src/constants.rs`                   | `SCENE_AUDIO_FADE_IN_SECS`, `SCENE_AUDIO_FADE_OUT_SECS`             |
| Test helper pattern        | `tests/integration/scene_manager.rs` | `build_scene_mgr_app()` + nested `resource_scope`                    |

## Non-Negotiable Constraints (AGENTS.md)

- Files < 400 LOC, functions < 50 LOC.
- Zero-copy buffers; `&'static str` for audio track paths.
- No unit tests, no mocks — integration tests using real Bevy `App`.
- All constants in `constants.rs`.
- Dynamic tracing/debug logs in all methods.
- Very low coupling — every module reusable in isolation.
- Data/presentation separation; strong typing for domain models.
- ECS best practices; minimize entity lookups/iterations.

## Merge Order & Dependencies

```
PR A (primitives + TET refactor) — no deps, merge first
  |
PR B (descriptors) — depends on A (uses primitives)
  |
PR C (generators) — depends on B (reads descriptors)
  |
  +-- PR D (Precursors) --+  depend on A+B+C
  +-- PR E (Descent)    --+  can be parallel
  |
PR F (glTF) — depends on B (uses descriptor loader)
PR G (fonts) — fully independent, can merge any time
PR H (picking) — independent of scenes, can merge after A
PR I (HUD transclusion) — independent, can merge after A
```

## Bevy 0.18.1 Capabilities Used

These are used **directly** (no wrappers):

- **glTF**: `asset_server.load("model.glb#Scene0")`, spawn `SceneRoot(handle)`
- **PBR**: Full metallic-roughness, normal/occlusion/emissive maps
- **Animation**: `AnimationPlayer`/`AnimationClip` from glTF
- **Audio**: `AudioPlayer`, `AudioSink`, looping, spatial
- **Picking**: `bevy_picking` 0.18.1 first-party
- **HDR/Tonemapping**: `EnvironmentMapLight`, `LightProbe`, tonemapping LUTs
- **Baked GI**: `LightmapPlugin`, `LightProbes` (production-ready)
- **Hot-reload**: `file_watcher` cargo feature for dev-mode

Ecosystem plugins (already in Cargo.toml):
- `bevy_egui = "0.39"` — egui integration
- `bevy_hanabi = "0.18"` — GPU compute particle system
- `bevy_tweening = "0.15"` — animation tweening
- `bevy_prototype_lyon = "0.16"` — 2D vector shapes

## Open Questions (None Blocking)

- Time-of-day mapping for Precursors sky: deferred.
- Seasonal palette changes: deferred.
- Settings UI for scene/theme switching: follow-up after scenes are complete.
- Parallax scroll fix (`PreviousCameraPosition` never updated): separate PR.
- Real-time GI (Solari): experimental only, not used.
