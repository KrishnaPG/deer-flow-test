use bevy::log::debug;

pub const FIRST_ARTIFACT_SELECTION: &str = "artifact_1";
pub const SUBMITTED_REQUEST_TARGET: &str = "submitted_request";
pub const CLEARED_SELECTION: &str = "";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstPlayableScenario {
    pub artifact_selection: &'static str,
    pub intervention_target: &'static str,
    pub degraded_stream_visible: bool,
    pub policy_invalidates_selection: bool,
}

pub fn first_playable_closed_loop_scenario() -> FirstPlayableScenario {
    debug!("composition::scripted_scenarios::first_playable_closed_loop_scenario");

    FirstPlayableScenario {
        artifact_selection: FIRST_ARTIFACT_SELECTION,
        intervention_target: SUBMITTED_REQUEST_TARGET,
        degraded_stream_visible: false,
        policy_invalidates_selection: false,
    }
}

pub fn degraded_first_playable_scenario() -> FirstPlayableScenario {
    debug!("composition::scripted_scenarios::degraded_first_playable_scenario");

    FirstPlayableScenario {
        artifact_selection: FIRST_ARTIFACT_SELECTION,
        intervention_target: SUBMITTED_REQUEST_TARGET,
        degraded_stream_visible: true,
        policy_invalidates_selection: true,
    }
}
