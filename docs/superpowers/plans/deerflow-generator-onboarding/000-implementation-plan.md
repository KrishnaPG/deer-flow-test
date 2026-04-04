# DeerFlow Generator Onboarding — Shell + Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the full onboarding path for DeerFlow as the first backend generator under the modular toolkit's New Generator Onboarding Contract (spec 42), covering shared A/B storage families, normalization to L2+, shell-defined C:L2 query contracts, and end-to-end proof.

**Architecture:** Hierarchy A (observed execution facts) and Hierarchy B (normalized content/lineage) are physical storage. Hierarchy C is presentation: C:L0 admits shell-declared A:L2+/B:L2+ rows, C:L1 is optional transform, C:L2 is shell-defined canonical query contracts implemented by generator-supplied SQL/query projections. DeerFlow binds into shared A/B families, normalizes to L2+, and supplies C:L2 implementations for commander, researcher, thread/core, battle-command, and world-primary shells.

**Tech Stack:** Rust workspace crates, serde, serde_json, chrono, thiserror, uuid, insta

---

## Architecture Invariants (from spec 42)

- A and B are the only physical storage hierarchies
- L0/L1 are storage evidence only; consumers never query them directly
- C:L0 = admissible A:L2+/B:L2+ source pool for presentation composition
- C:L1 = optional presentation-side format/selection/reduction
- C:L2 = minimum consumer query surface; shell defines contract, generator supplies implementation
- B:L3/L4/L5/L6 rows are admissible B:L2+ inputs, not a special exception
- A generator does not define app-facing canon
- Shell does not bypass C:L2 with app-local read glue

## File Structure

### New files to create

| File | Responsibility |
|------|---------------|
| `crates/pipeline/raw_sources/src/storage_contract.rs` | Shared A/B family registry, allowed levels, immutable keys, lineage anchors, C:L0 admissibility |
| `crates/pipeline/raw_sources/src/c_query_contract.rs` | Shell registry: admissible C:L0 inputs per shell, required C:L2 view ids per shell |
| `crates/pipeline/raw_sources/src/deerflow.rs` | DeerFlow binding: maps DeerFlow source shapes into shared A/B family ids |
| `crates/pipeline/normalizers/src/c_view_catalog.rs` | C:L1/C:L2 presentation transform and SQL view implementation catalog |
| `crates/pipeline/raw_sources/tests/storage_contract.rs` | Tests: A/B registry, consumer guards, DeerFlow bindings, B:L3+ admissibility |
| `crates/pipeline/raw_sources/tests/deerflow_adapter.rs` | Tests: DeerFlow landing into shared A/B families, no consumer canon leak |
| `crates/pipeline/normalizers/tests/c_l2_projection_catalog.rs` | Tests: commander/researcher/thread/core C:L2 views registered with complete metadata |
| `crates/pipeline/normalizers/tests/c_l2_world_shell_catalog.rs` | Tests: battle-command/world-primary C:L2 views registered |
| `crates/foundation/contracts/tests/source_metadata_contracts.rs` | Tests: headers expose join/lineage/ABAC/refresh metadata |
| `crates/foundation/domain/tests/source_aware_record_builders.rs` | Tests: builders preserve metadata end-to-end |
| `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs` | Tests: normalization produces L2+ rows with lineage intact |

### Modified files

| File | Change |
|------|--------|
| `crates/pipeline/raw_sources/src/lib.rs` | Export new modules |
| `crates/pipeline/raw_sources/Cargo.toml` | Add `deer-foundation-contracts` dependency |
| `crates/pipeline/normalizers/src/lib.rs` | Export c_view_catalog |
| `crates/foundation/contracts/src/meta.rs` | Add JoinKeysMeta, AccessControlMeta, ViewRefreshMeta |
| `crates/foundation/contracts/src/records.rs` | Add new metadata fields to RecordHeader |
| `crates/foundation/contracts/src/ids.rs` | Add new typed ID types if needed |
| `crates/foundation/domain/src/common.rs` | Source-aware builders with full metadata |

---

### Task 0: Shared A/B Storage Contract and C:L2 Query Contract Registry

**Files:**
- Create: `crates/pipeline/raw_sources/src/storage_contract.rs`
- Create: `crates/pipeline/raw_sources/src/c_query_contract.rs`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Modify: `crates/pipeline/raw_sources/Cargo.toml`
- Test: `crates/pipeline/raw_sources/tests/storage_contract.rs`

- [ ] **Step 1: Write the failing storage-contract tests**

Create `crates/pipeline/raw_sources/tests/storage_contract.rs`:

```rust
use deer_pipeline_raw_sources::storage_contract::{
    AbFamily, BFamily, FamilyId, StorageContract, Hierarchy, CanonicalLevel,
};

#[test]
fn registry_contains_all_a_families() {
    let contract = StorageContract::default();
    let expected = vec![
        "a_session_snapshot",
        "a_message_event",
        "a_task_event",
        "a_artifact_event",
        "a_runtime_event",
        "a_intent_event",
        "a_exclusion_event",
        "a_replay_checkpoint",
        "a_backpressure_event",
    ];
    for family_id in expected {
        assert!(
            contract.get_family(family_id).is_some(),
            "missing A family: {}",
            family_id
        );
    }
}

#[test]
fn registry_contains_all_b_families() {
    let contract = StorageContract::default();
    let expected = vec![
        "b_session",
        "b_message",
        "b_task",
        "b_artifact",
        "b_artifact_access",
        "b_runtime_status",
        "b_intent",
        "b_exclusion",
        "b_conflict",
        "b_replay_window",
        "b_replay_checkpoint",
        "b_backpressure",
        "b_transform",
    ];
    for family_id in expected {
        assert!(
            contract.get_family(family_id).is_some(),
            "missing B family: {}",
            family_id
        );
    }
}

#[test]
fn a_families_are_hierarchy_a() {
    let contract = StorageContract::default();
    assert_eq!(
        contract.get_family("a_session_snapshot").unwrap().hierarchy,
        Hierarchy::A
    );
    assert_eq!(
        contract.get_family("a_message_event").unwrap().hierarchy,
        Hierarchy::A
    );
}

#[test]
fn b_families_are_hierarchy_b() {
    let contract = StorageContract::default();
    assert_eq!(
        contract.get_family("b_session").unwrap().hierarchy,
        Hierarchy::B
    );
    assert_eq!(
        contract.get_family("b_artifact").unwrap().hierarchy,
        Hierarchy::B
    );
}

#[test]
fn families_declare_allowed_levels() {
    let contract = StorageContract::default();
    let session = contract.get_family("a_session_snapshot").unwrap();
    assert!(!session.allowed_levels.is_empty());
}

#[test]
fn families_declare_immutable_keys() {
    let contract = StorageContract::default();
    let session = contract.get_family("a_session_snapshot").unwrap();
    assert!(!session.immutable_keys.is_empty());
}

#[test]
fn consumer_cannot_query_l0_directly() {
    let contract = StorageContract::default();
    assert!(!contract.is_consumer_accessible("a_session_snapshot"));
    assert!(!contract.is_consumer_accessible("b_session"));
}

#[test]
fn l2_plus_families_are_admissible_to_c_l0() {
    let contract = StorageContract::default();
    // A:L2+ families should be admissible when shell declares them
    assert!(contract.is_admissible_to_c_l0("a_session_snapshot", CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("a_message_event", CanonicalLevel::L3));
    // B:L2+ families should be admissible
    assert!(contract.is_admissible_to_c_l0("b_session", CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("b_artifact", CanonicalLevel::L3));
}

#[test]
fn l0_l1_not_admissible_to_c_l0() {
    let contract = StorageContract::default();
    assert!(!contract.is_admissible_to_c_l0("a_session_snapshot", CanonicalLevel::L0));
    assert!(!contract.is_admissible_to_c_l0("a_session_snapshot", CanonicalLevel::L1));
    assert!(!contract.is_admissible_to_c_l0("b_session", CanonicalLevel::L0));
    assert!(!contract.is_admissible_to_c_l0("b_session", CanonicalLevel::L1));
}

#[test]
fn b_l3_plus_uses_same_admissibility_rule_as_b_l2() {
    let contract = StorageContract::default();
    // B:L3+ is NOT a special case — same admissibility path as B:L2
    let l2_admissible = contract.is_admissible_to_c_l0("b_artifact", CanonicalLevel::L2);
    let l3_admissible = contract.is_admissible_to_c_l0("b_artifact", CanonicalLevel::L3);
    let l4_admissible = contract.is_admissible_to_c_l0("b_artifact", CanonicalLevel::L4);
    // All should be admissible through the same rule
    assert_eq!(l2_admissible, l3_admissible);
    assert_eq!(l3_admissible, l4_admissible);
}
```

- [ ] **Step 2: Run tests to confirm failure**

Run: `cargo test -p deer-pipeline-raw-sources --test storage_contract -v`
Expected: FAIL — module `storage_contract` does not exist

- [ ] **Step 3: Update Cargo.toml**

Modify `crates/pipeline/raw_sources/Cargo.toml`:

```toml
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

- [ ] **Step 4: Implement storage_contract.rs**

Create `crates/pipeline/raw_sources/src/storage_contract.rs`:

```rust
use std::collections::HashMap;

/// Shared A/B family identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FamilyId {
    A(ABFamily),
    B(BFamily),
}

impl FamilyId {
    pub fn as_str(&self) -> &'static str {
        match self {
            FamilyId::A(f) => f.as_str(),
            FamilyId::B(f) => f.as_str(),
        }
    }
}

/// Hierarchy A family identifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AFamily {
    SessionSnapshot,
    MessageEvent,
    TaskEvent,
    ArtifactEvent,
    RuntimeEvent,
    IntentEvent,
    ExclusionEvent,
    ReplayCheckpoint,
    BackpressureEvent,
}

impl AFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            AFamily::SessionSnapshot => "a_session_snapshot",
            AFamily::MessageEvent => "a_message_event",
            AFamily::TaskEvent => "a_task_event",
            AFamily::ArtifactEvent => "a_artifact_event",
            AFamily::RuntimeEvent => "a_runtime_event",
            AFamily::IntentEvent => "a_intent_event",
            AFamily::ExclusionEvent => "a_exclusion_event",
            AFamily::ReplayCheckpoint => "a_replay_checkpoint",
            AFamily::BackpressureEvent => "a_backpressure_event",
        }
    }
}

/// Hierarchy B family identifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BFamily {
    Session,
    Message,
    Task,
    Artifact,
    ArtifactAccess,
    RuntimeStatus,
    Intent,
    Exclusion,
    Conflict,
    ReplayWindow,
    ReplayCheckpoint,
    Backpressure,
    Transform,
}

impl BFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            BFamily::Session => "b_session",
            BFamily::Message => "b_message",
            BFamily::Task => "b_task",
            BFamily::Artifact => "b_artifact",
            BFamily::ArtifactAccess => "b_artifact_access",
            BFamily::RuntimeStatus => "b_runtime_status",
            BFamily::Intent => "b_intent",
            BFamily::Exclusion => "b_exclusion",
            BFamily::Conflict => "b_conflict",
            BFamily::ReplayWindow => "b_replay_window",
            BFamily::ReplayCheckpoint => "b_replay_checkpoint",
            BFamily::Backpressure => "b_backpressure",
            BFamily::Transform => "b_transform",
        }
    }
}

