# PR F — glTF Runtime Scene Loading

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** PR B

---

## Problem

Scenes need to support externally authored 3D content (Blender-exported
glTF/GLB files) in addition to procedurally generated content. Bevy has
native glTF support — this PR wires it into the descriptor system.

## Design Principles

- Use Bevy's native `bevy_gltf` directly. No wrapper.
- `asset_server.load("path.glb#Scene0")` returns a `Handle<Scene>`.
- Spawn via `SceneRoot(handle)` — Bevy handles hierarchy, PBR materials,
  animations, and lights automatically.
- FBX is NOT natively supported by Bevy. Offline conversion via FBX2glTF
  or Blender headless is the supported path.

## Descriptor Integration

The `SceneDescriptor` has an optional `gltf_scene` field. When present,
the loader spawns the glTF scene as the base, then overlays procedural
generators on top.

```rust
// In DescriptorSceneConfig::spawn_environment:
if let Some(gltf_path) = &self.descriptor.gltf_scene {
    let scene_handle = asset_server.load(format!("{gltf_path}#Scene0"));
    commands.spawn((
        ChildOf(root),
        SceneRoot(scene_handle),
    ));
}
// Then iterate generators as usual...
```

## glTF Subscene Generator

The `GltfSubscene` generator param allows loading additional glTF
fragments within a procedurally generated scene:

```rust
GeneratorParams::GltfSubscene {
    path: "models/watchtower.glb".to_string(),
    transform: Some([0.0, 10.0, 0.0]),
    scale: Some(2.0),
}
```

This is handled by `gen_gltf_subscene()` in
`src/scene/generators/gltf_subscene.rs`.

## Example Descriptor

```ron
SceneDescriptor(
    name: "CustomLevel",
    ambient_audio: "audio/custom_ambient.ogg",
    gltf_scene: Some("scenes/custom_level.glb"),
    theme: "TET Orchestrator",
    generators: [
        (generator: "starfield", params: Starfield(
            count: 500,
            radius: 400.0,
            emissive: (1.5, 1.5, 2.0, 1.0),
        )),
    ],
)
```

## Asset Workflow

1. Author scene in Blender.
2. Export as `.glb` (binary glTF).
3. Place in `assets/scenes/` or `assets/models/`.
4. Reference from scene descriptor's `gltf_scene` or via `GltfSubscene`
   generator.

For FBX source files:
1. Convert offline: `FBX2glTF -i model.fbx -o model.glb`
2. Or: Blender headless `blender --background --python convert.py`
3. Commit the `.glb` output.

## Tests

Extend `tests/integration/scene_descriptors.rs`:

| Test                                      | Verifies                                           |
| ----------------------------------------- | -------------------------------------------------- |
| `t_gltf_01_descriptor_with_gltf_parses`  | Descriptor with `gltf_scene: Some(…)` deserializes |
| `t_gltf_02_gltf_subscene_generator_runs` | `gen_gltf_subscene` spawns a `SceneRoot` child     |

Note: Full glTF rendering tests require a render backend and are run
as manual smoke tests, not in CI headless mode.
