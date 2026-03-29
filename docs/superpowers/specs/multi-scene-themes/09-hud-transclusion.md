# PR I — HUD Transclusion (Stateless Data-Only Fragments)

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** None (independent of scenes)

---

## Problem

The Center Canvas (`center_canvas.rs`) has stub modes (SwarmMonitor,
ArtifactGraph, etc.). Scenes and subsystems need to push content fragments
into the canvas without directly calling HUD internals.

## Design Principles

- **Stateless**: Fragments are data-only definitions. The HUD reads and
  renders them. Fragments never own state.
- **Decoupled**: No scene imports from `hud`; fragments are pushed via
  a thin public API on the `HudFragmentRegistry` resource.

## Module: `src/hud/transclusion.rs` (< 100 LOC)

```rust
/// A single content fragment pushed by any subsystem into the Center Canvas.
#[derive(Debug, Clone)]
pub struct HudFragment {
    /// Unique provider ID (e.g. "scene.tet", "swarm.monitor").
    pub provider: &'static str,
    /// Display title shown in the canvas tab.
    pub title: String,
    /// Render callback, called each egui frame when this fragment is active.
    pub render: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>,
}

/// Resource holding all currently registered fragments.
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

## Integration

- `HudPlugin::build` calls `app.init_resource::<HudFragmentRegistry>()`.
- `center_canvas_system` reads `HudFragmentRegistry` and renders
  registered fragments as tabs alongside built-in modes.
- Purely additive — existing modes are untouched.

## Tests

New file: `tests/integration/hud_transclusion.rs`

| Test                                | Verifies                                  |
| ----------------------------------- | ----------------------------------------- |
| `t_trans_01_registry_default_empty` | Registry starts with 0 fragments          |
| `t_trans_02_register_fragment`      | After register, `fragments().len() == 1`  |
| `t_trans_03_unregister_removes`     | After unregister, count drops to 0        |
| `t_trans_04_multiple_providers`     | Two different providers both present      |