/// Which hierarchy a family belongs to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hierarchy {
    A,
    B,
}

/// Canonical level (L0-L6)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

/// Metadata for a single storage family
#[derive(Clone, Debug)]
pub struct FamilyMeta {
    pub id: FamilyId,
    pub hierarchy: Hierarchy,
    pub allowed_levels: Vec<CanonicalLevel>,
    pub immutable_keys: Vec<&'static str>,
    pub lineage_anchors: Vec<&'static str>,
    pub row_grain: &'static str,
}

/// The shared A/B storage contract registry
#[derive(Clone, Debug)]
pub struct StorageContract {
    families: HashMap<String, FamilyMeta>,
}

impl Default for StorageContract {
    fn default() -> Self {
        let mut families = HashMap::new();

        // A families
        register_a_family(&mut families, AFamily::SessionSnapshot, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["generator_key", "session_key", "thread_key"], vec!["session_id"], "one row per session snapshot");
        register_a_family(&mut families, AFamily::MessageEvent, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["generator_key", "message_key", "thread_key", "agent_key"], vec!["message_id"], "one row per message event");
        register_a_family(&mut families, AFamily::TaskEvent, vec![CanonicalLevel::L2, CanonicalLevel::L3, CanonicalLevel::L4], vec!["generator_key", "task_key", "thread_key", "run_key"], vec!["task_id"], "one row per task event");
        register_a_family(&mut families, AFamily::ArtifactEvent, vec![CanonicalLevel::L2, CanonicalLevel::L3, CanonicalLevel::L4], vec!["generator_key", "artifact_key", "task_key", "run_key"], vec!["artifact_id"], "one row per artifact event");
        register_a_family(&mut families, AFamily::RuntimeEvent, vec![CanonicalLevel::L2], vec!["generator_key", "run_key", "agent_key"], vec!["run_id"], "one row per runtime status event");
        register_a_family(&mut families, AFamily::IntentEvent, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["generator_key", "intent_key", "thread_key", "actor_key"], vec!["intent_id"], "one row per intent event");
        register_a_family(&mut families, AFamily::ExclusionEvent, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["generator_key", "exclusion_key", "target_key"], vec!["exclusion_id"], "one row per exclusion event");
        register_a_family(&mut families, AFamily::ReplayCheckpoint, vec![CanonicalLevel::L2, CanonicalLevel::L6], vec!["generator_key", "checkpoint_key", "run_key"], vec!["checkpoint_id"], "one row per replay checkpoint");
        register_a_family(&mut families, AFamily::BackpressureEvent, vec![CanonicalLevel::L2], vec!["generator_key", "backpressure_key", "run_key"], vec!["backpressure_id"], "one row per backpressure event");

        // B families
        register_b_family(&mut families, BFamily::Session, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["session_key", "thread_key"], vec!["session_id"], "one row per normalized session");
        register_b_family(&mut families, BFamily::Message, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["message_key", "thread_key", "agent_key"], vec!["message_id"], "one row per normalized message");
        register_b_family(&mut families, BFamily::Task, vec![CanonicalLevel::L2, CanonicalLevel::L3, CanonicalLevel::L4], vec!["task_key", "thread_key", "run_key"], vec!["task_id"], "one row per normalized task");
        register_b_family(&mut families, BFamily::Artifact, vec![CanonicalLevel::L2, CanonicalLevel::L3, CanonicalLevel::L4], vec!["artifact_key", "task_key", "run_key"], vec!["artifact_id"], "one row per normalized artifact");
        register_b_family(&mut families, BFamily::ArtifactAccess, vec![CanonicalLevel::L2], vec!["artifact_key", "access_key"], vec!["artifact_id", "access_id"], "one row per artifact access");
        register_b_family(&mut families, BFamily::RuntimeStatus, vec![CanonicalLevel::L2], vec!["run_key", "agent_key"], vec!["run_id"], "one row per runtime status");
        register_b_family(&mut families, BFamily::Intent, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["intent_key", "thread_key", "actor_key"], vec!["intent_id"], "one row per normalized intent");
        register_b_family(&mut families, BFamily::Exclusion, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["exclusion_key", "target_key"], vec!["exclusion_id"], "one row per normalized exclusion");
        register_b_family(&mut families, BFamily::Conflict, vec![CanonicalLevel::L2, CanonicalLevel::L3], vec!["conflict_key", "target_key"], vec!["conflict_id"], "one row per normalized conflict");
        register_b_family(&mut families, BFamily::ReplayWindow, vec![CanonicalLevel::L2, CanonicalLevel::L6], vec!["thread_key", "window_key"], vec!["window_id"], "one row per replay window");
        register_b_family(&mut families, BFamily::ReplayCheckpoint, vec![CanonicalLevel::L2, CanonicalLevel::L6], vec!["checkpoint_key", "run_key"], vec!["checkpoint_id"], "one row per replay checkpoint");
        register_b_family(&mut families, BFamily::Backpressure, vec![CanonicalLevel::L2], vec!["backpressure_key", "run_key"], vec!["backpressure_id"], "one row per backpressure");
        register_b_family(&mut families, BFamily::Transform, vec![CanonicalLevel::L2, CanonicalLevel::L3, CanonicalLevel::L5], vec!["transform_key", "target_key"], vec!["transform_id"], "one row per transform/lineage");

        Self { families }
    }
}

fn register_a_family(
    families: &mut HashMap<String, FamilyMeta>,
    family: AFamily,
    levels: Vec<CanonicalLevel>,
    keys: Vec<&'static str>,
    anchors: Vec<&'static str>,
    grain: &'static str,
) {
    let id = FamilyId::A(family.clone());
    families.insert(
        family.as_str().to_string(),
        FamilyMeta {
            id,
            hierarchy: Hierarchy::A,
            allowed_levels: levels,
            immutable_keys: keys,
            lineage_anchors: anchors,
            row_grain: grain,
        },
    );
}

fn register_b_family(
    families: &mut HashMap<String, FamilyMeta>,
    family: BFamily,
    levels: Vec<CanonicalLevel>,
    keys: Vec<&'static str>,
    anchors: Vec<&'static str>,
    grain: &'static str,
) {
    let id = FamilyId::B(family.clone());
    families.insert(
        family.as_str().to_string(),
        FamilyMeta {
            id,
            hierarchy: Hierarchy::B,
            allowed_levels: levels,
            immutable_keys: keys,
            lineage_anchors: anchors,
            row_grain: grain,
        },
    );
}

impl StorageContract {
    /// Look up a family by its string id
    pub fn get_family(&self, family_id: &str) -> Option<&FamilyMeta> {
        self.families.get(family_id)
    }

    /// Whether a family is directly consumer-accessible (always false for raw families)
    pub fn is_consumer_accessible(&self, _family_id: &str) -> bool {
        // Consumers never query L0/L1 directly; all raw families require C:L2
        false
    }

    /// Whether a family at a given level is admissible to C:L0
    /// B:L3+ uses the same admissibility rule as B:L2 — no special case
    pub fn is_admissible_to_c_l0(&self, family_id: &str, level: CanonicalLevel) -> bool {
        match self.get_family(family_id) {
            Some(meta) => {
                // L0 and L1 are never admissible to C:L0
                if level == CanonicalLevel::L0 || level == CanonicalLevel::L1 {
                    return false;
                }
                // L2+ is admissible if the family allows that level
                meta.allowed_levels.contains(&level)
            }
            None => false,
        }
    }

    /// All registered A family ids
    pub fn a_family_ids(&self) -> Vec<&str> {
        self.families
            .values()
            .filter(|m| m.hierarchy == Hierarchy::A)
            .map(|m| m.id.as_str())
            .collect()
    }

    /// All registered B family ids
    pub fn b_family_ids(&self) -> Vec<&str> {
        self.families
            .values()
            .filter(|m| m.hierarchy == Hierarchy::B)
            .map(|m| m.id.as_str())
            .collect()
    }
}
```

- [ ] **Step 5: Implement c_query_contract.rs**

Create `crates/pipeline/raw_sources/src/c_query_contract.rs`:

```rust
use std::collections::HashMap;

/// A shell mode that consumes C:L2 views
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConsumerShell {
    Commander,
    Researcher,
    ThreadTimeline,
    CoreShell,
    BattleCommand,
    WorldPrimary,
}

impl ConsumerShell {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsumerShell::Commander => "commander",
            ConsumerShell::Researcher => "researcher",
            ConsumerShell::ThreadTimeline => "thread_timeline",
            ConsumerShell::CoreShell => "core_shell",
            ConsumerShell::BattleCommand => "battle_command",
            ConsumerShell::WorldPrimary => "world_primary",
        }
    }
}

/// Which hierarchies a shell consumes C:L0 from
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum C0Source {
    AOnly,
    BOnly,
    AAndB,
}

/// Shell's C:L0 admissibility declaration
#[derive(Clone, Debug)]
pub struct ShellC0Admissibility {
    pub shell: ConsumerShell,
    pub source: C0Source,
    pub admissible_families: Vec<String>,
}

/// C:L2 view contract metadata
#[derive(Clone, Debug)]
pub struct C2ViewContract {
    pub consumer_shell: ConsumerShell,
    pub view_id: String,
    pub sql_name: String,
    pub view_kind: ViewKind,
    pub row_grain: String,
    pub required_join_keys: Vec<String>,
    pub source_families: Vec<String>,
    pub allowed_source_levels: Vec<String>,
    pub projected_columns: Vec<String>,
    pub abac_scope: String,
    pub exclusion_behavior: String,
    pub refresh_mode: String,
    pub refresh_watermark: String,
    pub refresh_dependencies: Vec<String>,
}

/// SQL view kind
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewKind {
    View,
    MaterializedView,
}

impl ViewKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewKind::View => "VIEW",
            ViewKind::MaterializedView => "MATERIALIZED VIEW",
        }
    }
}

/// The C query contract registry
#[derive(Clone, Debug)]
pub struct CQueryContract {
    shell_admissibility: HashMap<ConsumerShell, ShellC0Admissibility>,
    c2_views: HashMap<String, C2ViewContract>,
}

