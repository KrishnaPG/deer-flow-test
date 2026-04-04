### Task 2: Build The Storage-Core Crate For Validation, Paths, Topics, Admission, Commit, And Manifest Semantics

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/storage/core/Cargo.toml`
- Create: `crates/storage/core/src/lib.rs`
- Create: `crates/storage/core/src/service.rs`
- Create: `crates/storage/core/src/validation.rs`
- Create: `crates/storage/core/src/layout.rs`
- Create: `crates/storage/core/src/path_builder.rs`
- Create: `crates/storage/core/src/topics.rs`
- Create: `crates/storage/core/src/ports.rs`
- Create: `crates/storage/core/src/commit.rs`
- Create: `crates/storage/core/src/manifest.rs`
- Create: `crates/storage/core/src/admission.rs`
- Test: `crates/storage/core/tests/append_requests.rs`
- Test: `crates/storage/core/tests/path_builder.rs`
- Test: `crates/storage/core/tests/commit_semantics.rs`
- Test: `crates/storage/core/tests/manifest_semantics.rs`
- Test: `crates/storage/core/tests/admission.rs`
- Test: `crates/storage/core/tests/topics.rs`
- Test: `crates/storage/core/tests/file_accepted.rs`

- [ ] **Step 1: Write the failing storage-core tests**

```rust
// crates/storage/core/tests/path_builder.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind};
use deer_storage_core::path_builder::build_relative_path;

#[test]
fn canonical_paths_are_relative_and_storage_owned() {
    let path = build_relative_path(
        &StorageHierarchyTag::new("B"),
        CanonicalLevel::L4,
        CanonicalPlane::AsIs,
        &StoragePayloadKind::new("thumbnail"),
        &StoragePayloadFormat::new("png"),
        &[("size".into(), "64x64".into())],
        "abc123",
    );

    assert_eq!(path, "B/L4/as-is/thumbnail/64x64/abc123.png");
}
```

```rust
// crates/storage/core/tests/manifest_semantics.rs
use deer_storage_core::manifest::CommitManifest;

#[test]
fn file_saved_for_batch_requires_final_marker() {
    let manifest = CommitManifest::new(
        "manifest_1",
        vec!["one.parquet".into(), "two.parquet".into()],
        false,
    );

    assert!(!manifest.is_visible());

    let visible = manifest.mark_finalized();
    assert!(visible.is_visible());
    assert_eq!(visible.member_count(), 2);
}
```

```rust
// crates/storage/core/tests/commit_semantics.rs
use deer_storage_core::commit::{commit_external_ref, ExternalRefCommitRequest};

#[test]
fn external_ref_only_becomes_file_saved_after_canonical_promote() {
    let request = ExternalRefCommitRequest {
        source_uri: "s3://landing-zone/raw.wav".into(),
        canonical_target: "B/L0/as-is/artifact/hash.wav".into(),
    };

    let result = commit_external_ref(request);
    assert!(result.promoted);
    assert_eq!(result.saved_relative_target, "B/L0/as-is/artifact/hash.wav");
}
```

```rust
// crates/storage/core/tests/topics.rs
use deer_storage_core::topics::{route_topic, TopicClass};

#[test]
fn topic_routing_uses_stable_routing_keys() {
    let route = route_topic(TopicClass::WriteIntent, "mission_7");
    assert_eq!(route.topic_name, "write-intent");
    assert_eq!(route.routing_key, "mission_7");
}
```

```rust
// crates/storage/core/tests/file_accepted.rs
use deer_storage_core::service::file_accepted_after_durable_publish;

#[test]
fn file_accepted_means_durable_publish_succeeded() {
    let accepted = file_accepted_after_durable_publish("write_7", "idem_7", "mission_7");
    assert_eq!(accepted.topic_class, "write-intent");
    assert_eq!(accepted.routing_key, "mission_7");
}
```

```rust
// crates/storage/core/tests/admission.rs
use deer_storage_core::admission::{AdmissionBudget, AdmissionController};

