## Task 0: Define The Shared Generator Storage Contract And Promotion Policy Before Any Adapter Work

**Files:**
- Create: `crates/pipeline/raw_sources/src/source_catalog.rs`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Test: `crates/pipeline/raw_sources/tests/generator_source_catalog.rs`
- Create: `crates/pipeline/normalizers/src/promotion_policy.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/generator_promotion_policy.rs`

**Milestone unlock:** every backend generator storage object can bind to a shared immutable source contract, and every level/plane transition from `L0` through `L2` is governed by named promotion rules instead of ad hoc generator-specific normalization logic

**Forbidden shortcuts:** do not infer storage classes only from runtime code paths; do not let normalizers guess record families without a source catalog entry; do not introduce game-facing projections before the storage/promotion matrix is explicit; do not encode level/plane policy only in prose

- [ ] **Step 1: Write the failing source catalog and promotion policy tests**

```rust
// crates/pipeline/raw_sources/tests/generator_source_catalog.rs
use deer_pipeline_raw_sources::{
    bind_generator_source_kind, GeneratorProfile, GeneratorSourceKind, SourcePlanePolicy,
    SourceStorageClass,
};

#[test]
fn shared_source_catalog_classifies_generator_agnostic_source_kinds() {
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "thread_snapshot"),
        Some(GeneratorSourceKind::SessionSnapshot)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "live_message"),
        Some(GeneratorSourceKind::MessageEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "task_progress"),
        Some(GeneratorSourceKind::TaskEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "artifact_presented"),
        Some(GeneratorSourceKind::ArtifactEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "runtime_status"),
        Some(GeneratorSourceKind::RuntimeEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "operator_intent"),
        Some(GeneratorSourceKind::IntentEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "exclusion_intent"),
        Some(GeneratorSourceKind::ExclusionEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "storage_backpressure"),
        Some(GeneratorSourceKind::BackpressureEvent)
    );
    assert_eq!(
        bind_generator_source_kind(GeneratorProfile::DeerFlow, "replay_checkpoint"),
        Some(GeneratorSourceKind::ReplayCheckpointEvent)
    );
}

#[test]
fn shared_source_catalog_exposes_storage_and_plane_policy() {
    let artifact = GeneratorSourceKind::ArtifactEvent.spec();
    assert_eq!(artifact.storage_class, SourceStorageClass::StorageNative);
    assert_eq!(artifact.default_plane, SourcePlanePolicy::AsIsRequired);
    assert!(artifact.requires_as_is_hash);

    let backpressure = GeneratorSourceKind::BackpressureEvent.spec();
    assert_eq!(backpressure.storage_class, SourceStorageClass::OperationalLog);
    assert_eq!(backpressure.default_plane, SourcePlanePolicy::AsIsRequired);
}
```

```rust
// crates/pipeline/normalizers/tests/generator_promotion_policy.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use deer_pipeline_normalizers::{promotion_rule_for, PromotionStage};
use deer_pipeline_raw_sources::{GeneratorProfile, GeneratorSourceKind};

#[test]
fn promotion_policy_explicitly_maps_generator_source_kinds_to_canonical_targets() {
    let artifact_rule = promotion_rule_for(GeneratorProfile::DeerFlow, GeneratorSourceKind::ArtifactEvent).unwrap();
    assert_eq!(artifact_rule.stage, PromotionStage::Canonical);
    assert_eq!(artifact_rule.target_family, RecordFamily::Artifact);
    assert_eq!(artifact_rule.level, CanonicalLevel::L2);
    assert_eq!(artifact_rule.plane, CanonicalPlane::AsIs);
    assert_eq!(artifact_rule.storage, StorageDisposition::StorageNative);

    let exclusion_rule = promotion_rule_for(GeneratorProfile::DeerFlow, GeneratorSourceKind::ExclusionEvent).unwrap();
    assert_eq!(exclusion_rule.target_family, RecordFamily::Exclusion);
    assert_eq!(exclusion_rule.level, CanonicalLevel::L2);

    let checkpoint_rule = promotion_rule_for(GeneratorProfile::DeerFlow, GeneratorSourceKind::ReplayCheckpointEvent).unwrap();
    assert_eq!(checkpoint_rule.target_family, RecordFamily::ReplayCheckpoint);
    assert_eq!(checkpoint_rule.level, CanonicalLevel::L2);
}

#[test]
fn promotion_policy_requires_l0_and_l1_before_l2_for_observed_objects() {
    let task_progress_rule = promotion_rule_for(GeneratorProfile::DeerFlow, GeneratorSourceKind::TaskEvent).unwrap();
    assert!(task_progress_rule.requires_l0_capture);
    assert!(task_progress_rule.requires_l1_sanitization);
    assert_eq!(task_progress_rule.level, CanonicalLevel::L2);
}
```

- [ ] **Step 2: Run the source catalog and promotion policy tests and confirm they fail**

Run: `cargo test -p deer-pipeline-raw-sources --test generator_source_catalog -v && cargo test -p deer-pipeline-normalizers --test generator_promotion_policy -v`

Expected: FAIL with missing shared source catalog APIs, missing generator profiles/source kinds, and missing promotion policy exports.

- [ ] **Step 3: Implement the source catalog**