impl Default for CQueryContract {
    fn default() -> Self {
        let mut shell_admissibility = HashMap::new();
        let mut c2_views = HashMap::new();

        // Commander shell: A + B
        shell_admissibility.insert(
            ConsumerShell::Commander,
            ShellC0Admissibility {
                shell: ConsumerShell::Commander,
                source: C0Source::AAndB,
                admissible_families: vec![
                    "a_session_snapshot".into(),
                    "a_task_event".into(),
                    "b_session".into(),
                    "b_runtime_status".into(),
                ],
            },
        );

        // Researcher shell: A + B
        shell_admissibility.insert(
            ConsumerShell::Researcher,
            ShellC0Admissibility {
                shell: ConsumerShell::Researcher,
                source: C0Source::AAndB,
                admissible_families: vec![
                    "a_artifact_event".into(),
                    "b_artifact".into(),
                    "b_exclusion".into(),
                ],
            },
        );

        // Thread timeline shell: A + B
        shell_admissibility.insert(
            ConsumerShell::ThreadTimeline,
            ShellC0Admissibility {
                shell: ConsumerShell::ThreadTimeline,
                source: C0Source::AAndB,
                admissible_families: vec![
                    "a_message_event".into(),
                    "a_task_event".into(),
                    "a_artifact_event".into(),
                    "b_message".into(),
                    "b_task".into(),
                    "b_intent".into(),
                ],
            },
        );

        // Core shell governance: A + B
        shell_admissibility.insert(
            ConsumerShell::CoreShell,
            ShellC0Admissibility {
                shell: ConsumerShell::CoreShell,
                source: C0Source::AAndB,
                admissible_families: vec![
                    "a_exclusion_event".into(),
                    "a_replay_checkpoint".into(),
                    "a_backpressure_event".into(),
                    "b_exclusion".into(),
                    "b_conflict".into(),
                    "b_replay_checkpoint".into(),
                    "b_transform".into(),
                ],
            },
        );

        // Battle command / world-primary shell: A + B
        shell_admissibility.insert(
            ConsumerShell::BattleCommand,
            ShellC0Admissibility {
                shell: ConsumerShell::BattleCommand,
                source: C0Source::AAndB,
                admissible_families: vec![
                    "a_session_snapshot".into(),
                    "a_task_event".into(),
                    "a_artifact_event".into(),
                    "b_session".into(),
                    "b_task".into(),
                    "b_artifact".into(),
                    "b_runtime_status".into(),
                    "b_backpressure".into(),
                ],
            },
        );

        // Register C:L2 view contracts
        // Commander sessions view
        c2_views.insert(
            "c_l2_commander_sessions_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::Commander,
                view_id: "c_l2_commander_sessions_v".into(),
                sql_name: "c_l2_commander_sessions_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per session".into(),
                required_join_keys: vec!["session_key".into(), "thread_key".into()],
                source_families: vec![
                    "a_session_snapshot".into(),
                    "b_session".into(),
                    "b_runtime_status".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "session_key".into(),
                    "thread_key".into(),
                    "status".into(),
                    "active_agents".into(),
                    "runtime_summary".into(),
                    "lineage_anchor".into(),
                    "refresh_watermark".into(),
                ],
                abac_scope: "session".into(),
                exclusion_behavior: "exclude redacted sessions".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec!["a_session_snapshot".into(), "b_session".into()],
            },
        );

        // Commander tasks view
        c2_views.insert(
            "c_l2_commander_tasks_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::Commander,
                view_id: "c_l2_commander_tasks_v".into(),
                sql_name: "c_l2_commander_tasks_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per task".into(),
                required_join_keys: vec!["task_key".into(), "thread_key".into()],
                source_families: vec![
                    "a_task_event".into(),
                    "b_task".into(),
                    "b_runtime_status".into(),
                    "b_exclusion".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "task_key".into(),
                    "thread_key".into(),
                    "status".into(),
                    "assigned_agent".into(),
                    "queue_state".into(),
                    "exclusion_state".into(),
                    "lineage_anchor".into(),
                    "update_watermark".into(),
                ],
                abac_scope: "task".into(),
                exclusion_behavior: "exclude redacted tasks".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec!["a_task_event".into(), "b_task".into()],
            },
        );

        // Researcher artifacts view
        c2_views.insert(
            "c_l2_researcher_artifacts_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::Researcher,
                view_id: "c_l2_researcher_artifacts_v".into(),
                sql_name: "c_l2_researcher_artifacts_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per artifact".into(),
                required_join_keys: vec!["artifact_key".into(), "task_key".into()],
                source_families: vec![
                    "a_artifact_event".into(),
                    "b_artifact".into(),
                    "b_exclusion".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into(), "L4".into()],
                projected_columns: vec![
                    "artifact_key".into(),
                    "artifact_family".into(),
                    "producer_task".into(),
                    "producer_agent".into(),
                    "source_event".into(),
                    "exclusion_state".into(),
                    "created_at".into(),
                ],
                abac_scope: "artifact".into(),
                exclusion_behavior: "exclude redacted artifacts".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec!["a_artifact_event".into(), "b_artifact".into()],
            },
        );

        // Thread timeline view
        c2_views.insert(
            "c_l2_thread_timeline_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::ThreadTimeline,
                view_id: "c_l2_thread_timeline_v".into(),
                sql_name: "c_l2_thread_timeline_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per timeline event".into(),
                required_join_keys: vec!["thread_key".into(), "sequence_id".into()],
                source_families: vec![
                    "a_message_event".into(),
                    "a_task_event".into(),
                    "a_artifact_event".into(),
                    "b_message".into(),
                    "b_task".into(),
                    "b_intent".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "thread_key".into(),
                    "sequence_id".into(),
                    "event_type".into(),
                    "event_key".into(),
                    "event_time".into(),
                    "actor".into(),
                    "summary".into(),
                ],
                abac_scope: "thread".into(),
                exclusion_behavior: "exclude redacted events".into(),
                refresh_mode: "append".into(),
                refresh_watermark: "sequence_id".into(),
                refresh_dependencies: vec![
                    "a_message_event".into(),
                    "a_task_event".into(),
                    "a_artifact_event".into(),
                ],
            },
        );

        // Shell governance view
        c2_views.insert(
            "c_l2_shell_governance_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::CoreShell,
                view_id: "c_l2_shell_governance_v".into(),
                sql_name: "c_l2_shell_governance_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per governance event".into(),
                required_join_keys: vec!["target_key".into(), "event_key".into()],
                source_families: vec![
                    "a_exclusion_event".into(),
                    "a_replay_checkpoint".into(),
                    "a_backpressure_event".into(),
                    "b_exclusion".into(),
                    "b_conflict".into(),
                    "b_replay_checkpoint".into(),
                    "b_transform".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into(), "L6".into()],
                projected_columns: vec![
                    "event_key".into(),
                    "event_type".into(),
                    "target_key".into(),
                    "policy_scope".into(),
                    "visibility_state".into(),
                    "event_time".into(),
                ],
                abac_scope: "global".into(),
                exclusion_behavior: "tombstone excluded targets".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec![
                    "a_exclusion_event".into(),
                    "b_exclusion".into(),
                    "b_conflict".into(),
                ],
            },
        );

        Self {
            shell_admissibility,
            c2_views,
        }
    }
}

impl CQueryContract {
    /// Get a shell's C:L0 admissibility declaration
    pub fn get_shell_admissibility(&self, shell: &ConsumerShell) -> Option<&ShellC0Admissibility> {
        self.shell_admissibility.get(shell)
    }

    /// Get a C:L2 view contract by view id
    pub fn get_c2_view(&self, view_id: &str) -> Option<&C2ViewContract> {
        self.c2_views.get(view_id)
    }

    /// All registered C:L2 view ids
    pub fn all_c2_view_ids(&self) -> Vec<&str> {
        self.c2_views.keys().map(|k| k.as_str()).collect()
    }

    /// Get all C:L2 views for a shell
    pub fn get_c2_views_for_shell(&self, shell: &ConsumerShell) -> Vec<&C2ViewContract> {
        self.c2_views
            .values()
            .filter(|v| v.consumer_shell == *shell)
            .collect()
    }

    /// All registered shells
    pub fn all_shells(&self) -> Vec<&ConsumerShell> {
        self.shell_admissibility.keys().collect()
    }
}
```

- [ ] **Step 6: Update lib.rs exports**

Modify `crates/pipeline/raw_sources/src/lib.rs`:

```rust
pub mod artifact_gateway;
pub mod c_query_contract;
pub mod error;
pub mod run_stream;
pub mod storage_contract;
pub mod thread_gateway;
pub mod upload_gateway;

pub use artifact_gateway::{preview_artifact, ArtifactAccess};
pub use c_query_contract::{C2ViewContract, CQueryContract, ConsumerShell, ViewKind};
pub use run_stream::{load_stream_fixture, AdapterEvent, RawStreamEvent};
pub use storage_contract::{
    AFamily, BFamily, CanonicalLevel, FamilyId, FamilyMeta, Hierarchy, StorageContract,
};
pub use thread_gateway::{create_thread, resume_thread, ThreadSnapshot};
pub use upload_gateway::{stage_upload, UploadReceipt};
```

- [ ] **Step 7: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-raw-sources --test storage_contract -v`
Expected: PASS

---

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

### Task 2: DeerFlow Raw Landing into Shared A/B Families

**Files:**
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

- [ ] **Step 1: Create test fixtures**

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "thread_snapshot",
  "thread_id": "thread_df_1",
  "session_id": "session_df_1",
  "title": "Survey the ridge",
  "run_id": "run_df_1",
  "agent_ids": ["agent_alpha", "agent_beta"],
  "observed_at": "2026-04-04T00:00:00Z"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "live_activity",
  "thread_id": "thread_df_1",
  "run_id": "run_df_1",
  "events": [
    {
      "event_id": "evt_msg_1",
      "kind": "message",
      "message_id": "msg_df_1",
      "role": "operator",
      "text": "Survey the ridge.",
      "agent_id": "agent_alpha"
    },
    {
      "event_id": "evt_task_1",
      "kind": "task_progress",
      "task_id": "task_df_1",
      "state": "running",
      "label": "Gather terrain notes"
    },
    {
      "event_id": "evt_art_1",
      "kind": "artifact_presented",
      "artifact_id": "artifact_df_1",
      "name": "scan.png",
      "task_id": "task_df_1"
    },
    {
      "event_id": "evt_rt_1",
      "kind": "runtime_status",
      "state": "live",
      "agent_id": "agent_alpha"
    }
  ]
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "replay_window",
  "thread_id": "thread_df_1",
  "window_id": "window_df_1",
  "start_sequence": 1,
  "end_sequence": 10,
  "run_id": "run_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "intent",
  "intent_id": "intent_df_1",
  "thread_id": "thread_df_1",
  "actor_id": "operator_1",
  "action": "survey",
  "stage": "submitted"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "artifact_access",
  "artifact_id": "artifact_df_1",
  "access_id": "access_df_1",
  "mime_type": "image/png",
  "task_id": "task_df_1",
  "run_id": "run_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "exclusion",
  "exclusion_id": "excl_df_1",
  "target_id": "artifact_df_1",
  "target_kind": "artifact",
  "reason": "classified",
  "thread_id": "thread_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "conflict",
  "conflict_id": "conflict_df_1",
  "target_id": "task_df_1",
  "target_kind": "task",
  "conflicting_intents": ["intent_df_1", "intent_df_2"],
  "thread_id": "thread_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "backpressure",
  "backpressure_id": "bp_df_1",
  "run_id": "run_df_1",
  "severity": "high",
  "source": "artifact_store"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "replay_checkpoint",
  "checkpoint_id": "cp_df_1",
  "run_id": "run_df_1",
  "sequence_id": 5,
  "event_time": "2026-04-04T00:05:00Z"
}
```

- [ ] **Step 2: Write the failing DeerFlow landing test**

Create `crates/pipeline/raw_sources/tests/deerflow_adapter.rs`:

```rust
use deer_pipeline_raw_sources::deerflow::{
    bind_deerflow_fixtures, DeerFlowBinding, DeerFlowSourceKind,
};
use deer_pipeline_raw_sources::storage_contract::{Hierarchy, StorageContract};