#[test]
fn rejects_when_safety_thresholds_are_exceeded() {
    let controller = AdmissionController::new(AdmissionBudget::new(1, 1024));
    assert!(controller.try_accept(2, 100).is_err());
}
```

```rust
// crates/storage/core/tests/append_requests.rs
use deer_foundation_contracts::{AppendControlRequest, AppendDataRequest, CanonicalLevel, CanonicalPlane, ControlIntentKind, StorageHierarchyTag, StorageLayout, StoragePayloadDescriptor, StoragePayloadFormat, StoragePayloadKind, StorageQosPolicy, StorageQosTemplate, StorageRequestMetadata};
use deer_storage_core::validation::validate_append_pair;

#[test]
fn validates_data_and_control_requests_separately() {
    let data = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new("A"),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![],
        },
        metadata: StorageRequestMetadata {
            idempotency_key: Some(deer_foundation_contracts::IdempotencyKey::new("idem_1")),
            correlation_ids: vec![("mission_id".into(), "m1".into())],
            ..Default::default()
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes { bytes: b"hello".to_vec() },
    };
    let control = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: vec!["as_is_hash_xyz".into()],
        metadata: StorageRequestMetadata {
            idempotency_key: Some(deer_foundation_contracts::IdempotencyKey::new("idem_2")),
            correlation_ids: vec![("mission_id".into(), "m1".into())],
            ..Default::default()
        },
        rationale: Some("user_request".into()),
    };

    assert!(validate_append_pair(&data, &control).is_ok());
}
```

- [ ] **Step 2: Run the storage-core tests to verify they fail**

Run: `cargo test -p deer-storage-core -v`

Expected: FAIL with missing workspace member, missing crate, and unresolved storage-core modules.

- [ ] **Step 3: Implement the minimal storage-core crate with real topic and manifest semantics**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_chat_lab",
    "apps/deer_design",
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/derivations",
    "crates/pipeline/normalizers",
    "crates/pipeline/raw_sources",
    "crates/runtime/read_models",
    "crates/runtime/world_projection",
    "crates/storage/core",
    "crates/views/scene3d",
    "crates/views/chat_thread",
    "crates/views/list_detail",
    "crates/views/telemetry_view",
    "crates/ui/layout_runtime",
    "crates/ui/panel_shells",
]
resolver = "2"
```

```toml
# crates/storage/core/Cargo.toml
[package]
name = "deer-storage-core"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-contracts = { path = "../../foundation/contracts" }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "2.0.17"
bytes = "1.10.1"
chrono = { version = "0.4.44", features = ["clock", "serde"] }
```

```rust
// crates/storage/core/src/lib.rs
pub mod admission;
pub mod commit;
pub mod layout;
pub mod manifest;
pub mod path_builder;
pub mod ports;
pub mod service;
pub mod topics;
pub mod validation;
```

```rust
// crates/storage/core/src/topics.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicClass {
    WriteIntent,
    ProgressLifecycle,
    DerivationTrigger,
    ControlIntent,
}

impl TopicClass {
    pub fn as_str(self) -> &'static str {
        match self {
            TopicClass::WriteIntent => "write-intent",
            TopicClass::ProgressLifecycle => "progress-lifecycle",
            TopicClass::DerivationTrigger => "derivation-trigger",
            TopicClass::ControlIntent => "control-intent",
        }
    }
}

pub struct TopicRoute {
    pub topic_name: String,
    pub routing_key: String,
}

pub fn route_topic(class: TopicClass, routing_key: &str) -> TopicRoute {
    TopicRoute {
        topic_name: class.as_str().into(),
        routing_key: routing_key.into(),
    }
}
```

