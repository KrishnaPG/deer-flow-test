## Task 1: Harden Foundation Metadata For Source-Level Generator Onboarding

**Files:**
- Modify: `crates/foundation/contracts/src/meta.rs`
- Modify: `crates/foundation/contracts/src/records.rs`
- Modify: `crates/foundation/domain/src/common.rs`
- Test: `crates/foundation/contracts/tests/source_metadata_contracts.rs`
- Test: `crates/foundation/domain/tests/source_aware_record_builders.rs`

**Milestone unlock:** foundation records can express source level/plane, storage flags, and agent-aware correlations required by state-server-aligned generator pipelines

**Forbidden shortcuts:** do not break existing `new(...)` constructors; do not push source metadata down into app code; do not encode level/plane only in ad hoc strings

- [ ] **Step 1: Write the failing contract tests**

```rust
// crates/foundation/contracts/tests/source_metadata_contracts.rs
use deer_foundation_contracts::{
    AgentId, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordFamily, RecordHeader, RecordId, StorageDisposition,
};

#[test]
fn record_header_tracks_source_level_plane_and_storage_flags() {
    let header = RecordHeader::new(
        RecordId::from_static("rec_1"),
        RecordFamily::L2View,
        CanonicalLevel::L2,
        CanonicalPlane::AsIs,
        StorageDisposition::StorageNative,
        IdentityMeta::hash_anchored(RecordId::from_static("rec_1"), None, None, None),
        CorrelationMeta {
            agent_id: Some(AgentId::from_static("agent_1")),
            ..Default::default()
        },
        LineageMeta::root(),
    );

    assert_eq!(header.source_level, CanonicalLevel::L2);
    assert_eq!(header.source_plane, CanonicalPlane::AsIs);
    assert!(header.is_persisted_truth);
    assert!(!header.is_index_projection);
    assert!(!header.is_client_transient);
    assert_eq!(header.correlations.agent_id.unwrap().as_str(), "agent_1");
}
```

```rust
// crates/foundation/domain/tests/source_aware_record_builders.rs
use deer_foundation_contracts::{
    AgentId, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordId,
};
use deer_foundation_domain::{MessageBody, MessageRecord};

#[test]
fn source_aware_record_builder_preserves_correlation_and_source_dimensions() {
    let record = MessageRecord::new_with_meta(
        RecordId::from_static("msg_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("msg_1"), None, None, None),
        CorrelationMeta {
            agent_id: Some(AgentId::from_static("agent_1")),
            ..Default::default()
        },
        LineageMeta::root(),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        MessageBody {
            role: "assistant".into(),
            text: "Scouting ridge".into(),
        },
    );

    assert_eq!(record.header.level, CanonicalLevel::L2);
    assert_eq!(record.header.source_level, CanonicalLevel::L0);
    assert_eq!(record.header.source_plane, CanonicalPlane::AsIs);
    assert_eq!(
        record.header.correlations.agent_id.as_ref().map(|id| id.as_str()),
        Some("agent_1")
    );
}
```

- [ ] **Step 2: Run the targeted foundation tests and confirm they fail**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v && cargo test -p deer-foundation-domain --test source_aware_record_builders -v`

Expected: FAIL with missing `agent_id`, `source_level`, `source_plane`, storage flag fields, and missing `new_with_meta`.

- [ ] **Step 3: Implement the minimal contract and domain metadata extensions**

```rust
// crates/foundation/contracts/src/meta.rs
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMeta {
    pub mission_id: Option<MissionId>,
    pub run_id: Option<RunId>,
    pub artifact_id: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub thread_id: Option<ThreadId>,
    pub task_id: Option<TaskId>,
    pub agent_id: Option<AgentId>,
}
```

```rust
// crates/foundation/contracts/src/records.rs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub record_id: RecordId,
    pub family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub source_level: CanonicalLevel,
    pub source_plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub is_persisted_truth: bool,
    pub is_index_projection: bool,
    pub is_client_transient: bool,
    pub identity: IdentityMeta,
    pub correlations: CorrelationMeta,
    pub lineage: LineageMeta,
    pub observed_at: DateTime<Utc>,
}