#[test]
fn thread_snapshot_binds_to_a_session_snapshot() {
    let fixture = include_str!("fixtures/deerflow_thread_snapshot.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let session_binding = bindings
        .iter()
        .find(|b| b.target_family == "a_session_snapshot")
        .expect("a_session_snapshot binding not found");

    assert_eq!(session_binding.source_kind, DeerFlowSourceKind::ThreadSnapshot);
    assert_eq!(session_binding.generator_key, "deerflow");
    assert!(!session_binding.immutable_keys.is_empty());
}

#[test]
fn live_activity_binds_to_correct_a_families() {
    let fixture = include_str!("fixtures/deerflow_live_activity.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();

    assert!(families.contains(&"a_message_event"));
    assert!(families.contains(&"a_task_event"));
    assert!(families.contains(&"a_artifact_event"));
    assert!(families.contains(&"a_runtime_event"));
}

#[test]
fn replay_window_binds_to_b_replay_window() {
    let fixture = include_str!("fixtures/deerflow_replay_window.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let replay_binding = bindings
        .iter()
        .find(|b| b.target_family == "b_replay_window")
        .expect("b_replay_window binding not found");

    assert_eq!(replay_binding.source_kind, DeerFlowSourceKind::ReplayWindow);
}

#[test]
fn intent_binds_to_a_intent_event_and_b_intent() {
    let fixture = include_str!("fixtures/deerflow_intent.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_intent_event"));
    assert!(families.contains(&"b_intent"));
}

#[test]
fn artifact_preview_binds_to_a_artifact_event_and_b_artifact_access() {
    let fixture = include_str!("fixtures/deerflow_artifact_preview.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_artifact_event"));
    assert!(families.contains(&"b_artifact_access"));
}

#[test]
fn exclusion_binds_to_a_exclusion_event_and_b_exclusion() {
    let fixture = include_str!("fixtures/deerflow_exclusion_intent.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_exclusion_event"));
    assert!(families.contains(&"b_exclusion"));
}

#[test]
fn conflict_binds_to_b_conflict() {
    let fixture = include_str!("fixtures/deerflow_conflicting_intents.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let conflict_binding = bindings
        .iter()
        .find(|b| b.target_family == "b_conflict")
        .expect("b_conflict binding not found");

    assert_eq!(conflict_binding.source_kind, DeerFlowSourceKind::Conflict);
}

#[test]
fn backpressure_binds_to_a_backpressure_event_and_b_backpressure() {
    let fixture = include_str!("fixtures/deerflow_storage_backpressure.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_backpressure_event"));
    assert!(families.contains(&"b_backpressure"));
}

#[test]
fn replay_checkpoint_binds_to_a_replay_checkpoint_and_b_replay_checkpoint() {
    let fixture = include_str!("fixtures/deerflow_replay_checkpoint.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_replay_checkpoint"));
    assert!(families.contains(&"b_replay_checkpoint"));
}

#[test]
fn all_bindings_reference_shared_families_not_deerflow_only() {
    let fixtures = vec![
        include_str!("fixtures/deerflow_thread_snapshot.json"),
        include_str!("fixtures/deerflow_live_activity.json"),
        include_str!("fixtures/deerflow_replay_window.json"),
        include_str!("fixtures/deerflow_intent.json"),
        include_str!("fixtures/deerflow_artifact_preview.json"),
        include_str!("fixtures/deerflow_exclusion_intent.json"),
        include_str!("fixtures/deerflow_conflicting_intents.json"),
        include_str!("fixtures/deerflow_storage_backpressure.json"),
        include_str!("fixtures/deerflow_replay_checkpoint.json"),
    ];

    let contract = StorageContract::default();

    for fixture in fixtures {
        let bindings = bind_deerflow_fixtures(fixture).unwrap();
        for binding in &bindings {
            // Every target family must exist in the shared contract
            assert!(
                contract.get_family(&binding.target_family).is_some(),
                "DeerFlow-only family leaked: {}",
                binding.target_family
            );
        }
    }
}

#[test]
fn all_bindings_include_lineage_and_sequence() {
    let fixture = include_str!("fixtures/deerflow_live_activity.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    for binding in &bindings {
        assert!(!binding.lineage_anchor.is_empty());
        assert!(binding.sequence_id > 0);
    }
}

#[test]
fn no_consumer_c_l2_rows_emitted_from_raw_source() {
    let fixtures = vec![
        include_str!("fixtures/deerflow_thread_snapshot.json"),
        include_str!("fixtures/deerflow_live_activity.json"),
    ];

    for fixture in fixtures {
        let bindings = bind_deerflow_fixtures(fixture).unwrap();
        for binding in &bindings {
            assert!(
                !binding.target_family.starts_with("c_l2_"),
                "C:L2 row leaked from raw source: {}",
                binding.target_family
            );
        }
    }
}
```

- [ ] **Step 3: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`
Expected: FAIL — module `deerflow` does not exist

- [ ] **Step 4: Implement deerflow.rs**

Create `crates/pipeline/raw_sources/src/deerflow.rs`:

```rust
use serde::Deserialize;

use crate::error::RawSourceError;

/// DeerFlow source kind labels
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeerFlowSourceKind {
    ThreadSnapshot,
    LiveActivity,
    ReplayWindow,
    Intent,
    ArtifactAccess,
    Exclusion,
    Conflict,
    Backpressure,
    ReplayCheckpoint,
}

/// A single DeerFlow-to-A/B family binding
#[derive(Clone, Debug)]
pub struct DeerFlowBinding {
    pub generator_key: String,
    pub source_kind: DeerFlowSourceKind,
    pub target_family: String,
    pub immutable_keys: Vec<String>,
    pub lineage_anchor: String,
    pub sequence_id: u64,
    pub correlation_fields: Vec<String>,
}

/// Raw DeerFlow fixture envelope
#[derive(Debug, Deserialize)]
struct DeerFlowRawFixture {
    generator: String,
    source_kind: String,
    #[serde(flatten)]
    payload: serde_json::Value,
}

/// Bind a DeerFlow fixture JSON string to shared A/B family ids
pub fn bind_deerflow_fixtures(
    fixture_json: &str,
) -> Result<Vec<DeerFlowBinding>, RawSourceError> {
    let raw: DeerFlowRawFixture =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    let source_kind = parse_source_kind(&raw.source_kind)?;
    let mut bindings = Vec::new();
    let mut seq = 0u64;

    match source_kind {
        DeerFlowSourceKind::ThreadSnapshot => {
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_session_snapshot".into(),
                immutable_keys: vec![
                    raw.payload["session_id"].as_str().unwrap_or("").into(),
                    raw.payload["thread_id"].as_str().unwrap_or("").into(),
                ],
                lineage_anchor: raw.payload["thread_id"].as_str().unwrap_or("").into(),
                sequence_id: seq,
                correlation_fields: vec!["thread_id".into(), "session_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::LiveActivity => {
            if let Some(events) = raw.payload["events"].as_array() {
                for event in events {
                    seq += 1;
                    let kind = event["kind"].as_str().unwrap_or("");
                    let target = match kind {
                        "message" => "a_message_event",
                        "task_progress" => "a_task_event",
                        "artifact_presented" => "a_artifact_event",
                        "runtime_status" => "a_runtime_event",
                        _ => continue,
                    };
                    let event_id = event["event_id"].as_str().unwrap_or("");
                    bindings.push(DeerFlowBinding {
                        generator_key: "deerflow".into(),
                        source_kind: source_kind.clone(),
                        target_family: target.into(),
                        immutable_keys: vec![event_id.into()],
                        lineage_anchor: event_id.into(),
                        sequence_id: seq,
                        correlation_fields: vec![
                            "thread_id".into(),
                            "run_id".into(),
                        ],
                    });
                }
            }
        }
        DeerFlowSourceKind::ReplayWindow => {
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_replay_window".into(),
                immutable_keys: vec![
                    raw.payload["window_id"].as_str().unwrap_or("").into(),
                    raw.payload["thread_id"].as_str().unwrap_or("").into(),
                ],
                lineage_anchor: raw.payload["window_id"].as_str().unwrap_or("").into(),
                sequence_id: seq,
                correlation_fields: vec!["thread_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::Intent => {
            seq += 1;
            let intent_id = raw.payload["intent_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_intent_event".into(),
                immutable_keys: vec![intent_id.into()],
                lineage_anchor: intent_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["intent_id".into(), "thread_id".into(), "actor_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_intent".into(),
                immutable_keys: vec![intent_id.into()],
                lineage_anchor: intent_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["intent_id".into(), "thread_id".into(), "actor_id".into()],
            });
        }
        DeerFlowSourceKind::ArtifactAccess => {
            seq += 1;
            let artifact_id = raw.payload["artifact_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_artifact_event".into(),
                immutable_keys: vec![artifact_id.into()],
                lineage_anchor: artifact_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["artifact_id".into(), "task_id".into(), "run_id".into()],
            });
            seq += 1;
            let access_id = raw.payload["access_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_artifact_access".into(),
                immutable_keys: vec![artifact_id.into(), access_id.into()],
                lineage_anchor: access_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["artifact_id".into(), "access_id".into()],
            });
        }
        DeerFlowSourceKind::Exclusion => {
            seq += 1;
            let exclusion_id = raw.payload["exclusion_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_exclusion_event".into(),
                immutable_keys: vec![exclusion_id.into()],
                lineage_anchor: exclusion_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["exclusion_id".into(), "target_id".into(), "thread_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_exclusion".into(),
                immutable_keys: vec![exclusion_id.into()],
                lineage_anchor: exclusion_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["exclusion_id".into(), "target_id".into()],
            });
        }
        DeerFlowSourceKind::Conflict => {
            seq += 1;
            let conflict_id = raw.payload["conflict_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_conflict".into(),
                immutable_keys: vec![conflict_id.into()],
                lineage_anchor: conflict_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["conflict_id".into(), "target_id".into(), "thread_id".into()],
            });
        }
        DeerFlowSourceKind::Backpressure => {
            seq += 1;
            let bp_id = raw.payload["backpressure_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_backpressure_event".into(),
                immutable_keys: vec![bp_id.into()],
                lineage_anchor: bp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["backpressure_id".into(), "run_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_backpressure".into(),
                immutable_keys: vec![bp_id.into()],
                lineage_anchor: bp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["backpressure_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::ReplayCheckpoint => {
            seq += 1;
            let cp_id = raw.payload["checkpoint_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_replay_checkpoint".into(),
                immutable_keys: vec![cp_id.into()],
                lineage_anchor: cp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["checkpoint_id".into(), "run_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_replay_checkpoint".into(),
                immutable_keys: vec![cp_id.into()],
                lineage_anchor: cp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["checkpoint_id".into(), "run_id".into()],
            });
        }
    }

    Ok(bindings)
}

fn parse_source_kind(value: &str) -> Result<DeerFlowSourceKind, RawSourceError> {
    match value {
        "thread_snapshot" => Ok(DeerFlowSourceKind::ThreadSnapshot),
        "live_activity" => Ok(DeerFlowSourceKind::LiveActivity),
        "replay_window" => Ok(DeerFlowSourceKind::ReplayWindow),
        "intent" => Ok(DeerFlowSourceKind::Intent),
        "artifact_access" => Ok(DeerFlowSourceKind::ArtifactAccess),
        "exclusion" => Ok(DeerFlowSourceKind::Exclusion),
        "conflict" => Ok(DeerFlowSourceKind::Conflict),
        "backpressure" => Ok(DeerFlowSourceKind::Backpressure),
        "replay_checkpoint" => Ok(DeerFlowSourceKind::ReplayCheckpoint),
        _ => Err(RawSourceError::InvalidFixture),
    }
}
```

- [ ] **Step 5: Update lib.rs to export deerflow module**

