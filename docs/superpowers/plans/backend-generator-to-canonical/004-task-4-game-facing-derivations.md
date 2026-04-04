## Task 4: Register The Shared C:L2 Presentation Views For Commander, Researcher, Thread, And Core Shell Surfaces

**Files:**
- Modify: `crates/pipeline/normalizers/src/c_view_catalog.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/c_l2_projection_catalog.rs`

**Milestone unlock:** the shared presentation hierarchy is explicit, and the first reusable shell-defined `C:L2` contracts are satisfied by generator-supplied SQL views for commander, researcher, thread, and core shell surfaces over shared `A:L2+` and `B:L2+` storage rows.

**Forbidden shortcuts:** do not create app-local projection code instead of SQL view contracts; do not read raw `L0/L1` rows; do not define shell views without explicit source families, join keys, row grain, required columns, and ABAC/exclusion behavior.

- [ ] **Step 1: Write the failing C:L2 projection-catalog test**

Write a test that proves all of the following are registered in `c_view_catalog.rs`:

- `c_l2_commander_sessions_v`
- `c_l2_commander_tasks_v`
- `c_l2_researcher_artifacts_v`
- `c_l2_thread_timeline_v`
- `c_l2_shell_governance_v`

For each view, assert:

- the source families are named explicitly from shared `A:L2+` and `B:L2+` rows
- the required join keys are present
- the output column list is complete
- ABAC/exclusion and refresh metadata are declared
- the view kind is `VIEW` or `MATERIALIZED VIEW`

- [ ] **Step 2: Run the projection-catalog test and confirm it fails**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`

Expected: FAIL with missing `C:L2` view specs and no shared registration of the first presentation views.

- [ ] **Step 3: Implement the shared C:L2 presentation view catalog**

Update `c_view_catalog.rs` so it contains the authoritative generator-supplied implementation catalog for the shell-defined `C:L2` contracts in this plan slice.

Register SQL definitions for:

- `c_l2_commander_sessions_v` - one row per session with status, active agents, runtime summary, lineage anchor, and refresh watermark
- `c_l2_commander_tasks_v` - one row per task with status, assigned agent, queue state, exclusion state, lineage anchor, and update watermark
- `c_l2_researcher_artifacts_v` - one row per artifact with artifact family, producer task, producer agent, source event, exclusion state, and creation timestamp
- `c_l2_thread_timeline_v` - one row per merged thread event spanning messages, tasks, artifacts, and intents with stable event ordering
- `c_l2_shell_governance_v` - one row per exclusion/conflict/replay/backpressure/governance event with policy scope and visibility state

Each contract must read directly from shared `A:L2+` and `B:L2+` row families.

- [ ] **Step 4: Re-run the C:L2 projection-catalog test**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`

Expected: PASS with the first shared `C:L2` view families registered over A/B `L2+` storage.
