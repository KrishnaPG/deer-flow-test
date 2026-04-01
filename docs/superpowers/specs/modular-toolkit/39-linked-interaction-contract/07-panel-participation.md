## Panel Participation Model

Panels participate in linked interaction through explicit roles.

### Roles

- `source`
  - originates linked state changes from operator action
- `sink`
  - receives linked state and re-renders, highlights, filters, or scopes detail
- `broker`
  - owns shell-level coordination for a given interaction type and republishes it
- `mirror`
  - reflects brokered state for orientation or continuity without becoming the
    authority

Each interaction type must have one clear broker.

### Canonical Panel Participation

| Panel | Default roles | Minimum participation |
| --- | --- | --- |
| `CenterCanvasPanel` | `source`, `sink`, optional `broker` | must originate and receive selection, focus, replay cursor, and pinned context where the shell mode uses them |
| `MissionRailPanel` | `source`, `sink` | must originate mission/session/run focus and reflect active shell filters and focus |
| `InspectorPanel` | `sink`, selective `source` | must resolve current shell selection into detail and actions, and may originate secondary drill-downs |
| `AttentionPanel` | `source`, `sink` | must push actionable interrupts into command, center, and inspector flows |
| `CommandDeckPanel` | `source`, `sink`, optional `broker` | must consume target scope and pinned context, and originate mediated intents |
| `ReplayPanel` | `source`, `sink` | must originate replay cursor or range and reflect externally selected checkpoints or ranges |
| `ArtifactShelfPanel` | `source`, `sink`, `mirror` | must originate artifact selection and pinning while preserving lineage and representation identity |
| `WorldViewportPanel` | `source`, `sink` | must originate world selection, focus, and viewport navigation and must consume brokered camera, temporal, and command-target context without silently arming commands |
| `MinimapPanel` | `source`, `sink`, `mirror` | must reflect current viewport and may originate viewport navigation and explicit locate or follow focus without taking command authority |
| `EventRailPanel` | `source`, `sink` | must originate and receive `replay_cursor`, `temporal_range`, and event-linked selection while preserving live-tail versus historical state |
| `FleetRailPanel` | `source`, `sink` | must originate hierarchy-driven selection/focus and reflect current command-target and policy-invalidated state |
| `BottomDeckPanel` | `sink`, selective `source` | must consume brokered selection, command-target, queue, and policy state, and may originate explicit command-surface actions through declared command views only |