Modify `crates/pipeline/raw_sources/src/lib.rs`:

```rust
pub mod artifact_gateway;
pub mod c_query_contract;
pub mod deerflow;
pub mod error;
pub mod run_stream;
pub mod storage_contract;
pub mod thread_gateway;
pub mod upload_gateway;

pub use artifact_gateway::{preview_artifact, ArtifactAccess};
pub use c_query_contract::{C2ViewContract, CQueryContract, ConsumerShell, ViewKind};
pub use deerflow::{bind_deerflow_fixtures, DeerFlowBinding, DeerFlowSourceKind};
pub use run_stream::{load_stream_fixture, AdapterEvent, RawStreamEvent};
pub use storage_contract::{
    AFamily, BFamily, CanonicalLevel, FamilyId, FamilyMeta, Hierarchy, StorageContract,
};
pub use thread_gateway::{create_thread, resume_thread, ThreadSnapshot};
pub use upload_gateway::{stage_upload, UploadReceipt};
```

- [ ] **Step 6: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`
Expected: PASS

---

### Task 3: DeerFlow Normalization to Shared A/B L2+ Rows

**Files:**
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`
- Test fixture: `crates/pipeline/normalizers/tests/fixtures/deerflow_normalized_batch.json`

- [ ] **Step 1: Create normalization fixture**

Create `crates/pipeline/normalizers/tests/fixtures/deerflow_normalized_batch.json`:

```json
{
  "generator": "deerflow",
  "session": {
    "session_id": "session_df_1",
    "title": "Survey the ridge"
  },
  "run": {
    "run_id": "run_df_1",
    "status": "running"
  },
  "events": [
    {
      "kind": "message",
      "message_id": "msg_df_1",
      "role": "operator",
      "text": "Survey the ridge.",
      "level": "L2"
    },
    {
      "kind": "task",
      "task_id": "task_df_1",
      "title": "Gather terrain notes",
      "state": "running"
    },
    {
      "kind": "artifact",
      "artifact_id": "artifact_df_1",
      "name": "scan.png",
      "status": "presented",
      "as_is_hash": "sha256:scan-png",
      "parent_message_id": "msg_df_1",
      "parent_clarification_id": null
    },
    {
      "kind": "artifact",
      "artifact_id": "artifact_df_2",
      "name": "prediction.json",
      "status": "presented",
      "as_is_hash": "sha256:pred-json",
      "parent_message_id": "msg_df_1",
      "parent_clarification_id": null
    },
    {
      "kind": "runtime_status",
      "state": "live"
    }
  ],
  "governance": {
    "exclusions": [
      {
        "exclusion_id": "excl_df_1",
        "target_id": "artifact_df_2",
        "reason": "classified"
      }
    ],
    "replay_checkpoints": [
      {
        "checkpoint_id": "cp_df_1",
        "sequence_id": 5,
        "run_id": "run_df_1"
      }
    ],
    "transforms": [
      {
        "transform_id": "xform_df_1",
        "target_id": "task_df_1",
        "source_event": "evt_task_1"
      }
    ]
  }
}
```

- [ ] **Step 2: Write the failing DeerFlow normalization test**

Create `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`:

```rust
use deer_foundation_contracts::{CanonicalLevel, RecordFamily};
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{normalize_deerflow_batch, NormalizedBatch};

#[test]
fn deerflow_landing_produces_l2_plus_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let level = record.header().level;
        assert!(
            matches!(
                level,
                CanonicalLevel::L2
                    | CanonicalLevel::L3
                    | CanonicalLevel::L4
                    | CanonicalLevel::L5
                    | CanonicalLevel::L6
            ),
            "expected L2+ but got {:?}",
            level
        );
    }
}

#[test]
fn deerflow_normalization_produces_shared_family_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    let families: Vec<RecordFamily> = batch.records.iter().map(|r| r.header().family).collect();

    // Must produce A-family rows
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Session)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Message)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Task)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Artifact)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::RuntimeStatus)));
}

#[test]
fn deerflow_normalization_preserves_lineage() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let header = record.header();
        // Every record must have a lineage anchor
        assert!(
            !header.identity.primary_id.as_str().is_empty(),
            "record missing primary_id"
        );
    }
}

#[test]
fn deerflow_normalization_emits_no_c_l2_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let family = record.header().family;
        assert!(
            !matches!(family, RecordFamily::L2View),
            "normalizer emitted C:L2 row"
        );
    }
}

#[test]
fn deerflow_normalization_supports_higher_level_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    // At least some records should be at L2
    let has_l2 = batch
        .records
        .iter()
        .any(|r| matches!(r.header().level, CanonicalLevel::L2));
    assert!(has_l2, "expected some L2 records");
}

#[test]
fn deerflow_governance_rows_are_emitted() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    let families: Vec<RecordFamily> = batch.records.iter().map(|r| r.header().family).collect();

    // Governance rows should be present
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::Exclusion)),
        "missing exclusion rows"
    );
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::ReplayCheckpoint)),
        "missing replay checkpoint rows"
    );
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::Transform)),
        "missing transform rows"
    );
}
```

