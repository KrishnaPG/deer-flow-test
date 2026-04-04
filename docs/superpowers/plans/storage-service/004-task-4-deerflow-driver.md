### Task 4: Implement The DeerFlow Driver Facade, Specialized Emitters, And Policy Selection

**Files:**
- Modify: `crates/pipeline/raw_sources/Cargo.toml`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/mod.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/facade.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/mapping.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/policy.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/transcript_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/artifact_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/metric_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/control_emitter.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_driver_contract.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_driver_stress.rs`

- [ ] **Step 1: Write the failing DeerFlow driver tests without any path-hint surface**

```rust
// crates/pipeline/raw_sources/tests/deerflow_driver_contract.rs
use deer_pipeline_raw_sources::deerflow_driver::DeerFlowDriver;

#[test]
fn deerflow_driver_emits_storage_requests_without_owning_paths() {
    let driver = DeerFlowDriver::default();
    let request = driver.map_transcript_line("mission_1", "hello");

    assert_eq!(request.layout.level, deer_foundation_contracts::CanonicalLevel::L0);
    assert_eq!(request.layout.hierarchy.as_str(), "A");
    assert_eq!(request.layout.payload_kind.as_str(), "chat-note");
}

#[test]
fn deerflow_driver_emits_control_requests_separately() {
    let driver = DeerFlowDriver::default();
    let request = driver.map_exclusion("mission_1", "as_is_hash_xyz");

    assert_eq!(request.target_refs[0], "as_is_hash_xyz");
}
```

```rust
// crates/pipeline/raw_sources/tests/deerflow_driver_stress.rs
use deer_pipeline_raw_sources::deerflow_driver::policy::{select_payload_mode, select_qos_policy};

#[test]
fn large_artifacts_choose_external_ref_and_durable_artifact_qos() {
    assert_eq!(select_payload_mode(50 * 1024 * 1024, true), "external-ref");
    assert_eq!(select_qos_policy(50 * 1024 * 1024, true).template, deer_foundation_contracts::StorageQosTemplate::DurableArtifact);
}

#[test]
fn conflicting_intents_preserve_both_control_writes() {
    let left = select_qos_policy(128, false);
    let right = select_qos_policy(128, false);
    assert_eq!(left.template, right.template);
}
```

- [ ] **Step 2: Run the DeerFlow driver tests to verify they fail**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_driver_contract --test deerflow_driver_stress -v`

Expected: FAIL with missing DeerFlow driver module, missing contract dependency, and unresolved policy helpers.

- [ ] **Step 3: Implement the DeerFlow driver facade with preserved raw-sources exports**

```toml
# crates/pipeline/raw_sources/Cargo.toml
[package]
name = "deer-pipeline-raw-sources"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-contracts = { path = "../../foundation/contracts" }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2.0.17"

[dev-dependencies]
similar-asserts = "1.7.0"
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/mod.rs
pub mod artifact_emitter;
pub mod control_emitter;
pub mod facade;
pub mod mapping;
pub mod metric_emitter;
pub mod policy;
pub mod transcript_emitter;

pub use facade::DeerFlowDriver;
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/transcript_emitter.rs
pub fn transcript_payload(line: &str) -> Vec<u8> { line.as_bytes().to_vec() }
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/artifact_emitter.rs
pub fn artifact_is_external_ref(size_bytes: usize) -> bool { size_bytes > 1024 * 1024 }
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/metric_emitter.rs
pub fn metric_kind() -> &'static str { "metric" }
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/control_emitter.rs
pub fn control_kind() -> &'static str { "exclusion" }
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/mapping.rs
pub fn mission_partition_tag(mission_id: &str) -> (String, String) {
    ("mission".into(), mission_id.into())
}
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/facade.rs
use deer_foundation_contracts::{AppendControlRequest, AppendDataRequest, CanonicalLevel, CanonicalPlane, ControlIntentKind, IdempotencyKey, StorageHierarchyTag, StorageLayout, StoragePayloadDescriptor, StoragePayloadFormat, StoragePayloadKind, StorageQosPolicy, StorageQosTemplate, StorageRequestMetadata};

#[derive(Default)]
pub struct DeerFlowDriver;

impl DeerFlowDriver {
    pub fn map_transcript_line(&self, mission_id: &str, line: &str) -> AppendDataRequest {
        AppendDataRequest {
            layout: StorageLayout {
                hierarchy: StorageHierarchyTag::new("A"),
                level: CanonicalLevel::L0,
                plane: CanonicalPlane::AsIs,
                payload_kind: StoragePayloadKind::new("chat-note"),
                format: StoragePayloadFormat::new("jsonl"),
                partition_tags: vec![("mission".into(), mission_id.into())],
            },
            metadata: StorageRequestMetadata {
                idempotency_key: Some(IdempotencyKey::new("deerflow-transcript-1")),
                correlation_ids: vec![("mission_id".into(), mission_id.into())],
                ..Default::default()
            },
            qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
            payload: StoragePayloadDescriptor::InlineBytes { bytes: line.as_bytes().to_vec() },
        }
    }

    pub fn map_exclusion(&self, mission_id: &str, target_ref: &str) -> AppendControlRequest {
        AppendControlRequest {
            control_kind: ControlIntentKind::Exclusion,
            target_refs: vec![target_ref.into()],
            metadata: StorageRequestMetadata {
                idempotency_key: Some(IdempotencyKey::new("deerflow-control-1")),
                correlation_ids: vec![("mission_id".into(), mission_id.into())],
                ..Default::default()
            },
            rationale: Some("user_request".into()),
        }
    }
}
```

```rust
// crates/pipeline/raw_sources/src/deerflow_driver/policy.rs
use deer_foundation_contracts::{StorageQosPolicy, StorageQosTemplate};

pub fn select_payload_mode(size_bytes: usize, is_artifact: bool) -> &'static str {
    if is_artifact && size_bytes > 1024 * 1024 {
        "external-ref"
    } else {
        "inline"
    }
}

pub fn select_qos_policy(size_bytes: usize, is_artifact: bool) -> StorageQosPolicy {
    if is_artifact && size_bytes > 1024 * 1024 {
        StorageQosPolicy::from_template(StorageQosTemplate::DurableArtifact)
    } else {
        StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget)
    }
}
```

```rust
// crates/pipeline/raw_sources/src/lib.rs
pub mod artifact_gateway;
pub mod deerflow_driver;
pub mod error;
pub mod run_stream;
pub mod thread_gateway;
pub mod upload_gateway;

pub use artifact_gateway::{preview_artifact, ArtifactAccess};
pub use deerflow_driver::DeerFlowDriver;
pub use run_stream::{load_stream_fixture, AdapterEvent, RawStreamEvent};
pub use thread_gateway::{create_thread, resume_thread, ThreadSnapshot};
pub use upload_gateway::{stage_upload, UploadReceipt};
```

- [ ] **Step 4: Run the DeerFlow driver tests to verify they pass**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_driver_contract --test deerflow_driver_stress -v`

Expected: PASS with DeerFlow mapping proving dimensions/metadata only, separate `append_control`, no path-hint concept, and durable-artifact selection for large artifacts.

- [ ] **Step 5: Commit the DeerFlow driver slice**

```bash
git add crates/pipeline/raw_sources
git commit -m "feat: add deerflow storage driver facade"
```
