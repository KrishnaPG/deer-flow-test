use bevy::log::debug;

use super::detail_routing::route_to_detail;
use super::scripted_scenarios::{
    degraded_first_playable_scenario, first_playable_closed_loop_scenario, CLEARED_SELECTION,
};
use super::view_hosts::ARTIFACT_HOST;
use super::world_projection_binding::bind_world_selection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopStep {
    Submit,
    Progress,
    Artifact,
    WorldSelect,
    Intervention,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestState {
    Submitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Live,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAccess {
    MediatedPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionState {
    Submitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentSeedState {
    PrefillSeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyState {
    Allowed,
    Invalidated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableLoopResult {
    pub scenario_steps: [LoopStep; 6],
    pub request_state: RequestState,
    pub stream_state: StreamState,
    pub artifact_access: ArtifactAccess,
    pub artifact_detail_source: &'static str,
    pub world_selection: &'static str,
    pub world_selection_sources: [&'static str; 2],
    pub intervention_state: InterventionState,
    pub intervention_target: &'static str,
    pub history_target: &'static str,
    pub history_source: &'static str,
    pub intent_seed_state: IntentSeedState,
    pub policy_state: PolicyState,
    pub degraded_stream_visible: bool,
}

fn build_loop_result(
    scenario: super::scripted_scenarios::FirstPlayableScenario,
) -> FirstPlayableLoopResult {
    let world_selection = if scenario.policy_invalidates_selection {
        CLEARED_SELECTION
    } else {
        scenario.artifact_selection
    };
    let world_selection_sources = if scenario.policy_invalidates_selection {
        [CLEARED_SELECTION, CLEARED_SELECTION]
    } else {
        bind_world_selection(scenario.artifact_selection).source_hosts
    };
    let history_route = route_to_detail(scenario.artifact_selection);

    FirstPlayableLoopResult {
        scenario_steps: [
            LoopStep::Submit,
            LoopStep::Progress,
            LoopStep::Artifact,
            LoopStep::WorldSelect,
            LoopStep::Intervention,
            LoopStep::History,
        ],
        request_state: RequestState::Submitted,
        stream_state: if scenario.degraded_stream_visible {
            StreamState::Degraded
        } else {
            StreamState::Live
        },
        artifact_access: ArtifactAccess::MediatedPreview,
        artifact_detail_source: ARTIFACT_HOST,
        world_selection,
        world_selection_sources,
        intervention_state: InterventionState::Submitted,
        intervention_target: scenario.intervention_target,
        history_target: history_route.target,
        history_source: history_route.source,
        intent_seed_state: IntentSeedState::PrefillSeed,
        policy_state: if scenario.policy_invalidates_selection {
            PolicyState::Invalidated
        } else {
            PolicyState::Allowed
        },
        degraded_stream_visible: scenario.degraded_stream_visible,
    }
}

pub fn run_first_playable_closed_loop() -> FirstPlayableLoopResult {
    debug!("composition::closed_loop::run_first_playable_closed_loop");

    build_loop_result(first_playable_closed_loop_scenario())
}

pub fn run_degraded_first_playable_closed_loop() -> FirstPlayableLoopResult {
    debug!("composition::closed_loop::run_degraded_first_playable_closed_loop");

    build_loop_result(degraded_first_playable_scenario())
}
