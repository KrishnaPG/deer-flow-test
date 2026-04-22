## 1. Hybrid Camera System

- [x] 1.1 Add CameraMode enum to CinematicCamera component
- [x] 1.2 Implement FirstPerson mode with WASD movement
- [x] 1.3 Implement mouse look with pitch/yaw rotation
- [x] 1.4 Implement ThirdPerson mode with orbit controls
- [x] 1.5 Implement Orbital mode (existing functionality refactor)
- [x] 1.6 Add Tab key mode toggle with smooth lerp transition
- [ ] 1.7 Add camera mode settings to user preferences
- [x] 1.8 Write unit tests for camera mode switching

## 2. Medieval Terrain System

- [x] 2.1 Create HeightmapTerrain generator in scene/generators/
- [x] 2.2 Implement heightmap mesh generation from PNG
- [x] 2.3 Add terrain material with texture splatting (basic)
- [x] 2.6 Implement basic LOD system for terrain chunks
- [ ] 2.4 Create terrain texture assets (grass, dirt, rock, snow)
- [ ] 2.5 Create splat mask texture format
- [ ] 2.7 Add terrain to scene descriptor RON format
- [ ] 2.8 Write integration tests for terrain loading

## 3. Procedural Vegetation

- [x] 3.1 Create ProceduralVegetation generator
- [x] 3.2 Implement vegetation placement with density control
- [x] 3.3 Add biome filter for vegetation placement
- [ ] 3.4 Implement GPU instancing for vegetation rendering
- [ ] 3.5 Create glTF tree models (oak, pine, birch)
- [ ] 3.6 Create glTF bush and grass models
- [ ] 3.7 Implement wind animation shader for vegetation
- [ ] 3.8 Add vegetation to scene descriptor

## 4. Building Placement System

- [ ] 4.1 Create BuildingCluster generator
- [ ] 4.2 Implement building placement with layout presets
- [ ] 4.3 Create glTF medieval building models (house, tower, church)
- [ ] 4.4 Add weathering level support (pristine to ruined)
- [ ] 4.5 Implement faction color application to buildings
- [ ] 4.6 Add collision shapes to buildings
- [ ] 4.7 Create building cluster layout presets (village, castle, farm)
- [ ] 4.8 Write tests for building placement

## 5. NPC Population System

- [ ] 5.1 Create NPCPopulation generator
- [ ] 5.2 Implement NPC spawning with count and radius
- [ ] 5.3 Create glTF character models (knight, peasant)
- [ ] 5.4 Create glTF horse model
- [ ] 5.5 Implement skeletal animation controller
- [ ] 5.6 Add idle, walk, work animation states
- [ ] 5.7 Implement simple waypoint navigation
- [ ] 5.8 Add NPC behavior types (idle, wander, work)

## 6. Water Plane System

- [x] 6.1 Create WaterPlane generator
- [x] 6.2 Implement water plane mesh at configured level
- [x] 6.3 Create water shader with wave animation (core algorithms)
- [x] 6.4 Add flow direction for rivers
- [ ] 6.5 Implement water collision for swimming
- [ ] 6.6 Add splash particle effect on water contact
- [ ] 6.7 Write tests for water plane rendering

## 7. Faction Theme Engine

- [ ] 7.1 Create FactionTheme struct with colors and style
- [ ] 7.2 Add FactionId enum (English, French, Byzantine, Mongol)
- [ ] 7.3 Implement faction preset definitions
- [ ] 7.4 Add faction transition system to ThemeManager
- [ ] 7.5 Implement color interpolation with purple midpoint
- [ ] 7.6 Implement border style morphing
- [ ] 7.7 Implement heraldry crossfade
- [ ] 7.8 Create faction symbol assets
- [ ] 7.9 Add faction selector UI to settings
- [ ] 7.10 Write tests for faction transitions

## 8. Medieval HUD Components

- [ ] 8.1 Create medieval resource bar component
- [ ] 8.2 Create 5x3 command grid component
- [ ] 8.3 Create selection panel component
- [ ] 8.4 Create minimap component with octagonal frame
- [ ] 8.5 Create event feed component
- [ ] 8.6 Create stone/wood/parchment texture assets
- [ ] 8.7 Create Gothic font assets
- [ ] 8.8 Implement faction color application to HUD
- [ ] 8.9 Write component integration tests

## 9. Configurable Resources

- [ ] 9.1 Create ResourceMode enum (Traditional, AgentMetrics, Hybrid)
- [ ] 9.2 Implement Traditional resource display
- [ ] 9.3 Implement AgentMetrics resource display
- [ ] 9.4 Implement Hybrid layout with collapsible sections
- [ ] 9.5 Create resource icon assets (wheat, log, stone, coin, ingot)
- [ ] 9.6 Add resource threshold configuration
- [ ] 9.7 Add resource mode to user preferences
- [ ] 9.8 Write tests for resource display modes

## 10. Time of Day System

- [ ] 10.1 Create TimeOfDay resource and system
- [ ] 10.2 Implement time progression with configurable scale
- [ ] 10.3 Update sun position based on time
- [ ] 10.4 Implement directional light color interpolation
- [ ] 10.5 Update sky gradient for day/night cycle
- [ ] 10.6 Add star rendering for night sky
- [ ] 10.7 Add moon rendering for night hours
- [ ] 10.8 Write tests for time progression

## 11. Weather System

- [ ] 11.1 Create WeatherState enum (Clear, Cloudy, Rainy, Foggy, Stormy)
- [ ] 11.2 Create weather transition system
- [ ] 11.3 Implement rain particle effect (bevy_hanabi)
- [ ] 11.4 Implement fog effect with draw distance reduction
- [ ] 11.5 Implement storm effect (rain + lightning)
- [ ] 11.6 Add lightning flash and thunder sound
- [ ] 11.7 Create weather ambient audio assets
- [ ] 11.8 Write tests for weather transitions

## 12. Season System

- [ ] 12.1 Create SeasonState enum (Spring, Summer, Autumn, Winter)
- [ ] 12.2 Implement vegetation color changes per season
- [ ] 12.3 Implement terrain texture blending per season
- [ ] 12.4 Add falling leaves particle effect for Autumn
- [ ] 12.5 Add snow overlay for Winter
- [ ] 12.6 Create seasonal texture variants
- [ ] 12.7 Write tests for season transitions

## 13. Asset Pipeline

- [ ] 13.1 Create medieval scene descriptor RON format
- [ ] 13.2 Create example medieval_open.scene.ron
- [ ] 13.3 Create example tutorial_village.scene.ron
- [ ] 13.4 Set up glTF model export workflow
- [ ] 13.5 Create terrain texture atlas
- [ ] 13.6 Create audio assets (ambient, music, sfx)
- [ ] 13.7 Document asset naming conventions
- [ ] 13.8 Create asset loading error handling

## 14. Integration & Testing

- [ ] 14.1 Integrate all components in main.rs
- [ ] 14.2 Create medieval RTS demo scene
- [ ] 14.3 Test camera mode switching in demo
- [ ] 14.4 Test faction theme switching in demo
- [ ] 14.5 Test resource display modes in demo
- [ ] 14.6 Test weather transitions in demo
- [ ] 14.7 Performance profiling for open world
- [ ] 14.8 Create user documentation
