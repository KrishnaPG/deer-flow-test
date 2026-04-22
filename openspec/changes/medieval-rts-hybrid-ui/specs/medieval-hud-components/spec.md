## ADDED Requirements

### Modular Architecture

All HUD components SHALL be implemented as reusable egui widgets:
- Widgets SHALL be extractable to `crates/medieval-widgets` for use in any egui application
- Each widget SHALL accept a `MedievalStyle` struct for consistent theming
- Widgets SHALL integrate with `FactionThemePlugin` for dynamic color application
- Public API SHALL follow egui conventions (Builder pattern, `ui.horizontal(|ui| ...)` )
- Each widget SHALL include usage examples in documentation

### Requirement: Medieval Resource Bar
The system SHALL display resources in a medieval-themed top bar.

#### Scenario: Resource display layout
- **WHEN** medieval HUD is active
- **THEN** system SHALL render resource icons and counts across top bar
- **AND** icons SHALL use faction-tinted medieval style (wheat, wood, stone, gold)
- **AND** counts SHALL use monospace font for alignment

#### Scenario: Resource alerts
- **WHEN** resource falls below threshold
- **THEN** system SHALL highlight resource icon with warning color
- **AND** display alert banner in center of resource bar

#### Scenario: Population display
- **WHEN** population is shown
- **THEN** system SHALL display population count with icon
- **AND** show population limit with progress bar

### Requirement: Medieval Command Grid
The system SHALL provide 5x3 command grid for RTS actions.

#### Scenario: Grid layout
- **WHEN** command console is active
- **THEN** system SHALL display 5x3 grid of command buttons
- **AND** each button SHALL have icon and hotkey label
- **AND** bottom-right slot SHALL be reserved for Cancel

#### Scenario: Button states
- **WHEN** command button is rendered
- **THEN** system SHALL show enabled state with faction border style
- **AND** show disabled state with greyed out appearance
- **AND** show hover state with faction highlight color
- **AND** show active/pressed state with depressed appearance

#### Scenario: Command categories
- **WHEN** user switches command category
- **THEN** system SHALL display Build commands (buildings)
- **AND** display Train commands (units)
- **AND** display Research commands (technologies)
- **AND** display Agent commands (agent system modes)

### Requirement: Medieval Selection Panel
The system SHALL display selected entity information in medieval style.

#### Scenario: Unit selection
- **WHEN** unit is selected
- **THEN** system SHALL show unit portrait in selection panel
- **AND** display unit name with faction-appropriate styling
- **AND** show health ring with animated fill

#### Scenario: Unit stats
- **WHEN** unit is selected
- **THEN** system SHALL display stats grid (attack, defense, speed, etc.)
- **AND** use parchment-style background for stat area
- **AND** show ability icons with cooldown overlays

#### Scenario: Building selection
- **WHEN** building is selected
- **THEN** system SHALL show building portrait
- **AND** display building name and level
- **AND** show production queue if applicable

### Requirement: Medieval Minimap
The system SHALL render minimap with medieval styling.

#### Scenario: Minimap rendering
- **WHEN** minimap is displayed
- **THEN** system SHALL render terrain preview with biome colors
- **AND** show unit dots with faction colors
- **AND** show fog of war overlay for unexplored areas

#### Scenario: Minimap frame
- **WHEN** minimap is displayed
- **THEN** system SHALL use octagonal clip mask (not rectangular)
- **AND** apply faction border style to frame
- **AND** show faction heraldry in corner

#### Scenario: Minimap interaction
- **WHEN** user clicks minimap
- **THEN** system SHALL pan camera to clicked world position
- **AND** show camera frustum indicator

### Requirement: Medieval Event Feed
The system SHALL display scrolling event log in medieval style.

#### Scenario: Event entry rendering
- **WHEN** event is added to feed
- **THEN** system SHALL display timestamp with monospace font
- **AND** show severity icon (info, warning, error)
- **AND** show event message with parchment background

#### Scenario: Event colors
- **WHEN** event is rendered
- **THEN** system SHALL use faction secondary color for info events
- **AND** use warning color for warning events
- **AND** use error color for error events

#### Scenario: Event feed scrolling
- **WHEN** new events are added
- **THEN** system SHALL auto-scroll to bottom
- **AND** allow manual scroll with mouse wheel
- **AND** cap maximum entries at 200

### Requirement: Medieval Panel Styling
The system SHALL style HUD panels with medieval textures.

#### Scenario: Panel textures
- **WHEN** medieval HUD is active
- **THEN** system SHALL use stone texture for headers
- **AND** use wood texture for panel frames
- **AND** use parchment texture for content areas

#### Scenario: Panel borders
- **WHEN** panel is rendered
- **THEN** system SHALL apply faction border style
- **AND** show corner ornaments with faction symbol
- **AND** apply inner shadow for depth
