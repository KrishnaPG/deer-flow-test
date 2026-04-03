use bevy::log::debug;

pub const FIRST_ARTIFACT_SELECTION: &str = "artifact_1";
pub const SUBMITTED_REQUEST_TARGET: &str = "submitted_request";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstPlayableScenario {
    pub artifact_selection: &'static str,
    pub intervention_target: &'static str,
}

pub fn first_playable_closed_loop_scenario() -> FirstPlayableScenario {
    debug!("composition::scripted_scenarios::first_playable_closed_loop_scenario");

    FirstPlayableScenario {
        artifact_selection: FIRST_ARTIFACT_SELECTION,
        intervention_target: SUBMITTED_REQUEST_TARGET,
    }
}
