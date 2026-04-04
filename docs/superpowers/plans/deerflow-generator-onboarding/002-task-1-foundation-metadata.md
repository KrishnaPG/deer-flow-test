### Task 1: Harden Foundation Metadata

**Files:**
- Modify: `crates/foundation/contracts/src/meta.rs`
- Modify: `crates/foundation/contracts/src/records.rs`
- Modify: `crates/foundation/domain/src/common.rs`
- Test: `crates/foundation/contracts/tests/source_metadata_contracts.rs`
- Test: `crates/foundation/domain/tests/source_aware_record_builders.rs`

- [ ] **Step 1: Write the failing foundation metadata tests**

Create `crates/foundation/contracts/tests/source_metadata_contracts.rs`:

```rust
use deer_foundation_contracts::{
    AccessControlMeta, CorrelationMeta, IdentityMeta, JoinKeysMeta, LineageMeta, RecordFamily,
    RecordId, RecordRef, ViewRefreshMeta,
};

#[test]
fn join_keys_meta_exposes_immutable_keys() {
    let meta = JoinKeysMeta {
        generator_key: "gen_1".into(),
        storage_object_key: "obj_1".into(),
        session_key: Some("sess_1".into()),
        thread_key: Some("thread_1".into()),
        run_key: Some("run_1".into()),
        agent_key: Some("agent_1".into()),
        task_key: Some("task_1".into()),
        artifact_key: None,
        intent_key: None,
        exclusion_key: None,
    };
    assert_eq!(meta.generator_key, "gen_1");
    assert_eq!(meta.session_key, Some("sess_1".into()));
}

#[test]
fn lineage_meta_tracks_source_and_transforms() {
    let parent = RecordRef::new(RecordFamily::Session, RecordId::from("sess_1"));
    let meta = LineageMeta::derived_from(parent.clone());
    assert_eq!(meta.derived_from.len(), 1);
    assert_eq!(meta.derived_from[0].record_id.as_str(), "sess_1");
    assert!(meta.supersedes.is_none());
}

#[test]
fn lineage_meta_supports_supersedes() {
    let mut meta = LineageMeta::root();
    meta.supersedes = Some(RecordRef::new(RecordFamily::Task, RecordId::from("task_old")));
    assert!(meta.supersedes.is_some());
}

#[test]
fn correlation_meta_preserves_cross_family_anchors() {
    let meta = CorrelationMeta {
        mission_id: Some("mission_1".into()),
        run_id: Some("run_1".into()),
        artifact_id: Some("artifact_1".into()),
        trace_id: Some("trace_1".into()),
        thread_id: Some("thread_1".into()),
        task_id: Some("task_1".into()),
    };
    assert_eq!(meta.mission_id, Some("mission_1".into()));
    assert_eq!(meta.run_id, Some("run_1".into()));
}

#[test]
fn access_control_meta_expresses_abac_and_exclusions() {
    let meta = AccessControlMeta {
        abac_scope: "session".into(),
        policy_tags: vec!["classified".into()],
        exclusion_targets: vec!["artifact_2".into()],
        redaction_mode: "tombstone".into(),
        authorized_projection: vec!["c_l2_commander_sessions_v".into()],
    };
    assert_eq!(meta.abac_scope, "session");
    assert_eq!(meta.exclusion_targets.len(), 1);
}

#[test]
fn view_refresh_meta_declares_c_l2_dependencies() {
    let meta = ViewRefreshMeta {
        eligible_c_views: vec!["c_l2_commander_sessions_v".into()],
        refresh_mode: "incremental".into(),
        refresh_watermark: "observed_at".into(),
        refresh_group: "commander".into(),
        invalidated_by: Some("a_session_snapshot".into()),
        last_source_level: "L2".into(),
    };
    assert_eq!(meta.eligible_c_views.len(), 1);
    assert_eq!(meta.refresh_mode, "incremental");
}
```

Create `crates/foundation/domain/tests/source_aware_record_builders.rs`:

```rust
use deer_foundation_contracts::{
    AccessControlMeta, IdentityMeta, JoinKeysMeta, LineageMeta, RecordFamily, RecordId,
    ViewRefreshMeta,
};
use deer_foundation_domain::common::SourceAwareHeaderBuilder;

#[test]
fn default_builder_keeps_existing_behavior() {
    let record_id = RecordId::from("test_1");
    let identity = IdentityMeta::hash_anchored(record_id.clone(), None, None, None);
    let lineage = LineageMeta::root();

    let header = SourceAwareHeaderBuilder::default()
        .record_id(record_id.clone())
        .family(RecordFamily::Session)
        .identity(identity)
        .lineage(lineage)
        .build();

    assert_eq!(header.record_id.as_str(), "test_1");
}

#[test]
fn source_aware_builder_preserves_join_keys() {
    let record_id = RecordId::from("test_1");
    let identity = IdentityMeta::hash_anchored(record_id.clone(), None, None, None);
    let lineage = LineageMeta::root();
    let join_keys = JoinKeysMeta {
        generator_key: "gen_1".into(),
        storage_object_key: "obj_1".into(),
        session_key: Some("sess_1".into()),
        thread_key: Some("thread_1".into()),
        run_key: Some("run_1".into()),
        agent_key: None,
        task_key: None,
        artifact_key: None,
        intent_key: None,
        exclusion_key: None,
    };

    let header = SourceAwareHeaderBuilder::default()
        .record_id(record_id.clone())
        .family(RecordFamily::Session)
        .identity(identity)
        .lineage(lineage)
        .join_keys(join_keys.clone())
        .build();

    assert!(header.join_keys.is_some());
    assert_eq!(
        header.join_keys.as_ref().unwrap().session_key,
        Some("sess_1".into())
    );
}

#[test]
fn source_aware_builder_preserves_lineage_across_levels() {
    let record_id = RecordId::from("test_1");
    let identity = IdentityMeta::hash_anchored(record_id.clone(), None, None, None);
    let source_ref = deer_foundation_contracts::RecordRef::new(
        RecordFamily::Session,
        RecordId::from("sess_1"),
    );
    let lineage = LineageMeta::derived_from(source_ref);

    let header = SourceAwareHeaderBuilder::default()
        .record_id(record_id.clone())
        .family(RecordFamily::Task)
        .identity(identity)
        .lineage(lineage.clone())
        .build();

    assert_eq!(header.lineage.derived_from.len(), 1);
    assert_eq!(
        header.lineage.derived_from[0].record_id.as_str(),
        "sess_1"
    );
}

#[test]
fn source_aware_builder_attaches_refresh_metadata() {
    let record_id = RecordId::from("test_1");
    let identity = IdentityMeta::hash_anchored(record_id.clone(), None, None, None);
    let lineage = LineageMeta::root();
    let refresh = ViewRefreshMeta {
        eligible_c_views: vec!["c_l2_commander_sessions_v".into()],
        refresh_mode: "incremental".into(),
        refresh_watermark: "observed_at".into(),
        refresh_group: "commander".into(),
        invalidated_by: None,
        last_source_level: "L2".into(),
    };

    let header = SourceAwareHeaderBuilder::default()
        .record_id(record_id.clone())
        .family(RecordFamily::Session)
        .identity(identity)
        .lineage(lineage)
        .view_refresh(refresh.clone())
        .build();

    assert!(header.view_refresh.is_some());
    assert_eq!(
        header.view_refresh.as_ref().unwrap().eligible_c_views,
        vec!["c_l2_commander_sessions_v".into()]
    );
}

#[test]
fn source_aware_builder_attaches_abac_metadata() {
    let record_id = RecordId::from("test_1");
    let identity = IdentityMeta::hash_anchored(record_id.clone(), None, None, None);
    let lineage = LineageMeta::root();
    let abac = AccessControlMeta {
        abac_scope: "session".into(),
        policy_tags: vec!["internal".into()],
        exclusion_targets: vec![],
        redaction_mode: "hide".into(),
        authorized_projection: vec!["c_l2_commander_sessions_v".into()],
    };

    let header = SourceAwareHeaderBuilder::default()
        .record_id(record_id.clone())
        .family(RecordFamily::Session)
        .identity(identity)
        .lineage(lineage)
        .access_control(abac.clone())
        .build();

    assert!(header.access_control.is_some());
    assert_eq!(
        header.access_control.as_ref().unwrap().abac_scope,
        "session"
    );
}
```

- [ ] **Step 2: Run tests to confirm failure**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v`
Run: `cargo test -p deer-foundation-domain --test source_aware_record_builders -v`
Expected: FAIL — types don't exist yet

- [ ] **Step 3: Extend meta.rs with new metadata blocks**

Modify `crates/foundation/contracts/src/meta.rs` — add these structs after the existing ones:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinKeysMeta {
    pub generator_key: String,
    pub storage_object_key: String,
    pub session_key: Option<String>,
    pub thread_key: Option<String>,
    pub run_key: Option<String>,
    pub agent_key: Option<String>,
    pub task_key: Option<String>,
    pub artifact_key: Option<String>,
    pub intent_key: Option<String>,
    pub exclusion_key: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessControlMeta {
    pub abac_scope: String,
    pub policy_tags: Vec<String>,
    pub exclusion_targets: Vec<String>,
    pub redaction_mode: String,
    pub authorized_projection: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRefreshMeta {
    pub eligible_c_views: Vec<String>,
    pub refresh_mode: String,
    pub refresh_watermark: String,
    pub refresh_group: String,
    pub invalidated_by: Option<String>,
    pub last_source_level: String,
}
```

