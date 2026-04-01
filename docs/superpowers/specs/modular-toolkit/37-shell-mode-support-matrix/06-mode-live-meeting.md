### `live_meeting`

Purpose:

- live HITL collaboration with transcript, pinned context, intervention, and
  artifact access in one shell

Required views:

- `SessionRunHeaderView`
- `TranscriptView`
- `IntentComposerView`
- `InspectorTabsView`

Optional views:

- `PinnedContextStripView`
- `ArtifactShelfView`
- `ArtifactAccessView`
- `InterventionGateView`

Required panels:

- `CenterCanvasPanel`
- `CommandDeckPanel`
- `InspectorPanel`

Required layout traits:

- `dominant_center_canvas`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `deep_inspection_escalation`

World projection requirement:

- `not_applicable`

Degradation rule:

- transcript-only fallback is allowed only if intent submission and detail
  inspection remain available

Required linked dashboard interactions:

- `selection`
- `focus`
- `pinning`
- `drill_down`
- `action_intent_emission`
- `command_target`

Required canonical record families:

- `RunRecord`
- `SessionRecord`
- `MessageRecord`
- `ClarificationRecord`
- `ArtifactRecord`
- `IntentRecord`

Required metadata:

- `Required Metadata Baseline`
- speaker/source identity and ordering
- target scope, attachment identity, and policy visibility

Required mediated reads:

- live-activity stream payload
- artifact pointer payload, where artifact access is exposed

Required intents:

- `operator intent`
- `clarification response`
- `intervention intent`, where intervention surfaces are present

Required broker ownership map:

- `selection`: `CenterCanvasPanel`
- `focus`: `InspectorPanel`
- `pinning`: `CenterCanvasPanel`
- `drill_down`: `InspectorPanel`
- `action_intent_emission`: `CommandDeckPanel`
- `command_target`: `CommandDeckPanel`

