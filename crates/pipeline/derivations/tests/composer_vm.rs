use deer_pipeline_derivations::derive_composer_vm;
use deer_runtime_read_models::{ChatDraftState, ClarificationState, OptimisticSendState};
use insta::assert_yaml_snapshot;

#[test]
fn derives_composer_vm_with_clarification_mode_and_optimistic_send_state() {
    let chat_state = ChatDraftState {
        prompt_text: "Confirm the survey radius".into(),
        attachment_ids: vec!["upload_1".into()],
        clarification_state: ClarificationState::Responding,
        optimistic_send: OptimisticSendState::SendingClarificationResponse,
        ..Default::default()
    };

    let vm = derive_composer_vm(&chat_state);

    assert_yaml_snapshot!(vm, @r#"
mode: clarification_response
send_state: sending_clarification_response
prompt_text: Confirm the survey radius
attachments:
  - upload_1
"#);
}
