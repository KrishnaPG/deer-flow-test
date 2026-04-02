use deer_gui::composition::run_first_playable_closed_loop;

#[test]
fn runs_submit_progress_artifact_world_select_intervention_history_loop() {
    let loop_result = run_first_playable_closed_loop();

    assert_eq!(loop_result.request_state, "submitted");
    assert_eq!(loop_result.stream_state, "live");
    assert_eq!(loop_result.artifact_access, "mediated_preview");
    assert_eq!(loop_result.world_selection, "artifact_1");
    assert_eq!(loop_result.intervention_state, "submitted");
    assert_eq!(loop_result.history_target, "artifact_detail");
}