- [ ] **Step 4: Extend RecordHeader with new metadata fields**

Modify `crates/foundation/contracts/src/records.rs` — update RecordHeader:

```rust
use crate::{
    AccessControlMeta, CorrelationMeta, IdentityMeta, JoinKeysMeta, LineageMeta, RecordId,
    ViewRefreshMeta,
};
```

Update RecordHeader struct:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub record_id: RecordId,
    pub family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub identity: IdentityMeta,
    pub correlations: CorrelationMeta,
    pub lineage: LineageMeta,
    pub observed_at: DateTime<Utc>,
    pub join_keys: Option<JoinKeysMeta>,
    pub access_control: Option<AccessControlMeta>,
    pub view_refresh: Option<ViewRefreshMeta>,
}
```

- [ ] **Step 5: Add SourceAwareHeaderBuilder to common.rs**

Modify `crates/foundation/domain/src/common.rs` — add after existing code:

```rust
use deer_foundation_contracts::{
    AccessControlMeta, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta,
    JoinKeysMeta, LineageMeta, RecordFamily, RecordHeader, RecordId, StorageDisposition,
    ViewRefreshMeta,
};

pub struct SourceAwareHeaderBuilder {
    record_id: Option<RecordId>,
    family: Option<RecordFamily>,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: Option<IdentityMeta>,
    correlations: CorrelationMeta,
    lineage: Option<LineageMeta>,
    join_keys: Option<JoinKeysMeta>,
    access_control: Option<AccessControlMeta>,
    view_refresh: Option<ViewRefreshMeta>,
}

impl Default for SourceAwareHeaderBuilder {
    fn default() -> Self {
        Self {
            record_id: None,
            family: None,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            identity: None,
            correlations: CorrelationMeta::default(),
            lineage: None,
            join_keys: None,
            access_control: None,
            view_refresh: None,
        }
    }
}

impl SourceAwareHeaderBuilder {
    pub fn record_id(mut self, id: RecordId) -> Self {
        self.record_id = Some(id);
        self
    }

    pub fn family(mut self, family: RecordFamily) -> Self {
        self.family = Some(family);
        self
    }

    pub fn level(mut self, level: CanonicalLevel) -> Self {
        self.level = level;
        self
    }

    pub fn plane(mut self, plane: CanonicalPlane) -> Self {
        self.plane = plane;
        self
    }

    pub fn storage(mut self, storage: StorageDisposition) -> Self {
        self.storage = storage;
        self
    }

    pub fn identity(mut self, identity: IdentityMeta) -> Self {
        self.identity = Some(identity);
        self
    }

    pub fn correlations(mut self, correlations: CorrelationMeta) -> Self {
        self.correlations = correlations;
        self
    }

    pub fn lineage(mut self, lineage: LineageMeta) -> Self {
        self.lineage = Some(lineage);
        self
    }

    pub fn join_keys(mut self, join_keys: JoinKeysMeta) -> Self {
        self.join_keys = Some(join_keys);
        self
    }

    pub fn access_control(mut self, access_control: AccessControlMeta) -> Self {
        self.access_control = Some(access_control);
        self
    }

    pub fn view_refresh(mut self, view_refresh: ViewRefreshMeta) -> Self {
        self.view_refresh = Some(view_refresh);
        self
    }

    pub fn build(self) -> RecordHeader {
        RecordHeader {
            record_id: self.record_id.expect("record_id required"),
            family: self.family.expect("family required"),
            level: self.level,
            plane: self.plane,
            storage: self.storage,
            identity: self.identity.expect("identity required"),
            correlations: self.correlations,
            lineage: self.lineage.expect("lineage required"),
            observed_at: chrono::Utc::now(),
            join_keys: self.join_keys,
            access_control: self.access_control,
            view_refresh: self.view_refresh,
        }
    }
}
```

- [ ] **Step 6: Re-run tests to verify pass**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v`
Run: `cargo test -p deer-foundation-domain --test source_aware_record_builders -v`
Expected: PASS

---

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `foundation/contracts` (meta types), `foundation/domain` (builders) |
| 2 | Contract being defined | `JoinKeysMeta`, `AccessControlMeta`, `ViewRefreshMeta` on `RecordHeader`; `SourceAwareHeaderBuilder` |
| 3 | Failing test or fixture first | `source_metadata_contracts.rs` (6 tests) + `source_aware_record_builders.rs` (5 tests) |
| 4 | Proof app before `deer_gui` | None — pure foundation types, validated downstream by Tasks 2-6 |
| 5 | Forbidden dependencies | Pipeline crates, UI crates, generator-specific types |
| 6 | Milestone gate unlocked | Gate A — metadata needed for C:L2 joins, ABAC, and refresh |
