### Task 5: Run The Integrated Proof For Shared Contracts, Storage Core, And DeerFlow Stress Behavior

**Files:**
- Test: `crates/foundation/contracts/tests/storage_contracts.rs`
- Test: `crates/storage/core/tests/*.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_driver_contract.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_driver_stress.rs`

- [ ] **Step 1: Run the targeted package sweep**

Run: `cargo test -p deer-foundation-contracts --test storage_contracts -v && cargo test -p deer-storage-core -v && cargo test -p deer-pipeline-raw-sources --test deerflow_driver_contract --test deerflow_driver_stress -v`

Expected: PASS with full append/control contracts, `FileAccepted` / `FileSaved`, path ownership, topic classes, manifest-based visibility, handoff semantics, and DeerFlow driver boundary tests all green.

- [ ] **Step 2: Run the full workspace verification sweep**

Run: `cargo test -v`

Expected: PASS with no regressions in existing workspace crates, including preserved re-export surfaces in `deer-foundation-contracts` and `deer-pipeline-raw-sources`.

- [ ] **Step 3: Commit the split implementation plan set when requested**

```bash
git add docs/superpowers/plans/storage-service
git commit -m "docs: split storage service implementation plan"
```

## Self-Review Checklist

- The split plan preserves the approved storage-service boundary from `docs/superpowers/specs/2026-04-04-shared-storage-service-design.md`.
- The corrected tasks address the serious gaps found in the stress review: `append_control`, `FileAccepted`, full `FileSaved`, QoS policy objects, no driver path hints, real Redpanda topic classes, fixed manifest semantics, and preserved existing crate exports.
- The storage service still stops at immutable save plus `FileSaved` handoff; no Iceberg/LakeFS/ClickHouse/Milvus ownership leaks back into the implementation tasks.
- Replay-safe tests stay at the event-contract boundary and do not implement downstream consumers.
- CAS/hash-collapse dedupe is still deferred.
