## ADDED Requirements

### Modular Architecture

All systems in this spec SHALL be implemented as modular, reusable components:
- Time of day system SHALL use `bevy_skybox` (0.7.0) + `bevy_easings` (0.18.0) wrapped in `DayNightPlugin`
- Weather system SHALL use `bevy_hanabi` (0.18.0) + `bevy_exponential_height_fog` (0.1.0) wrapped in `WeatherPlugin`
- Season system SHALL integrate with `bevy_feronia` vegetation, wrapped in `SeasonPlugin`
- Each plugin SHALL expose configuration via Bevy resources
- Each plugin SHALL be reusable in external Bevy projects

### Requirement: Time of Day System
The system SHALL simulate day/night cycle with lighting transitions.

#### Scenario: Time progression
- **WHEN** time of day system is active
- **THEN** system SHALL advance game time based on real time
- **AND** time scale SHALL be configurable (default: 1 real minute = 1 game hour)

#### Scenario: Sun position
- **WHEN** time changes
- **THEN** system SHALL update sun position based on time
- **AND** sun SHALL rise at 06:00 and set at 18:00
- **AND** sun arc SHALL follow seasonal variation

#### Scenario: Lighting transitions
- **WHEN** time transitions between day and night
- **THEN** system SHALL smoothly interpolate directional light color
- **AND** adjust ambient light brightness
- **AND** update shadow direction and softness

#### Scenario: Sky rendering
- **WHEN** time of day changes
- **THEN** system SHALL update sky gradient (blue day, orange sunset, dark night)
- **AND** show stars during night
- **AND** render moon during night hours

### Requirement: Weather System
The system SHALL simulate weather conditions affecting visuals.

#### Scenario: Weather states
- **WHEN** weather system is active
- **THEN** system SHALL support states: Clear, Cloudy, Rainy, Foggy, Stormy

#### Scenario: Weather transitions
- **WHEN** weather changes
- **THEN** system SHALL smoothly transition over 30-60 seconds
- **AND** interpolate particle density and visibility
- **AND** adjust ambient lighting

#### Scenario: Rain weather
- **WHEN** weather is Rainy
- **THEN** system SHALL render rain particles
- **AND** darken ambient lighting
- **AND** add puddle reflections to terrain
- **AND** play rain ambient sound

#### Scenario: Fog weather
- **WHEN** weather is Foggy
- **THEN** system SHALL reduce draw distance
- **AND** add fog color to atmosphere
- **AND** reduce shadow intensity
- **AND** play fog ambient sound (optional)

#### Scenario: Storm weather
- **WHEN** weather is Stormy
- **THEN** system SHALL combine rain and fog effects
- **AND** add lightning flashes (random interval)
- **AND** play thunder sounds with delay based on lightning distance
- **AND** darken scene significantly

### Requirement: Atmospheric Effects
The system SHALL render atmospheric visual effects.

#### Scenario: Volumetric fog
- **WHEN** fog is present
- **THEN** system SHALL render volumetric fog with god rays
- **AND** fog density SHALL vary with altitude (denser in valleys)
- **AND** fog color SHALL be affected by time of day

#### Scenario: Dust particles
- **WHEN** weather is Clear and terrain is dry
- **THEN** system SHALL render floating dust particles
- **AND** particles SHALL be affected by wind direction
- **AND** density SHALL increase near roads and paths

#### Scenario: Leaf particles
- **WHEN** vegetation is present and wind is active
- **THEN** system SHALL render falling leaves
- **AND** leaves SHALL follow wind currents
- **AND** leaf color SHALL reflect season (green, yellow, brown)

### Requirement: Season System
The system SHALL support seasonal visual changes.

#### Scenario: Season states
- **WHEN** season system is active
- **THEN** system SHALL support: Spring, Summer, Autumn, Winter

#### Scenario: Vegetation seasons
- **WHEN** season changes
- **THEN** system SHALL update vegetation colors
- **AND** Spring: fresh green, flowers blooming
- **AND** Summer: mature green, full foliage
- **AND** Autumn: yellow/orange/brown, falling leaves
- **AND** Winter: bare trees, snow cover

#### Scenario: Terrain seasons
- **WHEN** season changes
- **THEN** system SHALL update terrain texture weights
- **AND** Spring: lush grass, flowers
- **AND** Summer: dry grass, full growth
- **AND** Autumn: brown grass, fallen leaves
- **AND** Winter: snow overlay, frozen water

### Requirement: Weather Particles (bevy_hanabi)
The system SHALL use bevy_hanabi for weather particle effects.

#### Scenario: Rain particles
- **WHEN** rain effect is needed
- **THEN** system SHALL create particle emitter with rain texture
- **AND** set spawn rate based on intensity
- **AND** apply gravity and wind to particles
- **AND** despawn particles on ground collision

#### Scenario: Snow particles
- **WHEN** winter snow is active
- **THEN** system SHALL create particle emitter with snow texture
- **AND** set slower fall speed than rain
- **AND** apply wind drift to particles
- **AND** accumulate on terrain surface (optional)
