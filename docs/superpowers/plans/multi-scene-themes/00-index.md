# Multi-Scene Themes — Implementation Plan Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform deer_gui from a single hard-coded TET scene into a data-driven, multi-theme cinematic engine supporting dynamic scene descriptors, procedural generators, glTF loading, and CSS font resolution.

**Architecture:** Scene descriptors (RON files) define what to spawn; a generator registry resolves generator names to factory functions that create entities. Themes are also descriptor-driven. The existing `SceneConfig` trait + `SceneManager` are bridged via `DescriptorSceneConfig`. Bevy's native APIs are used directly — no wrappers.

**Tech Stack:** Rust, Bevy 0.18.1, bevy_egui 0.39, bevy_hanabi 0.18, serde + RON, ureq (HTTP), Google Fonts CSS

---

## Plan Files

Each PR has its own plan file with step-by-step instructions:

| Plan File                           | PR  | Description                              | Depends On |
| ----------------------------------- | --- | ---------------------------------------- | ---------- |
| `01-shared-primitives.md`           | A   | Shared primitives + TET refactor         | —          |
| `02-scene-descriptors.md`           | B   | Scene/theme descriptor schema + loader   | A          |
| `03-procedural-generators.md`       | C   | Generator factories + registry           | A, B       |
| `04-precursors-scene.md`            | D   | Precursors scene + theme                 | A, B, C    |
| `05-descent-scene.md`               | E   | Descent scene + theme                    | A, B, C    |
| `06-gltf-runtime-loading.md`        | F   | glTF runtime loading                     | B          |
| `07-font-loading.md`                | G   | CSS font loading from Google Fonts       | —          |
| `08-picking-unification.md`         | H   | Two-phase picking                        | —          |
| `09-hud-transclusion.md`            | I   | HUD transclusion                         | —          |

## Execution Order

**Phase 1 (sequential):** A → B → C (each builds on the previous)

**Phase 2 (parallel):** D, E, F, G, H, I (all independent after Phase 1)

## Verification

After each PR's tasks are complete, run:
```bash
cd apps/deer_gui && cargo test 2>&1
cd apps/deer_gui && cargo clippy -- -D warnings 2>&1
```

All 139+ existing tests must remain green. New tests must pass.
