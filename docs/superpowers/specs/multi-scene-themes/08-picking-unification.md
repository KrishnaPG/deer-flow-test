# PR H — Two-Phase Picking Unification

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** None (independent of scenes)

---

## Current State

`src/picking/` has four systems chained in `Update`:
1. `spatial_index_rebuild_system` — rebuilds `SpatialIndex` from all
   `Transform + Selectable` entities.
2. `selection_update_system` — processes `EntityClicked` messages.
3. `deselection_system` — clears selection on unoccupied click.
4. `selection_sync_system` — reads `Selected + WorldEntity`, writes to
   `HudState::selected_entity`.

The two phases (coarse spatial query vs precise picking) are conflated.
This PR makes them explicit, testable, and decoupled.

## Design

### New Resource

```rust
/// Result of phase-1 coarse query.
#[derive(Resource, Default)]
pub struct PickingCandidates {
    pub screen_pos: Vec2,
    pub candidates: Vec<Entity>,
}
```

### System Split

| System                         | Phase     | Input                    | Output                       |
| ------------------------------ | --------- | ------------------------ | ---------------------------- |
| `spatial_index_rebuild_system` | Pre-phase | ECS query                | `SpatialIndex`               |
| `coarse_picking_system`        | Phase 1   | `PickingCandidates.screen_pos` | `PickingCandidates.candidates` |
| `precise_picking_system`       | Phase 2   | `PickingCandidates`      | Emits `EntityClicked`        |
| `selection_update_system`      | Post      | `EntityClicked`          | `Selected` changes           |
| `deselection_system`           | Post      | Input state              | Clears `Selected`            |
| `selection_sync_system`        | Post      | `Selected + WorldEntity` | `HudState::selected_entity`  |

### Input Flow

The existing `Pointer<Click>` observer is replaced by a thin observer
that only writes `PickingCandidates.screen_pos`. The chained systems
handle the rest. This keeps Bevy's native input detection while making
the pipeline fully testable.

## Constants

Added to `constants.rs` under `visual`:

```rust
pub const PICKING_COARSE_RADIUS_PX: f32 = 24.0;
```

## Tests

Extend `tests/integration/picking.rs`:

| Test                                    | Verifies                                              |
| --------------------------------------- | ----------------------------------------------------- |
| `t_pick_07_coarse_candidates_populated` | Candidates non-empty when near a Selectable entity    |
| `t_pick_08_precise_selects_closest`     | Closest candidate is selected from two options        |
| `t_pick_09_no_candidates_no_click`      | Empty candidates produces no EntityClicked            |
| `t_pick_10_deselection_clears_selected` | Deselection removes Selected when no click this frame |