```rust
// crates/pipeline/raw_sources/src/source_catalog.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratorProfile {
    DeerFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratorSourceKind {
    SessionSnapshot,
    MessageEvent,
    TaskEvent,
    ArtifactEvent,
    RuntimeEvent,
    IntentEvent,
    ExclusionEvent,
    BackpressureEvent,
    ReplayCheckpointEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceStorageClass {
    StorageNative,
    OperationalLog,
    MediatedReadModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourcePlanePolicy {
    AsIsRequired,
    ChunksAllowed,
    EmbeddingsAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratorSourceSpec {
    pub storage_class: SourceStorageClass,
    pub default_plane: SourcePlanePolicy,
    pub requires_as_is_hash: bool,
}

impl GeneratorSourceKind {
    pub fn spec(self) -> GeneratorSourceSpec {
        match self {
            Self::SessionSnapshot => GeneratorSourceSpec {
                storage_class: SourceStorageClass::MediatedReadModel,
                default_plane: SourcePlanePolicy::AsIsRequired,
                requires_as_is_hash: false,
            },
            Self::MessageEvent | Self::TaskEvent | Self::ArtifactEvent => GeneratorSourceSpec {
                storage_class: SourceStorageClass::StorageNative,
                default_plane: SourcePlanePolicy::AsIsRequired,
                requires_as_is_hash: true,
            },
            Self::RuntimeEvent | Self::BackpressureEvent | Self::ReplayCheckpointEvent => GeneratorSourceSpec {
                storage_class: SourceStorageClass::OperationalLog,
                default_plane: SourcePlanePolicy::AsIsRequired,
                requires_as_is_hash: false,
            },
            Self::IntentEvent | Self::ExclusionEvent => GeneratorSourceSpec {
                storage_class: SourceStorageClass::StorageNative,
                default_plane: SourcePlanePolicy::AsIsRequired,
                requires_as_is_hash: true,
            },
        }
    }
}

pub fn bind_generator_source_kind(
    profile: GeneratorProfile,
    shape: &str,
) -> Option<GeneratorSourceKind> {
    match (profile, shape) {
        (GeneratorProfile::DeerFlow, "thread_snapshot") => Some(GeneratorSourceKind::SessionSnapshot),
        (GeneratorProfile::DeerFlow, "live_message") => Some(GeneratorSourceKind::MessageEvent),
        (GeneratorProfile::DeerFlow, "task_progress") => Some(GeneratorSourceKind::TaskEvent),
        (GeneratorProfile::DeerFlow, "artifact_presented") => Some(GeneratorSourceKind::ArtifactEvent),
        (GeneratorProfile::DeerFlow, "runtime_status") => Some(GeneratorSourceKind::RuntimeEvent),
        (GeneratorProfile::DeerFlow, "operator_intent") => Some(GeneratorSourceKind::IntentEvent),
        (GeneratorProfile::DeerFlow, "exclusion_intent") => Some(GeneratorSourceKind::ExclusionEvent),
        (GeneratorProfile::DeerFlow, "storage_backpressure") => Some(GeneratorSourceKind::BackpressureEvent),
        (GeneratorProfile::DeerFlow, "replay_checkpoint") => Some(GeneratorSourceKind::ReplayCheckpointEvent),
        _ => None,
    }
}
```

```rust
// crates/pipeline/raw_sources/src/lib.rs
pub mod source_catalog;

pub use source_catalog::{
    bind_generator_source_kind, GeneratorProfile, GeneratorSourceKind, GeneratorSourceSpec,
    SourcePlanePolicy, SourceStorageClass,
};
```

- [ ] **Step 4: Implement the promotion policy**

```rust
// crates/pipeline/normalizers/src/promotion_policy.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use deer_pipeline_raw_sources::{GeneratorProfile, GeneratorSourceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionStage {
    Capture,
    Sanitization,
    Canonical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotionRule {
    pub stage: PromotionStage,
    pub target_family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub requires_l0_capture: bool,
    pub requires_l1_sanitization: bool,
}

pub fn promotion_rule_for(
    profile: GeneratorProfile,
    source_kind: GeneratorSourceKind,
) -> Option<PromotionRule> {
    let rule = match (profile, source_kind) {
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::SessionSnapshot) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Session,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::MessageEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Message,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::TaskEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Task,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::ArtifactEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Artifact,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::RuntimeEvent)
        | (GeneratorProfile::DeerFlow, GeneratorSourceKind::BackpressureEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::RuntimeStatus,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::IndexProjection,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::IntentEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Intent,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::ExclusionEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::Exclusion,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::StorageNative,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
        (GeneratorProfile::DeerFlow, GeneratorSourceKind::ReplayCheckpointEvent) => PromotionRule {
            stage: PromotionStage::Canonical,
            target_family: RecordFamily::ReplayCheckpoint,
            level: CanonicalLevel::L2,
            plane: CanonicalPlane::AsIs,
            storage: StorageDisposition::IndexProjection,
            requires_l0_capture: true,
            requires_l1_sanitization: true,
        },
    };

    Some(rule)
}
```

```rust
// crates/pipeline/normalizers/src/lib.rs
pub mod promotion_policy;

pub use promotion_policy::{promotion_rule_for, PromotionRule, PromotionStage};
```

- [ ] **Step 5: Re-run the source catalog and promotion policy tests**

Run: `cargo test -p deer-pipeline-raw-sources --test generator_source_catalog -v && cargo test -p deer-pipeline-normalizers --test generator_promotion_policy -v`

Expected: PASS with explicit shared source classifications and named promotion rules covering the shared generator contract plus DeerFlow's first binding.

- [ ] **Step 6: Commit the storage contract and promotion policy**

```bash
git add crates/pipeline/raw_sources/src/source_catalog.rs crates/pipeline/raw_sources/src/lib.rs crates/pipeline/raw_sources/tests/generator_source_catalog.rs crates/pipeline/normalizers/src/promotion_policy.rs crates/pipeline/normalizers/src/lib.rs crates/pipeline/normalizers/tests/generator_promotion_policy.rs
git commit -m "feat: define shared generator storage contract"
```
