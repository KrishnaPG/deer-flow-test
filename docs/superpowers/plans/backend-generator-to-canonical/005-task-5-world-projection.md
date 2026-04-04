## Task 5: Register The Battle-Command And World-Primary Shell Views In The Same C:L2 Catalog

**Files:**
- Modify: `crates/pipeline/normalizers/src/c_view_catalog.rs`
- Modify: `crates/pipeline/raw_sources/src/c_query_contract.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/c_l2_world_shell_catalog.rs`

**Milestone unlock:** battle-command and world-primary are expressed as one shell consumer slice inside hierarchy C, with explicit shell-defined `C:L2` contracts and generator-supplied SQL view implementations over shared `A:L2+` and `B:L2+` storage rows rather than a separate world-truth layer.

**Forbidden shortcuts:** do not describe world or battle state as source-of-truth data; do not build these shell views on raw `L0/L1`; do not bypass the shared `C:L2` catalog with app-local code; do not define shell requirements without naming exact view ids and output columns.

- [ ] **Step 1: Write the failing world-shell catalog test**

Write a test that proves all of the following are registered for the battle-command/world-primary shell slice:

- `c_l2_battle_command_summary_v`
- `c_l2_battle_command_units_v`
- `c_l2_battle_command_history_v`
- `c_l2_battle_command_artifacts_v`

For each view, assert:

- it is owned by the battle-command/world-primary shell in `c_query_contract.rs`
- it reads directly from shared `A:L2+` and `B:L2+` row families
- it names required columns for tactical summary, units, history anchors, or artifact unlocks
- it declares ABAC/exclusion behavior and refresh policy

- [ ] **Step 2: Run the world-shell catalog test and confirm it fails**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v`

Expected: FAIL with missing battle-command/world-primary `C:L2` registrations.

- [ ] **Step 3: Implement the battle-command/world-primary shell registrations**

Update `c_query_contract.rs` and `c_view_catalog.rs` so the battle-command/world-primary shell declares these `C:L2` contracts and the generator supplies implementations for them:

- `c_l2_battle_command_summary_v` - one row per battle/session with active agents, running tasks, queued tasks, pressure level, latest event id, and refresh watermark
- `c_l2_battle_command_units_v` - one row per tactical unit/task-agent pairing with lane/state/health and lineage anchors
- `c_l2_battle_command_history_v` - one row per history anchor with event story, artifact linkage, and timeline ordering
- `c_l2_battle_command_artifacts_v` - one row per artifact unlock/access state with family, producer, visibility, and creation timestamp

These views must read directly from shared `A:L2+` and `B:L2+` storage rows. They may reuse the same join conventions as Task 4, but they do not become a second-layer `C-on-C` canon.

- [ ] **Step 4: Re-run the world-shell catalog test and impacted catalog tests**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v && cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`

Expected: PASS with battle-command/world-primary documented as one shell-defined `C:L2` consumer package with generator-supplied implementations in the same presentation catalog.
