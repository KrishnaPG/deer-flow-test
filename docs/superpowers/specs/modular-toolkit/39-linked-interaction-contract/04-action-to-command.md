### `action_intent_emission`

Purpose:

- emit a canonical action candidate from any qualifying hosted view into mediated
  command flow

Required basis:

- actor and session context
- target IDs
- intent kind
- parameter references
- policy visibility
- consequence or validation state where relevant

Allowed sources:

- `ActionDeckView`
- `InspectorTabsView`
- `AttentionQueueView`
- `ArtifactAccessView`
- `TranscriptView`
- `ReplayActivityView`
- `SearchRetrievalResultsView`
- `MissionRailView`

Required receivers where declared:

- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- intervention or approval surfaces

Degradation rule:

- may degrade to shell-local `prefill` only when the mediated intent path
  remains available

### `prefill_seed`

Purpose:

- create or refresh shell-local `prefill` intent context without granting
  editable ownership or submit authority

Allowed sources:

- any declared `action_intent_emission` source
- any declared `command_target` source
- any explicitly allowed linked-gesture seed source declared by the shell mode

Allowed effect:

- create `prefill`
- refresh `prefill`
- update suggested target, scope, parameter, or provenance context

Forbidden effect:

- create `draft` automatically
- enter `validated`
- enter `submitted`
- append any canonical `IntentRecord`

Boundary rule:

- `prefill_seed` remains shell-local only; explicit operator promotion is
  required for `prefill -> draft`

### `command_target`

Purpose:

- normalize the current actionable target bundle for mediated command flow

Required basis:

- actor and session context
- canonical target refs
- scope
- provenance
- policy visibility
- broker sequence or epoch

Allowed sources:

- selection-capable views
- `InspectorTabsView`
- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- `ReplayActivityView`
- world picks

Required receivers where declared:

- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- intervention or approval surfaces

Degradation rule:

- may degrade to shell-local `prefill` context only when explicit operator
  promotion into editable `draft` remains required
- declared `command_target` sources may seed or refresh shell-local `prefill`,
  but may not advance beyond `prefill` without explicit operator promotion

