# Design: Modular Toolkit - Library Decision Matrix

**Date:** 2026-03-31
**Status:** Approved

| Stage | Preferred Library / Reuse | Why | Avoid Replacing Unless |
| --- | --- | --- | --- |
| transport/client | existing DeerFlow bridge + Python client | already integrated, low protocol risk | hard capability gap or measured performance issue |
| contracts/serde | `serde`, `serde_json`, `chrono`, `uuid` | mature, standard Rust fit | schema export needs force additions |
| graph core | `petgraph` | mature graph primitives | missing required graph capability |
| layout/docking | `egui_dock` | battle-tested egui docking | missing core docking behavior |
| 3D/spatial | existing Bevy stack | already in repo and aligned | measured inadequacy |
| snapshots | `insta`, `similar-asserts` | stable snapshot diff tooling | snapshot instability forces narrower scope |

## Generator Boundary Rule

Reusing the DeerFlow bridge must not leak DeerFlow-specific transport assumptions
into canonical contracts above the raw envelope layer.
