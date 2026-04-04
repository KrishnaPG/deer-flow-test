## Task 0: Define The Shared A/B Storage Contract And C:L2 SQL View Contract Before Any Adapter Work

**Files:**
- Create: `crates/pipeline/raw_sources/src/storage_contract.rs`
- Create: `crates/pipeline/raw_sources/src/c_query_contract.rs`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Test: `crates/pipeline/raw_sources/tests/storage_contract.rs`
- Create: `crates/pipeline/normalizers/src/c_view_catalog.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/c_view_catalog.rs`

**Milestone unlock:** every backend generator object binds to an explicit A/B storage family, every consumer shell declares the exact `C:L2` views it requires, and presentation canon is defined as a storage-native SQL view catalog rather than a generic promotion story.

**Forbidden shortcuts:** do not let consumers query raw `L0/L1`; do not invent DeerFlow-only family ids; do not define a `C:L2` view without join keys, row grain, source families, output columns, ABAC/exclusion behavior, and refresh policy; do not split shell-specific views into separate architectures.

### Required contract decisions in this task

| Concern | Required contract |
| --- | --- |
| Physical storage | Hierarchies A and B physically exist across `L0`-`L6` |
| Presentation hierarchy | Hierarchy C is query-only and starts at `C:L2` |
| Minimum query surface | consumers query `C:L2` only |
| Allowed `C:L2` inputs | `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6` only |
| Presentation canon | named SQL views/materialized views plus refresh metadata |
| Shell ownership | each shell declares required `C:L2` view ids explicitly |

### Shared A/B family ids that must exist after this task

| Family id | Hierarchy | Primary purpose | Minimum immutable keys |
| --- | --- | --- | --- |
| `a_session_snapshot` | A | observed session/thread snapshot | `generator_key`, `session_key`, `thread_key` |
| `a_message_event` | A | observed message/activity event | `generator_key`, `message_key`, `thread_key`, `agent_key` |
| `a_task_event` | A | observed task/progress event | `generator_key`, `task_key`, `thread_key`, `run_key` |
| `a_artifact_event` | A | observed artifact lifecycle event | `generator_key`, `artifact_key`, `task_key`, `run_key` |
| `a_runtime_event` | A | observed runtime/status event | `generator_key`, `run_key`, `agent_key` |
| `a_intent_event` | A | observed human/system intent | `generator_key`, `intent_key`, `thread_key`, `actor_key` |
| `a_exclusion_event` | A | observed exclusion/redaction event | `generator_key`, `exclusion_key`, `target_key` |
| `a_replay_checkpoint` | A | observed replay boundary event | `generator_key`, `checkpoint_key`, `run_key` |
| `a_backpressure_event` | A | observed storage/backpressure event | `generator_key`, `backpressure_key`, `run_key` |
| `b_session` | B | normalized session row | `session_key`, `thread_key` |
| `b_message` | B | normalized message row | `message_key`, `thread_key`, `agent_key` |
| `b_task` | B | normalized task row | `task_key`, `thread_key`, `run_key` |
| `b_artifact` | B | normalized artifact row | `artifact_key`, `task_key`, `run_key` |
| `b_artifact_access` | B | normalized artifact access row | `artifact_key`, `access_key` |
| `b_runtime_status` | B | normalized runtime status row | `run_key`, `agent_key` |
| `b_intent` | B | normalized intent row | `intent_key`, `thread_key`, `actor_key` |
| `b_exclusion` | B | normalized exclusion row | `exclusion_key`, `target_key` |
| `b_conflict` | B | normalized conflict row | `conflict_key`, `target_key` |
| `b_replay_window` | B | normalized replay window row | `thread_key`, `window_key` |
| `b_replay_checkpoint` | B | normalized replay checkpoint row | `checkpoint_key`, `run_key` |
| `b_backpressure` | B | normalized storage pressure row | `backpressure_key`, `run_key` |
| `b_transform` | B | normalized lineage/transform row | `transform_key`, `target_key` |

### DeerFlow first-shell `C:L2` view ids that must be registered in this task

| View id | Grain | Required A/B inputs |
| --- | --- | --- |
| `c_l2_commander_sessions_v` | one row per session | `a_session_snapshot`, `b_session`, `b_runtime_status` |
| `c_l2_commander_tasks_v` | one row per task | `a_task_event`, `b_task`, `b_runtime_status`, `b_exclusion` |
| `c_l2_researcher_artifacts_v` | one row per artifact | `a_artifact_event`, `b_artifact`, `b_exclusion` |
| `c_l2_thread_timeline_v` | one row per timeline event | `a_message_event`, `a_task_event`, `a_artifact_event`, `b_message`, `b_task`, `b_intent` |
| `c_l2_shell_governance_v` | one row per governance event | `a_exclusion_event`, `a_replay_checkpoint`, `a_backpressure_event`, `b_exclusion`, `b_conflict`, `b_replay_checkpoint`, `b_transform` |

Every `C:L2` view definition must declare:

- `consumer_shell`
- `view_id`
- `sql_name`
- `view_kind` (`VIEW` or `MATERIALIZED VIEW`)
- `row_grain`
- `required_join_keys`
- `source_families`
- `allowed_source_levels`
- `projected_columns`
- `abac_scope`
- `exclusion_behavior`
- `refresh_mode`, `refresh_watermark`, and `refresh_dependencies`

- [ ] **Step 1: Write the failing storage-contract and view-catalog tests**

Create tests that prove all of the following before any implementation exists:

- the shared registry names the A and B families above and attaches each family to explicit allowed levels
- consumer-facing access helpers reject direct reads from raw `L0/L1` families
- DeerFlow binds to shared family ids instead of introducing DeerFlow-only families
- each `C:L2` view listed above is registered with complete metadata
- each registered `C:L2` contract depends only on `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6`

- [ ] **Step 2: Run the targeted tests and confirm they fail**

Run the raw-source contract test target and the `C:L2` view-catalog test target.

Expected: failure because the A/B registry, consumer query guards, DeerFlow bindings, and `C:L2` SQL catalog do not exist yet.

- [ ] **Step 3: Implement the shared A/B storage contract**

Add `storage_contract.rs` with:

- explicit family ids for the A/B registry above
- hierarchy and allowed-level metadata for each family id
- immutable keys, row grain, lineage anchors, and storage kind for each family id
- DeerFlow-first binding hooks that map DeerFlow source shapes into the shared family ids without changing the registry surface
- helpers that mark families as consumer-visible only through registered `C:L2` views

- [ ] **Step 4: Implement the shared C query contract and SQL catalog**

Add `c_query_contract.rs` and `c_view_catalog.rs` with:

- a shell registry naming which `C:L2` view ids each shell requires
- a `CViewContract` type carrying the full projection metadata listed above
- explicit SQL text or SQL-builder output for each DeerFlow `C:L2` view listed in this task
- validation that every view declares join keys, columns, ABAC/exclusion behavior, and refresh metadata
- validation that every view reads only from allowed A/B levels and never from raw `L0/L1`

- [ ] **Step 5: Re-run the targeted tests**

Re-run the raw-source contract and `C:L2` view-catalog tests.

Expected: pass with explicit A/B family definitions, consumer query restrictions, DeerFlow first-binding coverage, and a complete `C:L2` SQL contract catalog.

- [ ] **Step 6: Review the contract tables against the architecture invariants**

Verify all of the following are now true:

- A and B are the only physical storage hierarchies in the contract
- C exists only as registered `C:L2` SQL view contracts in this slice
- no consumer shell depends on raw `L0/L1`
- DeerFlow is only a binding layer on top of the shared contract