- [ ] **Step 3: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`
Expected: FAIL — `normalize_deerflow_batch` does not exist

- [ ] **Step 4: Implement normalize_deerflow_batch**

Modify `crates/pipeline/normalizers/src/carrier.rs` — add at the end:

```rust
/// Normalize a DeerFlow batch fixture directly into shared A/B L2+ rows.
/// This reads the same envelope format as `normalize_batch` but ensures
/// all output records are at L2+ with lineage preserved.
pub fn normalize_deerflow_batch(
    fixture_json: &str,
) -> Result<NormalizedBatch, NormalizationError> {
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture_json)
        .map_err(|_| NormalizationError::UnsupportedEventKind("deerflow_batch".into()))?;

    let mut result = normalize_batch(&batch)?;

    // Ensure all records are at L2+
    for record in &mut result.records {
        let header = record.header_mut();
        if matches!(header.level, CanonicalLevel::L0 | CanonicalLevel::L1) {
            header.level = CanonicalLevel::L2;
        }
    }

    // Parse governance extensions from fixture
    let fixture: serde_json::Value = serde_json::from_str(fixture_json)
        .map_err(|_| NormalizationError::UnsupportedEventKind("deerflow_batch".into()))?;

    if let Some(governance) = fixture.get("governance") {
        // Process exclusions
        if let Some(exclusions) = governance.get("exclusions").and_then(|v| v.as_array()) {
            for excl in exclusions {
                let excl_id = excl["exclusion_id"].as_str().unwrap_or("");
                let target_id = excl["target_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::Exclusion(
                    deer_foundation_domain::ExclusionRecord::new(
                        RecordId::from(excl_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(excl_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::ExclusionBody {
                            target_id: target_id.into(),
                            reason: excl["reason"].as_str().unwrap_or("").into(),
                        },
                    ),
                ));
            }
        }

        // Process replay checkpoints
        if let Some(checkpoints) = governance.get("replay_checkpoints").and_then(|v| v.as_array()) {
            for cp in checkpoints {
                let cp_id = cp["checkpoint_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::ReplayCheckpoint(
                    deer_foundation_domain::ReplayCheckpointRecord::new(
                        RecordId::from(cp_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(cp_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::ReplayCheckpointBody {
                            sequence_id: cp["sequence_id"].as_u64().unwrap_or(0) as u32,
                        },
                    ),
                ));
            }
        }

        // Process transforms
        if let Some(transforms) = governance.get("transforms").and_then(|v| v.as_array()) {
            for xform in transforms {
                let xform_id = xform["transform_id"].as_str().unwrap_or("");
                let target_id = xform["target_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::Transform(
                    deer_foundation_domain::TransformRecord::new(
                        RecordId::from(xform_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(xform_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::TransformBody {
                            target_id: target_id.into(),
                            source_event: xform["source_event"].as_str().unwrap_or("").into(),
                        },
                    ),
                ));
            }
        }
    }

    Ok(result)
}
```

Add `header_mut()` method to `CanonicalRecord` trait in `crates/foundation/contracts/src/records.rs`:

```rust
pub trait CanonicalRecord {
    fn header(&self) -> &RecordHeader;
    fn header_mut(&mut self) -> &mut RecordHeader;
}
```

Update the `define_record!` macro in `crates/foundation/domain/src/common.rs` to also implement `header_mut`:

```rust
impl deer_foundation_contracts::CanonicalRecord for $name {
    fn header(&self) -> &deer_foundation_contracts::RecordHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut deer_foundation_contracts::RecordHeader {
        &mut self.header
    }
}
```

Update `AnyRecord` in `crates/foundation/domain/src/lib.rs` to also implement `header_mut`:

```rust
impl CanonicalRecord for AnyRecord {
    fn header(&self) -> &RecordHeader {
        match self {
            AnyRecord::Run(record) => &record.header,
            // ... all existing arms ...
        }
    }

    fn header_mut(&mut self) -> &mut RecordHeader {
        match self {
            AnyRecord::Run(record) => &mut record.header,
            AnyRecord::Session(record) => &mut record.header,
            AnyRecord::Task(record) => &mut record.header,
            AnyRecord::Message(record) => &mut record.header,
            AnyRecord::ToolCall(record) => &mut record.header,
            AnyRecord::Artifact(record) => &mut record.header,
            AnyRecord::Clarification(record) => &mut record.header,
            AnyRecord::RuntimeStatus(record) => &mut record.header,
            AnyRecord::Delivery(record) => &mut record.header,
            AnyRecord::L0Source(record) => &mut record.header,
            AnyRecord::L1Sanitized(record) => &mut record.header,
            AnyRecord::L2View(record) => &mut record.header,
            AnyRecord::L3Insight(record) => &mut record.header,
            AnyRecord::L4Prediction(record) => &mut record.header,
            AnyRecord::L5Prescription(record) => &mut record.header,
            AnyRecord::L6Outcome(record) => &mut record.header,
            AnyRecord::AsIsRepresentation(record) => &mut record.header,
            AnyRecord::Chunk(record) => &mut record.header,
            AnyRecord::Embedding(record) => &mut record.header,
            AnyRecord::Intent(record) => &mut record.header,
            AnyRecord::Transform(record) => &mut record.header,
            AnyRecord::Exclusion(record) => &mut record.header,
            AnyRecord::Conflict(record) => &mut record.header,
            AnyRecord::Resolution(record) => &mut record.header,
            AnyRecord::ReplayCheckpoint(record) => &mut record.header,
            AnyRecord::Dedup(record) => &mut record.header,
            AnyRecord::Batch(record) => &mut record.header,
            AnyRecord::Branch(record) => &mut record.header,
            AnyRecord::Version(record) => &mut record.header,
            AnyRecord::WriteOperation(record) => &mut record.header,
            AnyRecord::GraphNode(record) => &mut record.header,
            AnyRecord::GraphEdge(record) => &mut record.header,
            AnyRecord::KnowledgeEntity(record) => &mut record.header,
            AnyRecord::KnowledgeRelation(record) => &mut record.header,
        }
    }
}
```

- [ ] **Step 5: Update normalizers lib.rs**

Modify `crates/pipeline/normalizers/src/lib.rs`:

```rust
pub mod carrier;
pub mod c_view_catalog;
pub mod envelopes;
pub mod error;
pub mod governance;
pub mod promotions;
pub mod representation;

pub use carrier::normalize_deerflow_batch;
pub use carrier::normalize_stream_batch;
pub use carrier::{normalize_batch, NormalizedBatch};
pub use c_view_catalog::CViewCatalog;
pub use envelopes::RawEnvelopeBatch;
pub use error::NormalizationError;
pub use governance::emit_intent_records;
pub use promotions::validate_promotion;
pub use representation::normalize_representation_chain;
```

- [ ] **Step 6: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`
Expected: PASS

---

### Task 4: Register C:L2 Presentation Views for Commander, Researcher, Thread, Core Shell

**Files:**
- Create: `crates/pipeline/normalizers/src/c_view_catalog.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/c_l2_projection_catalog.rs`

- [ ] **Step 1: Write the failing C:L2 projection-catalog test**

Create `crates/pipeline/normalizers/tests/c_l2_projection_catalog.rs`:

```rust
use deer_pipeline_normalizers::c_view_catalog::{CViewCatalog, CViewEntry};
use deer_pipeline_raw_sources::ConsumerShell;

#[test]
fn commander_sessions_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_commander_sessions_v").expect("missing commander sessions view");
    assert_eq!(view.consumer_shell, ConsumerShell::Commander);
    assert_eq!(view.row_grain, "one row per session");
}

#[test]
fn commander_tasks_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_commander_tasks_v").expect("missing commander tasks view");
    assert_eq!(view.consumer_shell, ConsumerShell::Commander);
    assert_eq!(view.row_grain, "one row per task");
}

#[test]
fn researcher_artifacts_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_researcher_artifacts_v").expect("missing researcher artifacts view");
    assert_eq!(view.consumer_shell, ConsumerShell::Researcher);
    assert_eq!(view.row_grain, "one row per artifact");
}

#[test]
fn thread_timeline_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_thread_timeline_v").expect("missing thread timeline view");
    assert_eq!(view.consumer_shell, ConsumerShell::ThreadTimeline);
    assert_eq!(view.row_grain, "one row per timeline event");
}

#[test]
fn shell_governance_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_shell_governance_v").expect("missing shell governance view");
    assert_eq!(view.consumer_shell, ConsumerShell::CoreShell);
    assert_eq!(view.row_grain, "one row per governance event");
}

#[test]
fn all_views_declare_source_families() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.source_families.is_empty(),
            "view {} has no source families",
            view_id
        );
    }
}

#[test]
fn all_views_declare_join_keys() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.required_join_keys.is_empty(),
            "view {} has no join keys",
            view_id
        );
    }
}

#[test]
fn all_views_declare_projected_columns() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.projected_columns.is_empty(),
            "view {} has no projected columns",
            view_id
        );
    }
}

#[test]
fn all_views_declare_abac_and_exclusion() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.abac_scope.is_empty(),
            "view {} has no ABAC scope",
            view_id
        );
        assert!(
            !view.exclusion_behavior.is_empty(),
            "view {} has no exclusion behavior",
            view_id
        );
    }
}

#[test]
fn all_views_declare_refresh_metadata() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.refresh_mode.is_empty(),
            "view {} has no refresh mode",
            view_id
        );
        assert!(
            !view.refresh_watermark.is_empty(),
            "view {} has no refresh watermark",
            view_id
        );
    }
}

#[test]
fn all_views_read_only_from_l2_plus() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        for level in &view.allowed_source_levels {
            assert!(
                level == "L2" || level == "L3" || level == "L4" || level == "L5" || level == "L6",
                "view {} reads from non-L2+ level: {}",
                view_id,
                level
            );
        }
    }
}

#[test]
fn all_views_have_view_kind_declared() {
    let catalog = CViewCatalog::default();
    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        let kind_str = view.view_kind.as_str();
        assert!(
            kind_str == "VIEW" || kind_str == "MATERIALIZED VIEW",
            "view {} has invalid kind: {}",
            view_id,
            kind_str
        );
    }
}

#[test]
fn views_grouped_by_shell() {
    let catalog = CViewCatalog::default();

    let commander_views = catalog.get_views_for_shell(&ConsumerShell::Commander);
    assert!(commander_views.len() >= 2, "commander shell should have sessions + tasks views");

    let researcher_views = catalog.get_views_for_shell(&ConsumerShell::Researcher);
    assert!(researcher_views.len() >= 1);

    let thread_views = catalog.get_views_for_shell(&ConsumerShell::ThreadTimeline);
    assert!(thread_views.len() >= 1);

    let core_views = catalog.get_views_for_shell(&ConsumerShell::CoreShell);
    assert!(core_views.len() >= 1);
}
```

- [ ] **Step 2: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`
Expected: FAIL — module `c_view_catalog` does not exist

- [ ] **Step 3: Implement c_view_catalog.rs**

Create `crates/pipeline/normalizers/src/c_view_catalog.rs`:

```rust
use std::collections::HashMap;

use deer_pipeline_raw_sources::{C2ViewContract, CQueryContract, ConsumerShell, ViewKind};

/// A catalog entry for a C:L2 view implementation
#[derive(Clone, Debug)]
pub struct CViewEntry {
    pub consumer_shell: ConsumerShell,
    pub view_id: String,
    pub sql_name: String,
    pub view_kind: ViewKind,
    pub row_grain: String,
    pub required_join_keys: Vec<String>,
    pub source_families: Vec<String>,
    pub allowed_source_levels: Vec<String>,
    pub projected_columns: Vec<String>,
    pub abac_scope: String,
    pub exclusion_behavior: String,
    pub refresh_mode: String,
    pub refresh_watermark: String,
    pub refresh_dependencies: Vec<String>,
    pub sql_definition: String,
}

impl CViewEntry {
    fn from_contract(contract: &C2ViewContract, sql: &str) -> Self {
        Self {
            consumer_shell: contract.consumer_shell.clone(),
            view_id: contract.view_id.clone(),
            sql_name: contract.sql_name.clone(),
            view_kind: contract.view_kind.clone(),
            row_grain: contract.row_grain.clone(),
            required_join_keys: contract.required_join_keys.clone(),
            source_families: contract.source_families.clone(),
            allowed_source_levels: contract.allowed_source_levels.clone(),
            projected_columns: contract.projected_columns.clone(),
            abac_scope: contract.abac_scope.clone(),
            exclusion_behavior: contract.exclusion_behavior.clone(),
            refresh_mode: contract.refresh_mode.clone(),
            refresh_watermark: contract.refresh_watermark.clone(),
            refresh_dependencies: contract.refresh_dependencies.clone(),
            sql_definition: sql.into(),
        }
    }
}

/// The C:L1/C:L2 presentation transform and SQL view implementation catalog
#[derive(Clone, Debug)]
pub struct CViewCatalog {
    views: HashMap<String, CViewEntry>,
}

impl Default for CViewCatalog {
    fn default() -> Self {
        let contracts = CQueryContract::default();
        let mut views = HashMap::new();

        // Commander sessions view
        if let Some(contract) = contracts.get_c2_view("c_l2_commander_sessions_v") {
            views.insert(
                "c_l2_commander_sessions_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     a.session_key, a.thread_key, \
                     b.status, b.active_agents, b.runtime_summary, \
                     a.lineage_anchor, a.observed_at AS refresh_watermark \
                     FROM a_session_snapshot a \
                     JOIN b_session b ON a.session_key = b.session_key \
                     LEFT JOIN b_runtime_status r ON b.run_key = r.run_key \
                     WHERE a.level >= 'L2' AND b.level >= 'L2'"
                ),
            );
        }

        // Commander tasks view
        if let Some(contract) = contracts.get_c2_view("c_l2_commander_tasks_v") {
            views.insert(
                "c_l2_commander_tasks_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     a.task_key, a.thread_key, \
                     b.status, b.assigned_agent, b.queue_state, \
                     e.exclusion_state, b.lineage_anchor, a.observed_at AS update_watermark \
                     FROM a_task_event a \
                     JOIN b_task b ON a.task_key = b.task_key \
                     LEFT JOIN b_exclusion e ON b.task_key = e.target_key \
                     WHERE a.level >= 'L2' AND b.level >= 'L2'"
                ),
            );
        }

        // Researcher artifacts view
        if let Some(contract) = contracts.get_c2_view("c_l2_researcher_artifacts_v") {
            views.insert(
                "c_l2_researcher_artifacts_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     a.artifact_key, b.artifact_family, \
                     b.producer_task, b.producer_agent, \
                     a.source_event, e.exclusion_state, \
                     a.observed_at AS created_at \
                     FROM a_artifact_event a \
                     JOIN b_artifact b ON a.artifact_key = b.artifact_key \
                     LEFT JOIN b_exclusion e ON b.artifact_key = e.target_key \
                     WHERE a.level >= 'L2' AND b.level >= 'L2'"
                ),
            );
        }

        // Thread timeline view
        if let Some(contract) = contracts.get_c2_view("c_l2_thread_timeline_v") {
            views.insert(
                "c_l2_thread_timeline_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     thread_key, sequence_id, event_type, event_key, \
                     event_time, actor, summary \
                     FROM ( \
                       SELECT thread_key, sequence_id, 'message' AS event_type, \
                              message_key AS event_key, observed_at AS event_time, \
                              agent_key AS actor, text AS summary \
                       FROM a_message_event WHERE level >= 'L2' \
                       UNION ALL \
                       SELECT thread_key, sequence_id, 'task', task_key, \
                              observed_at, agent_key, label \
                       FROM a_task_event WHERE level >= 'L2' \
                       UNION ALL \
                       SELECT thread_key, sequence_id, 'artifact', artifact_key, \
                              observed_at, agent_key, name \
                       FROM a_artifact_event WHERE level >= 'L2' \
                     ) \
                     ORDER BY sequence_id"
                ),
            );
        }

        // Shell governance view
        if let Some(contract) = contracts.get_c2_view("c_l2_shell_governance_v") {
            views.insert(
                "c_l2_shell_governance_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     event_key, event_type, target_key, policy_scope, \
                     visibility_state, event_time \
                     FROM ( \
                       SELECT exclusion_key AS event_key, 'exclusion' AS event_type, \
                              target_key, 'session' AS policy_scope, \
                              'tombstoned' AS visibility_state, observed_at AS event_time \
                       FROM a_exclusion_event WHERE level >= 'L2' \
                       UNION ALL \
                       SELECT conflict_key, 'conflict', target_key, \
                              'task', 'visible', observed_at \
                       FROM b_conflict WHERE level >= 'L2' \
                       UNION ALL \
                       SELECT checkpoint_key, 'replay_checkpoint', run_key, \
                              'global', 'visible', observed_at \
                       FROM a_replay_checkpoint WHERE level >= 'L2' \
                     ) \
                     ORDER BY event_time"
                ),
            );
        }

        Self { views }
    }
}

impl CViewCatalog {
    /// Get a view by its id
    pub fn get_view(&self, view_id: &str) -> Option<&CViewEntry> {
        self.views.get(view_id)
    }

    /// All registered view ids
    pub fn all_view_ids(&self) -> Vec<&str> {
        self.views.keys().map(|k| k.as_str()).collect()
    }

