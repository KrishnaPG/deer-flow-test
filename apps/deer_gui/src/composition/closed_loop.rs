use super::detail_routing::route_to_detail;
use super::scripted_scenarios::{
    FIRST_ARTIFACT_SELECTION, FIRST_PLAYABLE_CLOSED_LOOP_SCENARIO, MEDIATED_PREVIEW,
    REQUEST_SUBMITTED, STREAM_LIVE,
};
use super::world_projection_binding::bind_world_selection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableLoopResult {
    pub request_state: String,
    pub stream_state: String,
    pub artifact_access: String,
    pub world_selection: String,
    pub intervention_state: String,
    pub history_target: String,
}

pub fn run_first_playable_closed_loop() -> FirstPlayableLoopResult {
    let _scenario = FIRST_PLAYABLE_CLOSED_LOOP_SCENARIO;
    let world_selection = bind_world_selection(FIRST_ARTIFACT_SELECTION);
    let history_target = route_to_detail(&world_selection);

    FirstPlayableLoopResult {
        request_state: REQUEST_SUBMITTED.into(),
        stream_state: STREAM_LIVE.into(),
        artifact_access: MEDIATED_PREVIEW.into(),
        world_selection,
        intervention_state: REQUEST_SUBMITTED.into(),
        history_target: history_target.into(),
    }
}
