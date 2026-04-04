### Task 1: Define Shared Storage Contracts In `deer-foundation-contracts`

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/foundation/contracts/src/lib.rs`
- Modify: `crates/foundation/contracts/src/ids.rs`
- Create: `crates/foundation/contracts/src/storage.rs`
- Test: `crates/foundation/contracts/tests/storage_contracts.rs`

- [ ] **Step 1: Write the failing shared storage contract tests**

```rust
// crates/foundation/contracts/tests/storage_contracts.rs
use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, DerivationTrigger, FileAccepted, FileSaved,
    FileSavedTarget, IdempotencyKey, LogicalWriteId, StorageHierarchyTag, StorageLayout, StoragePayloadFormat,
    StoragePayloadKind, StorageQosPolicy, StorageQosTemplate, StorageRequestMetadata,
    AppendControlRequest, AppendDataRequest, ControlIntentKind,
};

#[test]
fn file_accepted_represents_durable_intent_handoff() {
    let accepted = FileAccepted {
        logical_write_id: LogicalWriteId::new("write_123"),
        idempotency_key: IdempotencyKey::new("idem_123"),
        topic_class: "write-intent".into(),
        routing_key: "mission_7".into(),
    };

    assert_eq!(accepted.topic_class, "write-intent");
}

#[test]
fn file_saved_requires_full_downstream_handoff_metadata() {
    let saved = FileSaved {
        logical_write_id: LogicalWriteId::new("write_123"),
        target: FileSavedTarget::SingleFile {
            relative_path: "B/L4/as-is/thumbnail/64x64/hash.png".into(),
        },
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L4,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("thumbnail"),
        format: StoragePayloadFormat::new("png"),
        correlation_ids: vec![("mission_id".into(), "mission_7".into())],
        lineage_refs: vec!["artifact_parent_1".into()],
        routing_tags: vec![("size".into(), "64x64".into())],
    };

    match saved.target {
        FileSavedTarget::SingleFile { relative_path } => {
            assert_eq!(relative_path, "B/L4/as-is/thumbnail/64x64/hash.png");
        }
        _ => panic!("expected single-file target"),
    }
    assert_eq!(saved.correlation_ids[0].0, "mission_id");
}

#[test]
fn append_data_requires_full_metadata_and_qos_policy() {
    let request = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new("A"),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![("mission".into(), "mission_7".into())],
        },
        metadata: StorageRequestMetadata::default(),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: deer_foundation_contracts::StoragePayloadDescriptor::InlineBytes { bytes: b"hello".to_vec() },
    };

    assert!(request.validate().is_err());
}

#[test]
fn append_control_is_separate_from_append_data() {
    let request = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: vec!["as_is_hash_xyz".into()],
        metadata: StorageRequestMetadata::default(),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::ConflictIntent),
        rationale: Some("user_request".into()),
    };

    assert_eq!(request.target_refs.len(), 1);
    assert_eq!(request.qos.template, StorageQosTemplate::ConflictIntent);
}

