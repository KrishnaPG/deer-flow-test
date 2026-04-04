### Task 3: Add `FileSaved` Handoff, Derivation Triggers, Internal Diagnostics Separation, And LiveKit Boundary Notes

**Files:**
- Create: `crates/storage/core/src/downstream_handoff.rs`
- Create: `crates/storage/core/src/diagnostics.rs`
- Create: `crates/storage/core/src/boundary.rs`
- Test: `crates/storage/core/tests/file_saved_contract.rs`
- Test: `crates/storage/core/tests/post_commit_trigger.rs`
- Test: `crates/storage/core/tests/stress_scenarios.rs`

- [ ] **Step 1: Write the failing handoff and boundary tests**

```rust
// crates/storage/core/tests/file_saved_contract.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, FileSaved, LogicalWriteId, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind};
use deer_storage_core::downstream_handoff::make_file_saved;

#[test]
fn file_saved_carries_full_routing_context() {
    let payload = make_file_saved(
        LogicalWriteId::new("write_1"),
        "A/L0/as-is/chat-note/hash.jsonl",
        StorageHierarchyTag::new("A"),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        StoragePayloadKind::new("chat-note"),
        StoragePayloadFormat::new("jsonl"),
        vec![("mission_id".into(), "mission_1".into())],
        vec!["source_event_1".into()],
        vec![("mission".into(), "mission_1".into())],
    );

    match payload.target {
        deer_foundation_contracts::FileSavedTarget::SingleFile { relative_path } => {
            assert_eq!(relative_path, "A/L0/as-is/chat-note/hash.jsonl");
        }
        _ => panic!("expected single-file target"),
    }
    assert_eq!(payload.correlation_ids[0].1, "mission_1");
}
```

```rust
// crates/storage/core/tests/post_commit_trigger.rs
use deer_storage_core::downstream_handoff::derive_trigger_from_manifest;

#[test]
fn manifest_trigger_uses_manifest_target_not_partial_members() {
    let trigger = derive_trigger_from_manifest(
        "write_7",
        "B/L3/chunks/transcript/day/manifest.json",
        vec![("mission_id".into(), "mission_1".into())],
        vec!["parent_1".into()],
        vec![("date".into(), "2026-04-04".into())],
    );
    assert_eq!(trigger.relative_target, "B/L3/chunks/transcript/day/manifest.json");
    assert_eq!(trigger.correlation_ids[0].1, "mission_1");
}
```

```rust
// crates/storage/core/tests/stress_scenarios.rs
use deer_storage_core::boundary::livekit_bypass_note;
use deer_storage_core::diagnostics::InternalLifecycleEvent;

#[test]
fn livekit_media_bypass_stays_outside_normal_storage_ingress() {
    assert!(livekit_bypass_note().contains("landing zones"));
}

#[test]
fn internal_diagnostics_are_not_public_storage_facts() {
    let event = InternalLifecycleEvent::WriteStarted;
    assert_eq!(event.as_str(), "WriteStarted");
}
```

- [ ] **Step 2: Run the handoff tests to verify they fail**

Run: `cargo test -p deer-storage-core --test file_saved_contract --test post_commit_trigger --test stress_scenarios -v`

Expected: FAIL with missing handoff helpers, missing trigger helper, and missing diagnostics/boundary modules.

- [ ] **Step 3: Implement the minimal handoff and diagnostics layer without drifting into downstream consumers**

```rust
// crates/storage/core/src/downstream_handoff.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, DerivationTrigger, FileSaved, LogicalWriteId, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind};

pub fn make_file_saved(
    logical_write_id: LogicalWriteId,
    relative_path: &str,
    hierarchy: StorageHierarchyTag,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    payload_kind: StoragePayloadKind,
    format: StoragePayloadFormat,
    correlation_ids: Vec<(String, String)>,
    lineage_refs: Vec<String>,
    routing_tags: Vec<(String, String)>,
) -> FileSaved {
    FileSaved {
        logical_write_id,
        target: deer_foundation_contracts::FileSavedTarget::SingleFile {
            relative_path: relative_path.into(),
        },
        hierarchy,
        level,
        plane,
        payload_kind,
        format,
        correlation_ids,
        lineage_refs,
        routing_tags,
    }
}

pub fn derive_trigger_from_manifest(
    logical_write_id: &str,
    manifest_path: &str,
    correlation_ids: Vec<(String, String)>,
    lineage_refs: Vec<String>,
    routing_tags: Vec<(String, String)>,
) -> DerivationTrigger {
    DerivationTrigger {
        logical_write_id: LogicalWriteId::new(logical_write_id),
        relative_target: manifest_path.into(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L3,
        plane: CanonicalPlane::Chunks,
        payload_kind: StoragePayloadKind::new("transcript"),
        format: StoragePayloadFormat::new("parquet"),
        correlation_ids,
        lineage_refs,
        routing_tags,
    }
}
```

```rust
// crates/storage/core/src/diagnostics.rs
pub enum InternalLifecycleEvent {
    WriteStarted,
    RetryScheduled,
    WorkerFailure,
}

impl InternalLifecycleEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WriteStarted => "WriteStarted",
            Self::RetryScheduled => "RetryScheduled",
            Self::WorkerFailure => "WorkerFailure",
        }
    }
}
```

```rust
// crates/storage/core/src/boundary.rs
pub fn livekit_bypass_note() -> &'static str {
    "LiveKit media may bypass the storage service and write directly to landing zones before later hash/promote into canonical truth."
}
```

- [ ] **Step 4: Run the handoff tests to verify they pass**

Run: `cargo test -p deer-storage-core --test file_saved_contract --test post_commit_trigger --test stress_scenarios -v`

Expected: PASS with full `FileSaved` payload assembly, manifest-based trigger emission, and the LiveKit/internal-diagnostics boundary verified.

- [ ] **Step 5: Commit the handoff slice**

```bash
git add crates/storage/core crates/foundation/contracts/src/events.rs
git commit -m "feat: add storage handoff semantics"
```
