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

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `normalizers` (catalog) + `raw_sources` (query contract) |
| 2 | Contract being defined | Battle-command/world-primary C:L2 view contracts + SQL (4 views: summary, units, history, artifacts) |
| 3 | Failing test or fixture first | `c_l2_world_shell_catalog.rs` — 8 tests covering all battle-command views, L2+ only, ABAC/refresh declared |
| 4 | Proof app before `deer_gui` | `deer_chat_lab` — load catalog, assert battle-command views present |
| 5 | Forbidden dependencies | `deer_gui`, `scene3d`, `world_projection` crate, any renderer |
| 7 | World mapping rationale | This IS world mapping — but belongs in `c_view_catalog` as a **query contract**, not a renderer. The `world_projection` crate consumes these C:L2 views to build scene/hud VMs; separation preserved. |
| 6 | Milestone gate unlocked | Gate B — world-primary shell has C:L2 contracts |
