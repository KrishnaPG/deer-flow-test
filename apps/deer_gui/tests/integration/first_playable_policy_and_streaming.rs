use deer_gui::composition::closed_loop::{
    ArtifactAccess, IntentSeedState, PolicyState, StreamState,
};
use deer_gui::composition::scripted_scenarios::CLEARED_SELECTION;
use deer_gui::composition::{
    run_degraded_first_playable_closed_loop, run_first_playable_closed_loop,
};

#[test]
fn keeps_prefill_seed_policy_and_stream_state_safe_in_first_playable_loop() {
    let loop_result = run_first_playable_closed_loop();

    assert_eq!(loop_result.stream_state, StreamState::Live);
    assert_eq!(loop_result.artifact_access, ArtifactAccess::MediatedPreview);
    assert_eq!(loop_result.intent_seed_state, IntentSeedState::PrefillSeed);
    assert_eq!(loop_result.policy_state, PolicyState::Allowed);
    assert!(!loop_result.degraded_stream_visible);
    assert_eq!(loop_result.world_selection, "artifact_1");
}

#[test]
fn surfaces_degraded_stream_and_clears_selection_on_policy_invalidation() {
    let loop_result = run_degraded_first_playable_closed_loop();

    assert_eq!(loop_result.stream_state, StreamState::Degraded);
    assert!(loop_result.degraded_stream_visible);
    assert_eq!(loop_result.policy_state, PolicyState::Invalidated);
    assert_eq!(loop_result.world_selection, CLEARED_SELECTION);
    assert_eq!(
        loop_result.world_selection_sources,
        [CLEARED_SELECTION, CLEARED_SELECTION]
    );
}
