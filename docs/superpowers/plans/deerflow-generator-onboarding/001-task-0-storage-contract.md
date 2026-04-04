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

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `raw_sources` |
| 2 | Contract being defined | Shared A/B family registry (22 families); C:L0 admissibility rules; C:L2 view contract metadata |
| 3 | Failing test or fixture first | `crates/pipeline/raw_sources/tests/storage_contract.rs` — 8 tests covering all families, L0/L1 blocking, B:L3+ same rule |
| 4 | Proof app before `deer_gui` | None — pure contract definition, validated downstream by Tasks 2-6 |
| 5 | Forbidden dependencies | `deer_gui`, `world_projection`, `scene3d`, any UI crate |
| 6 | Milestone gate unlocked | Gate A — shared storage contract exists; all downstream tasks can now bind |
