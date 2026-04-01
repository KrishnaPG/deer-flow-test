### `control_center`

Purpose:

- broad HOTL/HITL operational awareness with bounded command, attention, and
  inspection loops

Required views:

- `GlobalVitalsView`
- `MissionRailView`
- `AttentionQueueView`
- `CommandConsoleView`
- `InspectorTabsView`
- `CenterCanvasModeSwitcherView`

Optional views:

- `ActivityRailView`
- `PinnedContextStripView`
- `ArtifactShelfView`

Required panels:

- `GlobalVitalsPanel`
- `MissionRailPanel`
- `AttentionPanel`
- `CommandDeckPanel`
- `InspectorPanel`
- `CenterCanvasPanel`

Required layout traits:

- `persistent_operational_frame`
- `persistent_edges`
- `dominant_center_canvas`
- `attention_without_takeover`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `mode_switchable_center`

World projection requirement:

- `optional`

Degradation rule:

- may degrade from cinematic world-bearing shell to atmospheric background shell,
  but may not lose command, attention, or inspect loops

Required linked dashboard interactions:

- `selection`
- `focus`
- `filtering`
- `pinning`
- `drill_down`
- `action_intent_emission`
- `command_target`

Required canonical record families:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `RuntimeStatusRecord`
- `ClarificationRecord`
- `IntentRecord`

Required metadata:

- `Required Metadata Baseline`
- target scope and policy visibility
- drill-down targets and lifecycle state

Required mediated reads:

- live-activity stream payload
- historical snapshot payload

Required intents:

- `operator intent`
- `intervention intent`
- `approval/denial intent`, where approval surfaces are present

Required broker ownership map:

- `selection`: `MissionRailPanel`
- `focus`: `InspectorPanel`
- `filtering`: `CenterCanvasPanel`
- `pinning`: `CenterCanvasPanel`
- `drill_down`: `InspectorPanel`
- `action_intent_emission`: `CommandDeckPanel`
- `command_target`: `CommandDeckPanel`

