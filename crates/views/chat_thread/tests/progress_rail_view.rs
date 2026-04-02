use deer_pipeline_derivations::orchestration::{
    RunStatusVm, TaskProgressVm, TranscriptEntryVm, TranscriptVm,
};
use deer_runtime_read_models::TemporalState;
use deer_view_chat_thread::render_progress_rail_view;
use insta::assert_yaml_snapshot;

#[test]
fn progress_rail_view_shows_clarification_and_degradation() {
    let vm = TranscriptVm {
        entries: vec![TranscriptEntryVm {
            record_id: "clar_1".into(),
            role: "clarification".into(),
            text: "Confirm the survey radius.".into(),
        }],
        run_status: RunStatusVm {
            run_id: "run_1".into(),
            state: "live".into(),
        },
        tasks: vec![
            TaskProgressVm {
                task_id: "task_1".into(),
                title: "Gather terrain notes".into(),
                state: "running".into(),
            },
            TaskProgressVm {
                task_id: "task_2".into(),
                title: "Prepare summary".into(),
                state: "pending".into(),
            },
        ],
    };
    let temporal = TemporalState {
        mode: "live_tail",
        cursor_id: None,
        is_stale: false,
        stream_state: Some("degraded".into()),
        degraded: true,
    };

    let rendered = render_progress_rail_view(&vm, &temporal);

    assert_yaml_snapshot!(rendered, @r#"
run_state: live
task_count: 2
contains_clarification_history: true
banner: connection_degraded
"#);
}

#[test]
fn progress_rail_view_marks_historical_state_as_not_live() {
    let vm = TranscriptVm {
        entries: vec![],
        run_status: RunStatusVm {
            run_id: "run_1".into(),
            state: "running".into(),
        },
        tasks: vec![],
    };
    let temporal = TemporalState::historical("checkpoint_7");

    let rendered = render_progress_rail_view(&vm, &temporal);

    assert_yaml_snapshot!(rendered, @r#"
run_state: running
task_count: 0
contains_clarification_history: false
banner: historical
"#);
}

#[test]
fn progress_rail_view_preserves_historical_context_when_stale_and_degraded() {
    let vm = TranscriptVm {
        entries: vec![],
        run_status: RunStatusVm {
            run_id: "run_1".into(),
            state: "running".into(),
        },
        tasks: vec![],
    };
    let temporal = TemporalState {
        mode: "historical",
        cursor_id: Some("checkpoint_7".into()),
        is_stale: true,
        stream_state: Some("degraded".into()),
        degraded: true,
    };

    let rendered = render_progress_rail_view(&vm, &temporal);

    assert_yaml_snapshot!(rendered, @r#"
run_state: running
task_count: 0
contains_clarification_history: false
banner: historical_stale
"#);
}
