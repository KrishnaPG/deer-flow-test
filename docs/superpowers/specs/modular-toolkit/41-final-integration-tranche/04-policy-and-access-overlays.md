## Policy And Access Overlays

### Pinned Rules

- policy overlays are shell-level constraints applied on top of canonical truth
- every policy evaluation must carry a `policy_epoch` and `policy_reason`
- no linked interaction path may preserve access to an object the current policy
  excludes or masks beyond joinability

### Exclusions And Tombstones

- exclusions are authoritative deny overlays
- when an object becomes excluded, the shell must either:
  - replace direct detail with a tombstone state, where allowed, or
  - remove it entirely if even tombstone visibility is forbidden
- tombstones must not leak hidden content, derivative previews, or reopen paths

### ABAC Masking

- masked objects may participate in linked interaction only when remaining fields
  still preserve joinability and drill-down safety
- if masking removes required join keys, compare axes, or drill-down targets, the
  affected linked interaction path becomes unsupported for that object

### Large-Object Mediated Handoff

Large objects must use a `LargeObjectAccessContract` with:

- canonical target reference
- ABAC validation at request time
- representation or version identity
- handoff form such as pre-signed URL or streaming session token
- expiry, revocation, and audit metadata

Rule:

- large objects are never proxied inline through linked interaction brokers

### Policy-Driven State Invalidation

On relevant `policy_epoch` changes, shell brokers must recompute:

- `selection_state`
- `pin_set`
- `compare_set`
- `drill_down_targets`
- `focus_state`

Rules:

- excluded references must be dropped or tombstoned immediately
- no shell may preserve ghost selection, stale compare alignment, or reopen paths
  to pre-policy detail

### Intent Rejection Behavior

`action_intent_emission` must revalidate policy at `validated` and `submitted`
stages.

Rejected intents must return:

- rejected target references
- rejection class
- operator-safe reason text
- resulting draft or context transition

Do not auto-retarget intents to fallback objects.

