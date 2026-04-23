## ADDED Requirements

### Requirement: Scene teardown recursively despawns all entities
The system SHALL remove all entities in a scene hierarchy when deactivating a scene.

#### Scenario: Switching from TET to Medieval
- **WHEN** `SceneManager::deactivate()` is called on the active TET scene
- **THEN** the scene root entity and all its children are despawned
- **AND** no TET-specific entities (monolith, starfield, trails) remain in the world

#### Scenario: Switching from Medieval to Descent
- **WHEN** `SceneManager::deactivate()` is called on the active Medieval scene
- **THEN** terrain mesh, vegetation instances, water plane, and NPCs are all removed
- **AND** the new Descent scene spawns in a clean world state

### Requirement: Scene activation checks for orphaned entities
The system SHALL log a warning if entities from a previous scene are detected after activation.

#### Scenario: Detecting entity leak
- **WHEN** a new scene is activated
- **AND** entities with tags from a different scene exist
- **THEN** the system logs a warning with entity count and scene name
