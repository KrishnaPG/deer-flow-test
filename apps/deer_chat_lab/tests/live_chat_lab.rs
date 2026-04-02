use deer_chat_lab::run_live_chat_proof;
use insta::assert_yaml_snapshot;

#[test]
fn proves_create_upload_stream_clarification_and_artifact_open_flow() {
    let proof = run_live_chat_proof();

    assert_yaml_snapshot!(proof, @r#"
thread:
  state: resumed
uploads:
  - briefing.pdf
stream:
  state: live
  tasks:
    - Gather terrain notes
  tools:
    - map_scan
clarification:
  state: resumed
artifacts:
  - artifact_id: artifact_2
    access: preview
"#);
}
