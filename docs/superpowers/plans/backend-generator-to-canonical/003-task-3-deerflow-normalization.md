## Task 3: Sanitize DeerFlow A/B Landings Into Shared L2+ Projection Inputs

**Files:**
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Modify: `crates/pipeline/normalizers/src/carrier.rs`
- Modify: `crates/pipeline/normalizers/src/promotions.rs`
- Modify: `crates/pipeline/normalizers/src/representation.rs`
- Modify: `crates/pipeline/normalizers/src/governance.rs`
- Test: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`

**Milestone unlock:** DeerFlow landing rows are sanitized into shared `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6` storage inputs that `C:L2` SQL views can query directly, while `L0` and `L1` remain append-only storage evidence only.

**Forbidden shortcuts:** do not describe this stage as promotion into the app model; do not let consumers query `L0` or `L1`; do not emit `C:L2` views here; do not drop lineage, hashes, joins, exclusions, or replay anchors from A/B rows during sanitation.

- [ ] **Step 1: Write the failing DeerFlow normalization test**

Write a test that proves all of the following:

- DeerFlow landings produce sanitized shared family rows such as `a_session_snapshot`, `a_task_event`, `a_artifact_event`, `a_runtime_event`, `b_session`, `b_task`, `b_artifact`, `b_exclusion`, `b_replay_checkpoint`, and `b_transform`, with level metadata set to `L2+` where appropriate
- higher-level rows can also be emitted where the fixture supports them, such as `b_artifact` at `L4`, `b_transform` at `L5`, or evaluation/deviation-oriented B families at `L6`
- every emitted `L2+` row preserves immutable join keys, source lineage, event anchors, and `as_is` hash identity
- the normalizer emits no consumer-ready `C:L2` view rows yet

- [ ] **Step 2: Run the DeerFlow normalization test and confirm it fails**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`

Expected: FAIL with missing DeerFlow landing support, missing sanitized A/B `L2+` rows, and missing lineage-preserving storage outputs.

- [ ] **Step 3: Implement the DeerFlow sanitation and normalization path for shared A/B `L2+` rows**

Update the normalizer modules so they:

- read DeerFlow landing batches from Task 2
- preserve `L0` and `L1` as storage evidence only
- emit shared A/B family ids from Task 0 with explicit target-level metadata for session, task, artifact, runtime, intent, exclusion, replay, backpressure, and transform/governance inputs
- use `promotions.rs` only as routing logic that selects the correct target A/B family and target level, not as a consumer-canon policy layer
- emit `as_is` representation metadata and hashes needed for later `Chunks` and `Embeddings` support

Implementation notes:

- frame this task as storage shaping for later SQL consumption, not presentation derivation
- keep lineage append-only: `source_events`, `derived_from`, `supersedes`, hashes, and row keys must survive every step
- make room for `L3/L4/L5/L6` rows when the source fixture supports them, without forcing every family to exist at every level

- [ ] **Step 4: Re-run the DeerFlow normalization test**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`

Expected: PASS with DeerFlow landing rows sanitized into shared A/B family ids at `L2+` and no consumer-facing `C:L2` views emitted yet.