#[test]
fn derivation_trigger_reuses_full_saved_context() {
    let trigger = DerivationTrigger {
        logical_write_id: LogicalWriteId::new("write_123"),
        relative_target: "B/L3/chunks/transcript/date/hash.parquet".into(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L3,
        plane: CanonicalPlane::Chunks,
        payload_kind: StoragePayloadKind::new("transcript"),
        format: StoragePayloadFormat::new("parquet"),
        correlation_ids: vec![("mission_id".into(), "mission_7".into())],
        lineage_refs: vec!["parent_1".into()],
        routing_tags: vec![("date".into(), "2026-04-04".into())],
    };

    assert_eq!(trigger.relative_target, "B/L3/chunks/transcript/date/hash.parquet");
}

#[test]
fn qos_policy_supports_template_plus_overrides() {
    let mut qos = StorageQosPolicy::from_template(StorageQosTemplate::DurableArtifact);
    qos.worker_priority = "burst-high".into();

    assert_eq!(qos.template, StorageQosTemplate::DurableArtifact);
    assert_eq!(qos.worker_priority, "burst-high");
}
```

- [ ] **Step 2: Run the contract tests to verify they fail**

Run: `cargo test -p deer-foundation-contracts --test storage_contracts -v`

Expected: FAIL with missing storage contract module, missing `FileAccepted`/`FileSaved`, missing `append_control`, and missing QoS policy types.

- [ ] **Step 3: Implement the minimal shared contract module without breaking existing exports**

```rust
// crates/foundation/contracts/src/storage.rs
use serde::{Deserialize, Serialize};

use crate::{CanonicalLevel, CanonicalPlane};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalWriteId(String);

impl LogicalWriteId {
    pub fn new(value: &str) -> Self { Self(value.into()) }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: &str) -> Self { Self(value.into()) }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageHierarchyTag(String);

impl StorageHierarchyTag {
    pub fn new(value: &str) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePayloadKind(String);

impl StoragePayloadKind {
    pub fn new(value: &str) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePayloadFormat(String);

impl StoragePayloadFormat {
    pub fn new(value: &str) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn extension(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageLayout {
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub partition_tags: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StorageRequestMetadata {
    pub logical_write_id: Option<LogicalWriteId>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub writer_identity: Option<String>,
    pub known_content_hash: Option<String>,
    pub correlation_ids: Vec<(String, String)>,
    pub lineage_refs: Vec<String>,
    pub opaque_annotations: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoragePayloadDescriptor {
    InlineBytes { bytes: Vec<u8> },
    ExternalRef { uri: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageQosTemplate {
    FireAndForget,
    DurableArtifact,
    RealtimeStream,
    ConflictIntent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageQosPolicy {
    pub template: StorageQosTemplate,
    pub durability_class: String,
    pub progress_emission: String,
    pub worker_priority: String,
}

impl StorageQosPolicy {
    pub fn from_template(template: StorageQosTemplate) -> Self {
        match template {
            StorageQosTemplate::FireAndForget => Self {
                template,
                durability_class: "balanced".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "normal".into(),
            },
            StorageQosTemplate::DurableArtifact => Self {
                template,
                durability_class: "strong".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "high".into(),
            },
            StorageQosTemplate::RealtimeStream => Self {
                template,
                durability_class: "latency-biased".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "realtime".into(),
            },
            StorageQosTemplate::ConflictIntent => Self {
                template,
                durability_class: "strong".into(),
                progress_emission: "internal-only".into(),
                worker_priority: "high".into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendDataRequest {
    pub layout: StorageLayout,
    pub metadata: StorageRequestMetadata,
    pub qos: StorageQosPolicy,
    pub payload: StoragePayloadDescriptor,
}

impl AppendDataRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.metadata.idempotency_key.is_none() {
            return Err("missing idempotency key");
        }
        if self.metadata.correlation_ids.is_empty() {
            return Err("missing correlation ids");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlIntentKind {
    Exclusion,
    Supersession,
    Reprocess,
    Invalidation,
    Backfill,
    OverrideRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendControlRequest {
    pub control_kind: ControlIntentKind,
    pub target_refs: Vec<String>,
    pub metadata: StorageRequestMetadata,
    pub qos: StorageQosPolicy,
    pub rationale: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSavedTarget {
    SingleFile { relative_path: String },
    CommitManifest { manifest_path: String, member_count: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileAccepted {
    pub logical_write_id: LogicalWriteId,
    pub idempotency_key: IdempotencyKey,
    pub topic_class: String,
    pub routing_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSaved {
    pub logical_write_id: LogicalWriteId,
    pub target: FileSavedTarget,
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub correlation_ids: Vec<(String, String)>,
    pub lineage_refs: Vec<String>,
    pub routing_tags: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationTrigger {
    pub logical_write_id: LogicalWriteId,
    pub relative_target: String,
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub correlation_ids: Vec<(String, String)>,
    pub lineage_refs: Vec<String>,
    pub routing_tags: Vec<(String, String)>,
}
```

```rust
// crates/foundation/contracts/src/lib.rs
pub mod commands;
pub mod events;
pub mod ids;
pub mod meta;
pub mod records;
pub mod storage;

pub use commands::*;
pub use events::*;
pub use ids::*;
pub use meta::*;
pub use records::*;
pub use storage::*;
```

- [ ] **Step 4: Run the contract tests to verify they pass**

Run: `cargo test -p deer-foundation-contracts --test storage_contracts -v`

Expected: PASS with full `append_data`, `append_control`, `FileAccepted`, `FileSaved`, and derivation-trigger contracts verified.

- [ ] **Step 5: Commit the shared storage contracts**

```bash
git add Cargo.toml crates/foundation/contracts
git commit -m "feat: add shared storage contracts"
```
