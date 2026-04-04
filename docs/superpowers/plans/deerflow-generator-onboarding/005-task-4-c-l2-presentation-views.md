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

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `normalizers` (`c_view_catalog`) |
| 2 | Contract being defined | Generator-supplied SQL for shell-defined C:L2 views (5 views: commander sessions, commander tasks, researcher artifacts, thread timeline, shell governance) |
| 3 | Failing test or fixture first | `c_l2_projection_catalog.rs` — 13 tests covering all views, complete metadata, L2+ only |
| 4 | Proof app before `deer_gui` | `deer_chat_lab` — load catalog, assert all views registered with valid SQL |
| 5 | Forbidden dependencies | `deer_gui`, `scene3d`, any DB driver (SQL is declarative only) |
| 6 | Milestone gate unlocked | Gate B — presentation contracts defined |
