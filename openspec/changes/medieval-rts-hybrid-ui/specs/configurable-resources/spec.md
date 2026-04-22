## ADDED Requirements

### Modular Architecture

The resource display system SHALL be implemented as a reusable `ResourceDisplayPlugin`:
- Plugin SHALL be extractable to `crates/resource-display` for use in any application
- Resource configuration SHALL be data-driven (support custom resource types)
- Plugin SHALL integrate with `FactionThemePlugin` for icon tinting
- Public API SHALL support: `set_mode()`, `update_resource()`, `set_threshold()`
- Plugin SHALL include examples for external reuse

### Requirement: Resource Mode Selection
The system SHALL support multiple resource display modes.

#### Scenario: Mode enumeration
- **WHEN** resource mode is requested
- **THEN** system SHALL return one of: Traditional, AgentMetrics, Hybrid

#### Scenario: Mode persistence
- **WHEN** user changes resource mode
- **THEN** system SHALL save selection to user preferences
- **AND** restore mode on application restart

### Requirement: Traditional Resources
The system SHALL display classic RTS resources (Food, Wood, Stone, Gold, Iron).

#### Scenario: Resource icons
- **WHEN** Traditional mode is active
- **THEN** system SHALL display wheat icon for Food
- **AND** display log icon for Wood
- **AND** display stone icon for Stone
- **AND** display coin icon for Gold
- **AND** display ingot icon for Iron

#### Scenario: Resource formatting
- **WHEN** resource count is displayed
- **THEN** system SHALL use monospace font
- **AND** format large numbers with K suffix (1000 → 1K)
- **AND** format millions with M suffix (1000000 → 1M)

#### Scenario: Resource production
- **WHEN** resource is being produced
- **THEN** system SHALL show production rate (+10/s)
- **AND** use green color for positive production
- **AND** use red color for negative production (consumption)

### Requirement: Agent Metrics Resources
The system SHALL display agent system metrics.

#### Scenario: Metric icons
- **WHEN** AgentMetrics mode is active
- **AND** system SHALL display token icon for Tokens/sec
- **AND** display model icon for Active Models
- **AND** display agent icon for Active Agents
- **AND** display dollar icon for Cost/hour

#### Scenario: Metric formatting
- **WHEN** metric value is displayed
- **THEN** system SHALL format tokens with K suffix
- **AND** format cost with dollar sign and 2 decimal places
- **AND** show percentage for utilization metrics

#### Scenario: Metric trends
- **WHEN** metric changes
- **THEN** system SHALL show trend arrow (up/down/stable)
- **AND** animate value change with data pulse effect
- **AND** color code based on thresholds (green=good, yellow=warning, red=critical)

### Requirement: Hybrid Resources
The system SHALL display both traditional and agent resources.

#### Scenario: Hybrid layout
- **WHEN** Hybrid mode is active
- **THEN** system SHALL display traditional resources on left side
- **AND** display agent metrics on right side
- **AND** show divider between sections

#### Scenario: Section collapsing
- **WHEN** Hybrid mode is active
- **THEN** system SHALL allow collapsing traditional section
- **AND** allow collapsing agent metrics section
- **AND** persist collapse state to preferences

### Requirement: Resource Thresholds
The system SHALL support configurable resource thresholds.

#### Scenario: Warning threshold
- **WHEN** resource falls below warning threshold
- **THEN** system SHALL highlight resource icon with warning color
- **AND** show warning indicator (exclamation mark)

#### Scenario: Critical threshold
- **WHEN** resource falls below critical threshold
- **THEN** system SHALL highlight resource icon with error color
- **AND** flash resource indicator
- **AND** play alert sound (optional)

#### Scenario: Threshold configuration
- **WHEN** user opens resource settings
- **THEN** system SHALL allow setting warning and critical thresholds per resource
- **AND** validate thresholds (critical < warning)
- **AND** persist thresholds to preferences
