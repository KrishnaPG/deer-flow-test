use deer_runtime_read_models::{reduce_intent_state, IntentAction, IntentLifecycleState};

#[test]
fn intent_reducer_requires_explicit_stepwise_promotion() {
    let state = IntentLifecycleState::default();

    let state = reduce_intent_state(
        state,
        IntentAction::SeedFromSelection {
            source_record_id: "task_1".into(),
        },
    );
    assert_eq!(state.stage, "prefill_seed");

    let state = reduce_intent_state(state, IntentAction::OpenComposer);
    assert_eq!(state.stage, "prefill");

    let state = reduce_intent_state(state, IntentAction::TakeOwnership);
    assert_eq!(state.stage, "draft");

    let state = reduce_intent_state(state, IntentAction::Validate);
    assert_eq!(state.stage, "validated");

    let state = reduce_intent_state(state, IntentAction::Submit);
    assert_eq!(state.stage, "submitted");
}
