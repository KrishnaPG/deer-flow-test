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
