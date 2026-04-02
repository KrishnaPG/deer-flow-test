use deer_pipeline_derivations::orchestration::{
    RunStatusVm, TaskProgressVm, TranscriptEntryVm, TranscriptVm,
};
use deer_runtime_read_models::TemporalState;
use deer_view_chat_thread::render_transcript_view;
use insta::assert_yaml_snapshot;

#[test]
fn transcript_view_renders_clarification_rows_and_degraded_state() {
    let vm = TranscriptVm {
        entries: vec![
            TranscriptEntryVm {
                record_id: "msg_1".into(),
                role: "assistant".into(),
                text: "Scanning sector".into(),
            },
            TranscriptEntryVm {
                record_id: "clar_1".into(),
                role: "clarification".into(),
                text: "Confirm the survey radius.".into(),
            },
        ],
        run_status: RunStatusVm {
            run_id: "run_1".into(),
            state: "live".into(),
        },
        tasks: vec![TaskProgressVm {
            task_id: "task_1".into(),
            title: "Gather terrain notes".into(),
            state: "running".into(),
        }],
    };
    let temporal = TemporalState {
        mode: "live_tail",
        cursor_id: None,
        is_stale: false,
        stream_state: Some("degraded".into()),
        degraded: true,
    };

    let rendered = render_transcript_view(&vm, &temporal);

    assert_yaml_snapshot!(rendered, @r#"
row_count: 2
contains_clarification: true
degraded: true
rows:
  - kind: assistant
    text: Scanning sector
  - kind: clarification
    text: Confirm the survey radius.
"#);
}
