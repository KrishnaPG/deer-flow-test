## ADDED Requirements

### Requirement: Hybrid Camera Mode System
The system SHALL support multiple camera modes with smooth transitions between them.

#### Scenario: Mode enumeration
- **WHEN** camera mode is requested
- **THEN** system SHALL return one of: FirstPerson, ThirdPerson, Orbital, Cinematic

#### Scenario: Tab key mode toggle
- **WHEN** user presses Tab key
- **THEN** system SHALL cycle through available camera modes
- **AND** transition SHALL complete within 0.5 seconds

#### Scenario: Smooth transition
- **WHEN** camera mode changes
- **THEN** system SHALL lerp camera position and rotation over 0.3-0.5 seconds
- **AND** input SHALL be disabled during transition

### Requirement: FirstPerson Camera Mode
The system SHALL provide first-person exploration with WASD movement and mouse look.

#### Scenario: WASD movement
- **WHEN** user presses W/A/S/D keys
- **THEN** camera SHALL move forward/left/backward/right relative to view direction
- **AND** movement speed SHALL be configurable (default: 5 m/s)

#### Scenario: Mouse look
- **WHEN** user moves mouse
- **THEN** camera yaw SHALL rotate based on horizontal mouse delta
- **AND** camera pitch SHALL rotate based on vertical mouse delta
- **AND** pitch SHALL be clamped to [-89, 89] degrees

#### Scenario: Eye height
- **WHEN** FirstPerson mode is active
- **THEN** camera position SHALL be at configured eye height (default: 1.7 meters)

### Requirement: ThirdPerson Camera Mode
The system SHALL provide third-person camera that follows a target entity.

#### Scenario: Camera positioning
- **WHEN** ThirdPerson mode is active
- **THEN** camera SHALL be positioned behind and above target entity
- **AND** distance SHALL be configurable (default: 3 meters)
- **AND** height offset SHALL be configurable (default: 1.5 meters)

#### Scenario: Camera rotation
- **WHEN** user moves mouse
- **THEN** camera SHALL orbit around target entity
- **AND** target entity SHALL rotate to face movement direction

### Requirement: Orbital Camera Mode
The system SHALL provide classic RTS-style orbital camera.

#### Scenario: Orbital controls
- **WHEN** Orbital mode is active
- **THEN** right-click drag SHALL rotate camera around focus point
- **AND** scroll wheel SHALL adjust zoom level
- **AND** middle-click drag SHALL pan camera

#### Scenario: Zoom limits
- **WHEN** user scrolls
- **THEN** zoom SHALL be clamped between min (0.25x) and max (4x) values

### Requirement: Cinematic Camera Mode
The system SHALL support waypoint-based cinematic sequences.

#### Scenario: Waypoint definition
- **WHEN** cinematic sequence is defined
- **THEN** system SHALL accept list of camera keyframes with position, rotation, duration

#### Scenario: Sequence playback
- **WHEN** cinematic playback starts
- **THEN** camera SHALL follow keyframes with smooth interpolation
- **AND** sequence SHALL be skippable with any input
