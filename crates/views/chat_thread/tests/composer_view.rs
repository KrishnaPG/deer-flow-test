use deer_pipeline_derivations::ComposerVm;
use deer_view_chat_thread::render_composer_view;
use insta::assert_yaml_snapshot;

#[test]
fn composer_view_emits_send_and_attachment_commands() {
    let vm = ComposerVm {
        mode: "prompt",
        send_state: "idle",
        prompt_text: "Survey the ridge".into(),
        attachments: vec!["upload_1".into()],
    };

    let rendered = render_composer_view(&vm);

    assert_yaml_snapshot!(rendered, @r#"
primary_command: send_message
send_state: idle
is_busy: false
attachment_count: 1
"#);
}

#[test]
fn composer_view_marks_clarification_send_as_busy() {
    let vm = ComposerVm {
        mode: "clarification_response",
        send_state: "sending_clarification_response",
        prompt_text: "Confirm the survey radius".into(),
        attachments: vec![],
    };

    let rendered = render_composer_view(&vm);

    assert_yaml_snapshot!(rendered, @r#"
primary_command: resume_clarification
send_state: sending_clarification_response
is_busy: true
attachment_count: 0
"#);
}
