## Shared Shell Interaction State

Linked dashboard behavior requires shell-local shared state.

That state is non-canonical and must never mutate storage truth.

Required shared state slices:

- `selection_state`
  - current selected canonical references
- `filter_state`
  - normalized active filter predicates with `global`, `panel`, or `view` scope
- `brush_state`
  - transient exploratory range, cohort, region, or timeline state
- `focus_state`
  - singular active target driving inspector, center, or modal routing
- `pin_set`
  - intentionally retained canonical references across shell navigation
- `compare_set`
  - bounded ordered set of comparable canonical references
- `drill_down_targets`
  - resolvable shell-local targets that map references to supported detail hosts
- `intent_draft_context`
  - unsent operator composition context for command and intervention flow
- `command_target_state`
  - normalized actionable target bundle for command surfaces
- `viewport_state`
  - current world viewport/camera target state
- `camera_sync_state`
  - brokered world/minimap camera alignment state
- `replay_cursor_state`
  - singular active temporal inspection point
- `temporal_range_state`
  - bounded active temporal window
- `stream_lifecycle_state`
  - per-surface `connecting` / `live` / `catching_up` / `degraded` /
    `stalled` / `recovered` / `closed` state
- `policy_overlay_state`
  - current exclusion, masking, tombstone, `policy_epoch`, and `policy_reason`
    overlays affecting linked interaction
- `broker_epoch_map`
  - current broker epoch by interaction type for failover-safe propagation

## Canonical Reference Discipline

Shell state must reference canonical truth through typed references rather than
embedded mutable payloads.

Every durable linked reference should preserve enough information to resolve the
target safely, including:

- canonical record family
- primary identity
- required correlation keys
- representation-specific identity where relevant
- lineage or source identity where required

Do not use render position, array position, or backend-native opaque handles as
the durable basis of linked interaction state.