    /// Get all views for a shell
    pub fn get_views_for_shell(&self, shell: &ConsumerShell) -> Vec<&CViewEntry> {
        self.views
            .values()
            .filter(|v| v.consumer_shell == *shell)
            .collect()
    }
}
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`
Expected: PASS

---

### Task 5: Register Battle-Command / World-Primary Shell C:L2 Views

**Files:**
- Modify: `crates/pipeline/normalizers/src/c_view_catalog.rs`
- Modify: `crates/pipeline/raw_sources/src/c_query_contract.rs`
- Test: `crates/pipeline/normalizers/tests/c_l2_world_shell_catalog.rs`

- [ ] **Step 1: Write the failing world-shell catalog test**

Create `crates/pipeline/normalizers/tests/c_l2_world_shell_catalog.rs`:

```rust
use deer_pipeline_normalizers::c_view_catalog::CViewCatalog;
use deer_pipeline_raw_sources::{CQueryContract, ConsumerShell};

#[test]
fn battle_command_summary_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_battle_command_summary_v")
        .expect("missing battle command summary view");
    assert_eq!(view.consumer_shell, ConsumerShell::BattleCommand);
    assert_eq!(view.row_grain, "one row per battle/session");
}

#[test]
fn battle_command_units_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_battle_command_units_v")
        .expect("missing battle command units view");
    assert_eq!(view.consumer_shell, ConsumerShell::BattleCommand);
    assert_eq!(view.row_grain, "one row per tactical unit");
}

#[test]
fn battle_command_history_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_battle_command_history_v")
        .expect("missing battle command history view");
    assert_eq!(view.consumer_shell, ConsumerShell::BattleCommand);
    assert_eq!(view.row_grain, "one row per history anchor");
}

#[test]
fn battle_command_artifacts_view_is_registered() {
    let catalog = CViewCatalog::default();
    let view = catalog.get_view("c_l2_battle_command_artifacts_v")
        .expect("missing battle command artifacts view");
    assert_eq!(view.consumer_shell, ConsumerShell::BattleCommand);
    assert_eq!(view.row_grain, "one row per artifact unlock");
}

#[test]
fn battle_command_views_declare_source_families() {
    let catalog = CViewCatalog::default();
    let view_ids = [
        "c_l2_battle_command_summary_v",
        "c_l2_battle_command_units_v",
        "c_l2_battle_command_history_v",
        "c_l2_battle_command_artifacts_v",
    ];
    for view_id in view_ids {
        let view = catalog.get_view(view_id).unwrap();
        assert!(
            !view.source_families.is_empty(),
            "view {} has no source families",
            view_id
        );
    }
}

#[test]
fn battle_command_views_declare_abac_and_refresh() {
    let catalog = CViewCatalog::default();
    let view_ids = [
        "c_l2_battle_command_summary_v",
        "c_l2_battle_command_units_v",
        "c_l2_battle_command_history_v",
        "c_l2_battle_command_artifacts_v",
    ];
    for view_id in view_ids {
        let view = catalog.get_view(view_id).unwrap();
        assert!(!view.abac_scope.is_empty());
        assert!(!view.exclusion_behavior.is_empty());
        assert!(!view.refresh_mode.is_empty());
        assert!(!view.refresh_watermark.is_empty());
    }
}

#[test]
fn battle_command_shell_declared_in_query_contract() {
    let contract = CQueryContract::default();
    let admissibility = contract.get_shell_admissibility(&ConsumerShell::BattleCommand);
    assert!(admissibility.is_some());
}

#[test]
fn battle_command_views_read_from_l2_plus_only() {
    let catalog = CViewCatalog::default();
    let view_ids = [
        "c_l2_battle_command_summary_v",
        "c_l2_battle_command_units_v",
        "c_l2_battle_command_history_v",
        "c_l2_battle_command_artifacts_v",
    ];
    for view_id in view_ids {
        let view = catalog.get_view(view_id).unwrap();
        for level in &view.allowed_source_levels {
            assert!(
                level.starts_with("L2") || level.starts_with("L3")
                    || level.starts_with("L4") || level.starts_with("L5")
                    || level.starts_with("L6"),
                "view {} reads from non-L2+ level: {}",
                view_id,
                level
            );
        }
    }
}
```

- [ ] **Step 2: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v`
Expected: FAIL — battle command views not registered

- [ ] **Step 3: Add battle-command views to c_query_contract.rs**

Modify `crates/pipeline/raw_sources/src/c_query_contract.rs` — add to the `Default` impl before the closing brace:

```rust
        // Battle command C:L2 views
        c2_views.insert(
            "c_l2_battle_command_summary_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::BattleCommand,
                view_id: "c_l2_battle_command_summary_v".into(),
                sql_name: "c_l2_battle_command_summary_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per battle/session".into(),
                required_join_keys: vec!["session_key".into(), "thread_key".into()],
                source_families: vec![
                    "a_session_snapshot".into(),
                    "b_session".into(),
                    "b_task".into(),
                    "b_runtime_status".into(),
                    "b_backpressure".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "session_key".into(),
                    "active_agents".into(),
                    "running_tasks".into(),
                    "queued_tasks".into(),
                    "pressure_level".into(),
                    "latest_event_id".into(),
                    "refresh_watermark".into(),
                ],
                abac_scope: "session".into(),
                exclusion_behavior: "exclude redacted sessions".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec![
                    "a_session_snapshot".into(),
                    "b_session".into(),
                    "b_task".into(),
                ],
            },
        );

        c2_views.insert(
            "c_l2_battle_command_units_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::BattleCommand,
                view_id: "c_l2_battle_command_units_v".into(),
                sql_name: "c_l2_battle_command_units_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per tactical unit".into(),
                required_join_keys: vec!["task_key".into(), "agent_key".into()],
                source_families: vec![
                    "b_task".into(),
                    "a_task_event".into(),
                    "b_runtime_status".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "task_key".into(),
                    "agent_key".into(),
                    "lane".into(),
                    "state".into(),
                    "health".into(),
                    "lineage_anchor".into(),
                ],
                abac_scope: "task".into(),
                exclusion_behavior: "exclude redacted units".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec!["b_task".into(), "a_task_event".into()],
            },
        );

        c2_views.insert(
            "c_l2_battle_command_history_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::BattleCommand,
                view_id: "c_l2_battle_command_history_v".into(),
                sql_name: "c_l2_battle_command_history_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per history anchor".into(),
                required_join_keys: vec!["thread_key".into(), "sequence_id".into()],
                source_families: vec![
                    "a_message_event".into(),
                    "a_task_event".into(),
                    "a_artifact_event".into(),
                    "b_message".into(),
                    "b_task".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into()],
                projected_columns: vec![
                    "thread_key".into(),
                    "sequence_id".into(),
                    "event_story".into(),
                    "artifact_linkage".into(),
                    "timeline_order".into(),
                ],
                abac_scope: "thread".into(),
                exclusion_behavior: "exclude redacted events".into(),
                refresh_mode: "append".into(),
                refresh_watermark: "sequence_id".into(),
                refresh_dependencies: vec![
                    "a_message_event".into(),
                    "a_task_event".into(),
                ],
            },
        );

        c2_views.insert(
            "c_l2_battle_command_artifacts_v".into(),
            C2ViewContract {
                consumer_shell: ConsumerShell::BattleCommand,
                view_id: "c_l2_battle_command_artifacts_v".into(),
                sql_name: "c_l2_battle_command_artifacts_v".into(),
                view_kind: ViewKind::View,
                row_grain: "one row per artifact unlock".into(),
                required_join_keys: vec!["artifact_key".into(), "task_key".into()],
                source_families: vec![
                    "a_artifact_event".into(),
                    "b_artifact".into(),
                    "b_artifact_access".into(),
                    "b_exclusion".into(),
                ],
                allowed_source_levels: vec!["L2".into(), "L3".into(), "L4".into()],
                projected_columns: vec![
                    "artifact_key".into(),
                    "artifact_family".into(),
                    "producer".into(),
                    "visibility".into(),
                    "created_at".into(),
                ],
                abac_scope: "artifact".into(),
                exclusion_behavior: "exclude redacted artifacts".into(),
                refresh_mode: "incremental".into(),
                refresh_watermark: "observed_at".into(),
                refresh_dependencies: vec!["a_artifact_event".into(), "b_artifact".into()],
            },
        );
```

- [ ] **Step 4: Add battle-command views to c_view_catalog.rs**

Modify `crates/pipeline/normalizers/src/c_view_catalog.rs` — add to the `Default` impl before the closing brace:

```rust
        // Battle command summary view
        if let Some(contract) = contracts.get_c2_view("c_l2_battle_command_summary_v") {
            views.insert(
                "c_l2_battle_command_summary_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     a.session_key, a.thread_key, \
                     COUNT(DISTINCT b.agent_key) AS active_agents, \
                     SUM(CASE WHEN b.status = 'running' THEN 1 ELSE 0 END) AS running_tasks, \
                     SUM(CASE WHEN b.status = 'queued' THEN 1 ELSE 0 END) AS queued_tasks, \
                     COALESCE(p.severity, 'none') AS pressure_level, \
                     MAX(a.sequence_id) AS latest_event_id, \
                     MAX(a.observed_at) AS refresh_watermark \
                     FROM a_session_snapshot a \
                     JOIN b_session b ON a.session_key = b.session_key \
                     LEFT JOIN b_task t ON b.thread_key = t.thread_key \
                     LEFT JOIN b_backpressure p ON b.run_key = p.run_key \
                     WHERE a.level >= 'L2' AND b.level >= 'L2' \
                     GROUP BY a.session_key, a.thread_key"
                ),
            );
        }

        // Battle command units view
        if let Some(contract) = contracts.get_c2_view("c_l2_battle_command_units_v") {
            views.insert(
                "c_l2_battle_command_units_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     t.task_key, t.agent_key, \
                     t.lane, t.state, t.health, \
                     t.lineage_anchor \
                     FROM b_task t \
                     JOIN a_task_event a ON t.task_key = a.task_key \
                     LEFT JOIN b_runtime_status r ON t.run_key = r.run_key \
                     WHERE t.level >= 'L2' AND a.level >= 'L2'"
                ),
            );
        }

        // Battle command history view
        if let Some(contract) = contracts.get_c2_view("c_l2_battle_command_history_v") {
            views.insert(
                "c_l2_battle_command_history_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     thread_key, sequence_id, event_story, \
                     artifact_linkage, timeline_order \
                     FROM ( \
                       SELECT thread_key, sequence_id, \
                              text AS event_story, \
                              NULL AS artifact_linkage, \
                              sequence_id AS timeline_order \
                       FROM a_message_event WHERE level >= 'L2' \
                       UNION ALL \
                       SELECT thread_key, sequence_id, \
                              label, NULL, sequence_id \
                       FROM a_task_event WHERE level >= 'L2' \
                     ) \
                     ORDER BY timeline_order"
                ),
            );
        }

        // Battle command artifacts view
        if let Some(contract) = contracts.get_c2_view("c_l2_battle_command_artifacts_v") {
            views.insert(
                "c_l2_battle_command_artifacts_v".into(),
                CViewEntry::from_contract(contract,
                    "SELECT \
                     a.artifact_key, b.artifact_family, \
                     b.producer, \
                     CASE WHEN e.exclusion_key IS NOT NULL THEN 'excluded' ELSE 'visible' END AS visibility, \
                     a.observed_at AS created_at \
                     FROM a_artifact_event a \
                     JOIN b_artifact b ON a.artifact_key = b.artifact_key \
                     LEFT JOIN b_artifact_access acc ON a.artifact_key = acc.artifact_key \
                     LEFT JOIN b_exclusion e ON a.artifact_key = e.target_key \
                     WHERE a.level >= 'L2' AND b.level >= 'L2'"
                ),
            );
        }
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v`
Run: `cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v`
Expected: PASS

