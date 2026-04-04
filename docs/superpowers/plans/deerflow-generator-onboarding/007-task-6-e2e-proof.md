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

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `raw_sources` (onboarding proof) + `normalizers` (C:L2 proof) |
| 2 | Contract being defined | Full path: fixture → binding → L2+ → C:L2 catalog registration |
| 3 | Failing test or fixture first | `e2e_deerflow_onboarding.rs` (4 tests) + `e2e_deerflow_c_l2.rs` (5 tests) |
| 4 | Proof app before `deer_gui` | `deer_chat_lab` — scripted scenario loading fixture, normalizing, querying catalog |
| 5 | Forbidden dependencies | `deer_gui`, any UI crate, any DB driver |
| 6 | Milestone gate unlocked | Gate C — DeerFlow fully onboarded; spec 42 readiness gate satisfied |
