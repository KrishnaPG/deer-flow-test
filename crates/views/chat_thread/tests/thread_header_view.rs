use deer_pipeline_derivations::ThreadHeaderVm;
use deer_view_chat_thread::render_thread_header_view;
use insta::assert_yaml_snapshot;

#[test]
fn thread_header_view_exposes_resume_and_refresh_commands() {
    let vm = ThreadHeaderVm {
        thread_id: "thread_1".into(),
        title: "Survey the ridge".into(),
        stream_state: "live".into(),
    };

    let rendered = render_thread_header_view(&vm);

    assert_yaml_snapshot!(rendered, @r#"
thread_id: thread_1
title: Survey the ridge
stream_state: live
primary_command: resume_thread
secondary_command: refresh_stream
"#);
}
