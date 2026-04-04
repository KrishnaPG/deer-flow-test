## Task 2: Land DeerFlow Source Objects Into Shared A/B Storage Families

**Files:**
- Modify: `crates/pipeline/raw_sources/Cargo.toml`
- Create: `crates/pipeline/raw_sources/src/envelopes.rs`
- Create: `crates/pipeline/raw_sources/src/hierarchy_a.rs`
- Create: `crates/pipeline/raw_sources/src/hierarchy_b.rs`
- Modify: `crates/pipeline/raw_sources/src/storage_contract.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow.rs`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_adapter.rs`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json`

**Milestone unlock:** DeerFlow becomes the first generator binding that lands storage-native rows in the shared A/B family registry, with lineage, immutable keys, level/plane occupancy, and source binding metadata that later normalization can sanitize into `A:L2+` and `B:L2+` query inputs.

**Forbidden shortcuts:** do not emit app-facing DTOs from `raw_sources`; do not invent DeerFlow-only family ids; do not flatten multi-agent activity into a single summary blob; do not land any row without immutable keys, lineage anchors, and a shared A/B family id.

- [ ] **Step 1: Write the failing DeerFlow landing test and fixtures**

Create fixture coverage for:

- session/thread snapshot landing
- multi-agent activity landing
- replay-window and checkpoint landing
- operator intent and exclusion landing
- artifact-access metadata landing
- storage backpressure and conflict ordering landing

Write a test that proves all of the following:

- `thread_snapshot` binds to `a_session_snapshot`
- live message/task/artifact/runtime payloads bind to the correct `a_*` family ids
- normalization-seed rows for session, artifact access, replay window, intent, exclusion, and backpressure land in the correct `b_*` family ids where Task 3 expects them
- every landed row includes `generator_key`, source kind, level, plane, source object key, and deterministic lineage sequence
- the adapter never produces consumer-facing `C:L2` rows

- [ ] **Step 2: Run the raw-source adapter test and confirm it fails**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`

Expected: FAIL with missing A/B landing types, missing DeerFlow family bindings, and missing lineage/binding metadata.

- [ ] **Step 3: Implement the DeerFlow landing modules around shared A/B family ids**

Implement `envelopes.rs`, `hierarchy_a.rs`, and `hierarchy_b.rs` so they expose:

- a shared `GeneratorBinding`
- storage hierarchy/level/plane refs
- append-only lineage metadata
- typed A landing rows for `a_session_snapshot`, `a_message_event`, `a_task_event`, `a_artifact_event`, `a_runtime_event`, `a_intent_event`, `a_exclusion_event`, `a_replay_checkpoint`, and `a_backpressure_event`
- typed B seed rows for `b_session`, `b_artifact`, `b_artifact_access`, `b_intent`, `b_exclusion`, `b_replay_window`, `b_replay_checkpoint`, `b_backpressure`, and `b_transform` inputs where DeerFlow fixtures already contain enough information to land them safely

Implementation notes:

- bind DeerFlow fixture labels through the shared registry in `storage_contract.rs`
- preserve source object ids, event ids, thread/run/task/agent correlations, and deterministic sequence ordering
- keep artifact preview, replay, exclusion, checkpoint, and backpressure payloads as A/B storage rows, not consumer view rows

- [ ] **Step 4: Re-run the DeerFlow landing test**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`

Expected: PASS with DeerFlow fixtures landing into shared A/B storage families and no consumer-facing canon crossing the raw-source boundary.
