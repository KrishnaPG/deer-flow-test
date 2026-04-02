use deer_ui_panel_shells::{PanelContract, PanelParticipation, PanelRole};

#[test]
fn panel_participation_cannot_be_constructed_directly() {
    let cases = trybuild::TestCases::new();

    cases.compile_fail("tests/ui/panel_participation_private_fields.rs");
}

#[test]
fn panel_contract_requires_hosted_views_roles_and_join_keys() {
    let contract = PanelContract {
        panel_id: "artifact_shelf".into(),
        required_hosted_views: vec!["artifact_shelf_view".into()],
        roles: vec![PanelRole::Source, PanelRole::Sink, PanelRole::Mirror],
        join_keys: vec!["artifact_id".into(), "thread_id".into()],
    };

    let participation = PanelParticipation::from_contract(&contract).unwrap();

    assert!(participation.roles().contains(&PanelRole::Source));
    assert_eq!(
        participation.join_keys(),
        vec!["artifact_id".to_string(), "thread_id".to_string()]
    );
    assert_eq!(
        participation.required_hosted_views(),
        vec!["artifact_shelf_view".to_string()]
    );
}

#[test]
fn panel_contract_rejects_participation_without_required_hosted_views() {
    let contract = PanelContract {
        panel_id: "artifact_shelf".into(),
        required_hosted_views: Vec::new(),
        roles: vec![PanelRole::Source],
        join_keys: vec!["artifact_id".into()],
    };

    let error = PanelParticipation::from_contract(&contract).unwrap_err();

    assert_eq!(
        error,
        "panel participation requires hosted views, declared roles, and join keys"
    );
}
