## ADDED Requirements

### Requirement: GeneratorRegistry invokes generators from scene descriptors
The system SHALL call each generator listed in a `SceneDescriptor` through the `GeneratorRegistry` when spawning a scene environment.

#### Scenario: Scene with generators loads
- **WHEN** `DescriptorSceneConfig::spawn_environment()` is called
- **THEN** it iterates over `descriptor.generators`
- **AND** for each generator, looks up the factory in `GeneratorRegistry`
- **AND** invokes the factory with the generator's parameters
- **AND** all spawned entities are parented to the scene root

### Requirement: Generator functions are registered for all scene content
The system SHALL provide generator functions for water, foliage, and NPC spawning registered in `GeneratorRegistry::with_builtins()`.

#### Scenario: Medieval scene includes vegetation
- **WHEN** a `.scene.ron` specifies a `vegetation` generator
- **THEN** `GeneratorRegistry` resolves it to the vegetation factory
- **AND** the factory spawns vegetation entities under the scene root

### Requirement: World plugins do not spawn independently
The system SHALL NOT have `WaterPlugin`, `FoliagePlugin`, or `NpcPlugin` spawn entities in their own `Startup` systems.

#### Scenario: Loading TET scene
- **WHEN** the TET scene is activated
- **THEN** no water plane, foliage, or NPCs appear
- **AND** only the generators listed in `tet.scene.ron` are invoked