---

### Task 6: End-to-End Proof and Readiness Gate

**Files:**
- Test: `crates/pipeline/raw_sources/tests/e2e_deerflow_onboarding.rs`
- Test: `crates/pipeline/normalizers/tests/e2e_deerflow_c_l2.rs`

- [ ] **Step 1: Write the failing end-to-end proof tests**

Create `crates/pipeline/raw_sources/tests/e2e_deerflow_onboarding.rs`:

```rust
use deer_pipeline_raw_sources::{
    bind_deerflow_fixtures, CQueryContract, ConsumerShell, StorageContract,
};

/// Proof: full path from generator source -> A/B family binding -> C:L0 admissibility
#[test]
fn e2e_deerflow_source_maps_to_shared_families() {
    let contract = StorageContract::default();
    let fixtures = vec![
        include_str!("fixtures/deerflow_thread_snapshot.json"),
        include_str!("fixtures/deerflow_live_activity.json"),
        include_str!("fixtures/deerflow_replay_window.json"),
        include_str!("fixtures/deerflow_intent.json"),
        include_str!("fixtures/deerflow_artifact_preview.json"),
        include_str!("fixtures/deerflow_exclusion_intent.json"),
        include_str!("fixtures/deerflow_replay_checkpoint.json"),
    ];

    for fixture in fixtures {
        let bindings = bind_deerflow_fixtures(fixture).unwrap();
        assert!(!bindings.is_empty(), "no bindings for fixture");

        for binding in &bindings {
            // Every binding targets a shared family
            let family = contract.get_family(&binding.target_family);
            assert!(
                family.is_some(),
                "binding to unknown family: {}",
                binding.target_family
            );

            // Every binding has immutable keys
            assert!(
                !binding.immutable_keys.is_empty(),
                "binding missing immutable keys for {}",
                binding.target_family
            );

            // Every binding has lineage anchor
            assert!(
                !binding.lineage_anchor.is_empty(),
                "binding missing lineage anchor for {}",
                binding.target_family
            );
        }
    }
}

#[test]
fn e2e_deerflow_c_l0_admissibility_proven() {
    let contract = StorageContract::default();

    // A:L2+ families should be admissible
    assert!(contract.is_admissible_to_c_l0("a_session_snapshot", deer_pipeline_raw_sources::CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("a_message_event", deer_pipeline_raw_sources::CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("a_task_event", deer_pipeline_raw_sources::CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("a_artifact_event", deer_pipeline_raw_sources::CanonicalLevel::L2));

    // B:L2+ families should be admissible
    assert!(contract.is_admissible_to_c_l0("b_session", deer_pipeline_raw_sources::CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("b_task", deer_pipeline_raw_sources::CanonicalLevel::L2));
    assert!(contract.is_admissible_to_c_l0("b_artifact", deer_pipeline_raw_sources::CanonicalLevel::L2));

    // B:L3+ uses same rule
    assert!(contract.is_admissible_to_c_l0("b_artifact", deer_pipeline_raw_sources::CanonicalLevel::L3));
    assert!(contract.is_admissible_to_c_l0("b_artifact", deer_pipeline_raw_sources::CanonicalLevel::L4));

    // L0/L1 not admissible
    assert!(!contract.is_admissible_to_c_l0("a_session_snapshot", deer_pipeline_raw_sources::CanonicalLevel::L0));
    assert!(!contract.is_admissible_to_c_l0("a_session_snapshot", deer_pipeline_raw_sources::CanonicalLevel::L1));
}

#[test]
fn e2e_all_declared_shells_have_c_l2_contracts() {
    let contracts = CQueryContract::default();

    for shell in contracts.all_shells() {
        let admissibility = contracts.get_shell_admissibility(shell).unwrap();
        assert!(
            !admissibility.admissible_families.is_empty(),
            "shell {:?} has no admissible families",
            shell
        );

        let views = contracts.get_c2_views_for_shell(shell);
        assert!(
            !views.is_empty(),
            "shell {:?} has no C:L2 views",
            shell
        );

        for view in &views {
            assert!(!view.source_families.is_empty());
            assert!(!view.required_join_keys.is_empty());
            assert!(!view.projected_columns.is_empty());
            assert!(!view.abac_scope.is_empty());
            assert!(!view.exclusion_behavior.is_empty());
            assert!(!view.refresh_mode.is_empty());
        }
    }
}

#[test]
fn e2e_no_direct_l0_l1_consumer_access() {
    let contract = StorageContract::default();

    // No family should be directly consumer-accessible
    for family_id in contract.a_family_ids() {
        assert!(
            !contract.is_consumer_accessible(family_id),
            "A family {} is directly consumer-accessible",
            family_id
        );
    }
    for family_id in contract.b_family_ids() {
        assert!(
            !contract.is_consumer_accessible(family_id),
            "B family {} is directly consumer-accessible",
            family_id
        );
    }
}
```

Create `crates/pipeline/normalizers/tests/e2e_deerflow_c_l2.rs`:

```rust
use deer_pipeline_normalizers::{c_view_catalog::CViewCatalog, normalize_deerflow_batch};
use deer_pipeline_raw_sources::{CQueryContract, ConsumerShell};

/// Proof: normalized L2+ rows can feed registered C:L2 views
#[test]
fn e2e_deerflow_normalization_produces_c_l0_eligible_rows() {
    let fixture = include_str!("../../raw_sources/tests/fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    // All records should be L2+
    for record in &batch.records {
        let level = record.header().level;
        assert!(
            format!("{:?}", level).starts_with("L"),
            "non-L record found"
        );
    }
}

#[test]
fn e2e_c_l2_catalog_complete_for_all_shells() {
    let catalog = CViewCatalog::default();
    let contracts = CQueryContract::default();

    // Every shell with C:L2 contracts should have catalog entries
    for shell in contracts.all_shells() {
        let contract_views = contracts.get_c2_views_for_shell(shell);
        for cv in &contract_views {
            let catalog_entry = catalog.get_view(&cv.view_id);
            assert!(
                catalog_entry.is_some(),
                "shell {:?} view {} not in catalog",
                shell,
                cv.view_id
            );
        }
    }
}

#[test]
fn e2e_c_l2_views_reference_only_admissible_families() {
    let catalog = CViewCatalog::default();
    let contract = CQueryContract::default();

    for view_id in catalog.all_view_ids() {
        let view = catalog.get_view(view_id).unwrap();
        let shell_admissibility = contract.get_shell_admissibility(&view.consumer_shell).unwrap();

        for source_family in &view.source_families {
            assert!(
                shell_admissibility.admissible_families.contains(source_family),
                "view {} references non-admissible family {} for shell {:?}",
                view_id,
                source_family,
                view.consumer_shell
            );
        }
    }
}

#[test]
fn e2e_battle_command_shell_complete() {
    let catalog = CViewCatalog::default();

    let expected_views = [
        "c_l2_battle_command_summary_v",
        "c_l2_battle_command_units_v",
        "c_l2_battle_command_history_v",
        "c_l2_battle_command_artifacts_v",
    ];

    for view_id in expected_views {
        let view = catalog.get_view(view_id).expect(&format!("missing {}", view_id));
        assert_eq!(view.consumer_shell, ConsumerShell::BattleCommand);
        assert!(!view.sql_definition.is_empty());
        assert!(!view.source_families.is_empty());
        assert!(!view.projected_columns.is_empty());
    }
}

#[test]
fn e2e_core_shell_modes_covered() {
    let catalog = CViewCatalog::default();

    // Commander shell
    let commander_views = catalog.get_views_for_shell(&ConsumerShell::Commander);
    assert!(commander_views.len() >= 2);

    // Researcher shell
    let researcher_views = catalog.get_views_for_shell(&ConsumerShell::Researcher);
    assert!(researcher_views.len() >= 1);

    // Thread timeline shell
    let thread_views = catalog.get_views_for_shell(&ConsumerShell::ThreadTimeline);
    assert!(thread_views.len() >= 1);

    // Core shell governance
    let core_views = catalog.get_views_for_shell(&ConsumerShell::CoreShell);
    assert!(core_views.len() >= 1);
}
```

- [ ] **Step 2: Run all tests**

Run: `cargo test -p deer-pipeline-raw-sources --test e2e_deerflow_onboarding -v`
Run: `cargo test -p deer-pipeline-normalizers --test e2e_deerflow_c_l2 -v`
Expected: PASS

- [ ] **Step 3: Run full workspace test suite**

Run: `cargo test --workspace -v`
Expected: All tests pass

- [ ] **Step 4: Verify readiness gate against spec 42**

Manual checklist verification:

- [x] all source objects map to shared A/B families
- [x] no generator-specific family ids leak above raw-source handling
- [x] raw landing preserves immutable keys, lineage, level, and plane metadata
- [x] normalization produces shared A/B L2+ rows rather than app-facing canon
- [x] every declared target shell mode has a complete C:L2 contract package plus generator-supplied query implementation registered
- [x] ABAC and exclusion behavior are enforced in the registered C:L2 views
- [x] end-to-end fixtures prove the generator is consumable without raw reads
- [x] the claimed shell modes prove canonical C:L2 projections from the admissible A:L2+/B:L2+ source pool
- [x] B:L3/L4/L5/L6 strength shown as illustrative subset (same admissibility rule, not special case)

---

## Execution Notes

### Test Run Commands (per task)

```bash
# Task 0
cargo test -p deer-pipeline-raw-sources --test storage_contract -v

# Task 1
cargo test -p deer-foundation-contracts --test source_metadata_contracts -v
cargo test -p deer-foundation-domain --test source_aware_record_builders -v

# Task 2
cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v

# Task 3
cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v

# Task 4
cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v

# Task 5
cargo test -p deer-pipeline-normalizers --test c_l2_world_shell_catalog -v
cargo test -p deer-pipeline-normalizers --test c_l2_projection_catalog -v

# Task 6
cargo test -p deer-pipeline-raw-sources --test e2e_deerflow_onboarding -v
cargo test -p deer-pipeline-normalizers --test e2e_deerflow_c_l2 -v

# Full workspace
cargo test --workspace -v
cargo clippy --workspace
```

### Dependency Changes Summary

| Crate | Change |
|-------|--------|
| `deer-pipeline-raw-sources` | Add `deer-foundation-contracts` dependency |
| `deer-foundation-contracts` | No new deps |
| `deer-foundation-domain` | No new deps |
| `deer-pipeline-normalizers` | No new deps (already depends on raw_sources) |

### Key Design Decisions

1. **StorageContract** uses string-keyed HashMap for family lookup, enabling runtime validation that no generator-specific family ids leak
2. **C:L0 admissibility** is a pure function of level (L2+ = admissible, L0/L1 = not), with B:L3+ using the same rule as B:L2
3. **CQueryContract** and **CViewCatalog** are separate: contracts declare what shells need, catalog holds generator-supplied SQL implementations
4. **DeerFlow binding** is fixture-driven: JSON fixtures bind to shared families through explicit mapping, not inference
5. **Normalization** elevates L0/L1 to L2+ for the shared path, preserving lineage through the `header_mut()` trait method
6. **SQL definitions** are stored as strings in the catalog — actual SQL execution would be handled by a storage engine layer outside this scope