impl RecordHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        record_id: RecordId,
        family: RecordFamily,
        level: CanonicalLevel,
        plane: CanonicalPlane,
        storage: StorageDisposition,
        identity: IdentityMeta,
        correlations: CorrelationMeta,
        lineage: LineageMeta,
    ) -> Self {
        Self::new_with_source(
            record_id,
            family,
            level,
            plane,
            level,
            plane,
            storage,
            identity,
            correlations,
            lineage,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_source(
        record_id: RecordId,
        family: RecordFamily,
        level: CanonicalLevel,
        plane: CanonicalPlane,
        source_level: CanonicalLevel,
        source_plane: CanonicalPlane,
        storage: StorageDisposition,
        identity: IdentityMeta,
        correlations: CorrelationMeta,
        lineage: LineageMeta,
    ) -> Self {
        Self {
            record_id,
            family,
            level,
            plane,
            source_level,
            source_plane,
            is_persisted_truth: matches!(storage, StorageDisposition::StorageNative),
            is_index_projection: matches!(storage, StorageDisposition::IndexProjection),
            is_client_transient: matches!(storage, StorageDisposition::ClientTransient),
            storage,
            identity,
            correlations,
            lineage,
            observed_at: Utc::now(),
        }
    }
}
```

```rust
// crates/foundation/domain/src/common.rs
pub fn build_header(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    build_header_with_meta(
        record_id,
        family,
        level,
        plane,
        level,
        plane,
        storage,
        identity,
        CorrelationMeta::default(),
        lineage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_header_with_meta(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    source_level: CanonicalLevel,
    source_plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    correlations: CorrelationMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    RecordHeader::new_with_source(
        record_id,
        family,
        level,
        plane,
        source_level,
        source_plane,
        storage,
        identity,
        correlations,
        lineage,
    )
}

#[macro_export]
macro_rules! define_record {
    ($name:ident, $body:ty, $family:expr, $level:expr, $plane:expr, $storage:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub header: deer_foundation_contracts::RecordHeader,
            pub body: $body,
        }

        impl $name {
            pub fn new(
                record_id: deer_foundation_contracts::RecordId,
                identity: deer_foundation_contracts::IdentityMeta,
                lineage: deer_foundation_contracts::LineageMeta,
                body: $body,
            ) -> Self {
                Self {
                    header: crate::common::build_header(
                        record_id, $family, $level, $plane, $storage, identity, lineage,
                    ),
                    body,
                }
            }

            pub fn new_with_meta(
                record_id: deer_foundation_contracts::RecordId,
                identity: deer_foundation_contracts::IdentityMeta,
                correlations: deer_foundation_contracts::CorrelationMeta,
                lineage: deer_foundation_contracts::LineageMeta,
                source_level: deer_foundation_contracts::CanonicalLevel,
                source_plane: deer_foundation_contracts::CanonicalPlane,
                body: $body,
            ) -> Self {
                Self {
                    header: crate::common::build_header_with_meta(
                        record_id,
                        $family,
                        $level,
                        $plane,
                        source_level,
                        source_plane,
                        $storage,
                        identity,
                        correlations,
                        lineage,
                    ),
                    body,
                }
            }
        }
    };
}
```

- [ ] **Step 4: Re-run the targeted foundation tests**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v && cargo test -p deer-foundation-domain --test source_aware_record_builders -v`

Expected: PASS with source-level/source-plane metadata and agent-aware correlations preserved.

- [ ] **Step 5: Commit the foundation metadata hardening**

```bash
git add crates/foundation/contracts/src/meta.rs crates/foundation/contracts/src/records.rs crates/foundation/contracts/tests/source_metadata_contracts.rs crates/foundation/domain/src/common.rs crates/foundation/domain/tests/source_aware_record_builders.rs
git commit -m "feat: add source-aware record metadata"
```