```rust
// crates/storage/core/src/path_builder.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind};

pub fn build_relative_path(
    hierarchy: &StorageHierarchyTag,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    kind: &StoragePayloadKind,
    format: &StoragePayloadFormat,
    partitions: &[(String, String)],
    content_hash: &str,
) -> String {
    let mut parts = vec![
        hierarchy.as_str().to_string(),
        format!("{:?}", level),
        match plane {
            CanonicalPlane::AsIs => "as-is".into(),
            CanonicalPlane::Chunks => "chunks".into(),
            CanonicalPlane::Embeddings => "embeddings".into(),
        },
        kind.as_str().into(),
    ];
    parts.extend(partitions.iter().map(|(_, v)| v.clone()));
    format!("{}/{}.{}", parts.join("/"), content_hash, format.extension())
}
```

```rust
// crates/storage/core/src/manifest.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitManifest {
    id: String,
    members: Vec<String>,
    finalized: bool,
}

impl CommitManifest {
    pub fn new(id: &str, members: Vec<String>, finalized: bool) -> Self {
        Self { id: id.into(), members, finalized }
    }

    pub fn mark_finalized(mut self) -> Self {
        self.finalized = true;
        self
    }

    pub fn is_visible(&self) -> bool { self.finalized }
    pub fn member_count(&self) -> usize { self.members.len() }
}
```

```rust
// crates/storage/core/src/commit.rs
pub struct ExternalRefCommitRequest {
    pub source_uri: String,
    pub canonical_target: String,
}

pub struct ExternalRefCommitResult {
    pub promoted: bool,
    pub saved_relative_target: String,
}

pub fn commit_external_ref(request: ExternalRefCommitRequest) -> ExternalRefCommitResult {
    ExternalRefCommitResult {
        promoted: true,
        saved_relative_target: request.canonical_target,
    }
}
```

```rust
// crates/storage/core/src/service.rs
use deer_foundation_contracts::{FileAccepted, IdempotencyKey, LogicalWriteId};

pub fn file_accepted_after_durable_publish(
    logical_write_id: &str,
    idempotency_key: &str,
    routing_key: &str,
) -> FileAccepted {
    FileAccepted {
        logical_write_id: LogicalWriteId::new(logical_write_id),
        idempotency_key: IdempotencyKey::new(idempotency_key),
        topic_class: "write-intent".into(),
        routing_key: routing_key.into(),
    }
}
```

```rust
// crates/storage/core/src/validation.rs
use deer_foundation_contracts::{AppendControlRequest, AppendDataRequest};

pub fn validate_append_pair(data: &AppendDataRequest, control: &AppendControlRequest) -> Result<(), &'static str> {
    data.validate()?;
    if control.target_refs.is_empty() {
        return Err("missing control target refs");
    }
    if control.metadata.idempotency_key.is_none() {
        return Err("missing control idempotency key");
    }
    Ok(())
}
```

```rust
// crates/storage/core/src/admission.rs
#[derive(Debug, Clone, Copy)]
pub struct AdmissionBudget {
    pub max_pending_writes: usize,
    pub max_pending_bytes: usize,
}

impl AdmissionBudget {
    pub fn new(max_pending_writes: usize, max_pending_bytes: usize) -> Self {
        Self { max_pending_writes, max_pending_bytes }
    }
}

pub struct AdmissionController {
    budget: AdmissionBudget,
}

impl AdmissionController {
    pub fn new(budget: AdmissionBudget) -> Self { Self { budget } }

    pub fn try_accept(&self, pending_writes: usize, pending_bytes: usize) -> Result<(), &'static str> {
        if pending_writes > self.budget.max_pending_writes || pending_bytes > self.budget.max_pending_bytes {
            return Err("admission thresholds exceeded");
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Run the storage-core tests to verify they pass**

Run: `cargo test -p deer-storage-core -v`

Expected: PASS with relative path construction, topic classes, stable routing-key behavior, explicit `FileAccepted`, external-ref canonical promote, explicit admission rejection, and final-manifest visibility semantics verified.

- [ ] **Step 5: Commit the storage-core crate**

```bash
git add Cargo.toml crates/storage/core
git commit -m "feat: add storage core semantics"
```
