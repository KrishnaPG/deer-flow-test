use deer_gui::composition::closed_loop::{
    ArtifactAccess, InterventionState, LoopStep, RequestState, StreamState,
};
use deer_gui::composition::run_first_playable_closed_loop;

#[test]
fn runs_submit_progress_artifact_world_select_intervention_history_loop() {
    let loop_result = run_first_playable_closed_loop();

    assert_eq!(
        loop_result.scenario_steps,
        [
            LoopStep::Submit,
            LoopStep::Progress,
            LoopStep::Artifact,
            LoopStep::WorldSelect,
            LoopStep::Intervention,
            LoopStep::History,
        ]
    );
    assert_eq!(loop_result.request_state, RequestState::Submitted);
    assert_eq!(loop_result.stream_state, StreamState::Live);
    assert_eq!(loop_result.artifact_access, ArtifactAccess::MediatedPreview);
    assert_eq!(loop_result.artifact_detail_source, "artifact_shelf_view");
    assert_eq!(loop_result.world_selection, "artifact_1");
    assert_eq!(
        loop_result.world_selection_sources,
        ["world_scene_view", "minimap_view"]
    );
    assert_eq!(loop_result.intervention_state, InterventionState::Submitted);
    assert_eq!(loop_result.intervention_target, "submitted_request");
    assert_eq!(loop_result.history_target, "artifact_detail");
    assert_eq!(loop_result.history_source, "selection_broker");
}
