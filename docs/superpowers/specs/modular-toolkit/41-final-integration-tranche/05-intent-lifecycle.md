## Intent Lifecycle Boundaries And Broker Ownership

External write families such as operator intents, clarification responses,
approval or denial submissions, and intervention intents all follow the same
boundary: shell-local before `submitted`, canonical only at explicit
submission. Approval and denial controls may submit a decision against an
existing intent, but they do not locally force that intent into `approved` or
`rejected`.

Intent state remains shell-local and non-canonical through:

- `prefill`
- `draft`
- `validated`

Only `submitted` may append an `IntentRecord`.

`approved`, `executed`, and `rejected` are later observed mediated outcomes.

### Lifecycle Entry Rules

`prefill_seed` is satisfied by any declared `action_intent_emission` source,
any declared `command_target` source, and any explicitly allowed linked-gesture
seed source declared by the shell mode.

| Stage | May be entered by | Boundary |
| --- | --- | --- |
| `prefill` | any declared `prefill_seed` source | non-submittable seed only |
| `draft` | `ActionDeckView`, `CommandConsoleView`, `IntentComposerView` | first editable operator-owned state |
| `validated` | `ActionDeckView`, `CommandConsoleView`, `IntentComposerView` | explicit validation against targets, parameters, and policy |
| `submitted` | `ActionDeckView`, `CommandConsoleView`, `IntentComposerView` | explicit submit action only |
| `approved` / `rejected` | backend authority or mediated adjudication stream | observed adjudication of a submitted intent only |
| `executed` | mediated operation or result stream only | never entered directly by local gesture |

### Command-Target Brokerage

- every actionable shell mode must declare exactly one `command_target` broker
- sources may suggest command targets from selection, focus, pins, compare sets,
  replay anchors, or world picks
- only the broker may publish the normalized command-target bundle
- broker output must include actor or session context, canonical target refs,
  scope, provenance, policy visibility, and broker sequence or epoch

### Conflict Presentation

If current selection, focus, pinning, policy overlays, or approval scope produce
incompatible command targets, the broker must surface explicit conflict state
before `validated`.

Allowed resolutions:

- keep current draft target
- replace with latest broker target
- split into separate drafts

Unresolved conflicts block submission.

### Guardrails Against Accidental Execution

- declared `prefill_seed` paths, including linked gestures, brushing, replay
  scrubbing, world picks, explicit camera-linked seed gestures, and attention
  interrupts, may create or refresh shell-local `prefill` only
- ordinary navigation-only camera movement remains intent-neutral by default
  unless the shell mode explicitly declares it as seed-capable
- no linked gesture may directly enter `validated`, `submitted`, `approved`, or
  `executed`
- `prefill -> draft`, `draft -> validated`, and `validated -> submitted` each
  require distinct explicit operator promotion
- target changes, exclusions, policy changes, or broker epoch changes after
  `validated` must demote the intent to `draft` or invalidate it visibly

