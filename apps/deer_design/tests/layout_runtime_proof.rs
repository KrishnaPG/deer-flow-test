use deer_design::run_layout_runtime_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_chat_artifact_and_inspector_views_embed_in_one_runtime_and_restore() {
    let proof = run_layout_runtime_proof();

    assert_yaml_snapshot!(proof, @r#"
mode: live_meeting
panels:
  - chat_panel
  - artifact_panel
  - inspector_panel
saved_layout: restored
selection_broker: chat_panel
"#);
}

#[test]
fn preserves_explicit_broker_role_and_explicit_hosted_panel_bindings() {
    let proof = run_layout_runtime_proof();

    assert_eq!(proof.selection_broker, "chat_panel");
    assert_eq!(proof.hosted_panels.chat, "chat_panel");
    assert_eq!(proof.hosted_panels.artifact, "artifact_panel");
    assert_eq!(proof.hosted_panels.inspector, "inspector_panel");
    assert_eq!(
        proof.linked_panel_roles.get("chat_panel").cloned(),
        Some(vec!["source".to_string(), "broker".to_string()])
    );
}
