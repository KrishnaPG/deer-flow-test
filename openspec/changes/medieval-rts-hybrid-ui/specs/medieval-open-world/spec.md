## ADDED Requirements

### Modular Architecture

All systems in this spec SHALL be implemented as modular, reusable components:
- Terrain system SHALL be extractable to `crates/terrain/`
- Vegetation system SHALL use `bevy_feronia` (0.8.2) wrapped in a reusable `FoliagePlugin`
- Water system SHALL use `bevy_water` (0.18.1) wrapped in a reusable `WaterPlugin`
- NPC system SHALL use `bevior_tree` (0.10.0) wrapped in a reusable `NpcPlugin`
- Each plugin SHALL expose a clean public API with configuration via Bevy resources
- Each plugin SHALL include documentation for reuse in external projects

### Requirement: Heightmap Terrain System
The system SHALL render terrain from heightmap data with texture splatting.

#### Scenario: Terrain generation
- **WHEN** scene descriptor specifies HeightmapTerrain generator
- **THEN** system SHALL load heightmap PNG (16-bit grayscale)
- **AND** generate mesh with configurable resolution
- **AND** apply height scale factor (default: 100 meters)

#### Scenario: Texture splatting
- **WHEN** terrain mesh is generated
- **THEN** system SHALL apply multiple terrain textures (grass, dirt, rock, snow)
- **AND** blend textures using splat mask (RGBA channels map to 4 textures)
- **AND** support UV scaling for tile repetition

#### Scenario: Terrain LOD
- **WHEN** terrain chunks are far from camera
- **THEN** system SHALL use lower resolution meshes
- **AND** transition between LOD levels SHALL be smooth (no popping)

### Requirement: Procedural Vegetation System
The system SHALL place vegetation instances procedurally with wind animation.

#### Scenario: Vegetation placement
- **WHEN** scene descriptor specifies ProceduralVegetation generator
- **THEN** system SHALL place trees/bushes based on density parameter
- **AND** placement SHALL respect biome filter (exclude water, rock biomes)
- **AND** placement SHALL use random offset within variation radius

#### Scenario: Vegetation instancing
- **WHEN** vegetation is rendered
- **THEN** system SHALL use GPU instancing for performance
- **AND** instances SHALL share mesh and material
- **AND** instance count SHALL support minimum 10,000 trees

#### Scenario: Wind animation
- **WHEN** wind_affected is true
- **THEN** vegetation SHALL sway based on wind direction and strength
- **AND** animation SHALL use vertex shader (not CPU-driven)
- **AND** sway amount SHALL vary by vegetation height

### Requirement: Building Placement System
The system SHALL place glTF building models with weathering variants.

#### Scenario: Building loading
- **WHEN** scene descriptor specifies BuildingCluster generator
- **THEN** system SHALL load glTF models from specified paths
- **AND** apply faction colors to building materials
- **AND** set weathering level (pristine, weathered, damaged, ruined)

#### Scenario: Building layout
- **WHEN** building cluster is generated
- **THEN** system SHALL arrange buildings according to layout type (village, castle, farm)
- **AND** buildings SHALL have collision shapes for player interaction

### Requirement: NPC Population System
The system SHALL spawn NPC entities with basic behaviors.

#### Scenario: NPC spawning
- **WHEN** scene descriptor specifies NPCPopulation generator
- **THEN** system SHALL spawn NPC count at random positions within radius
- **AND** NPCs SHALL have skeletal mesh with animation controller

#### Scenario: NPC behavior
- **WHEN** NPC is spawned with behavior type
- **THEN** idle NPCs SHALL play idle animation
- **AND** wandering NPCs SHALL follow random waypoints
- **AND** working NPCs SHALL play work animation loop

### Requirement: Water Plane System
The system SHALL render animated water surfaces.

#### Scenario: Water rendering
- **WHEN** scene descriptor specifies WaterPlane generator
- **THEN** system SHALL render water plane at specified level
- **AND** apply water shader with wave animation
- **AND** support flow direction for rivers

#### Scenario: Water interactions
- **WHEN** player touches water surface
- **THEN** system SHALL apply swimming physics (reduced speed, different camera)
- **AND** trigger splash particle effect
