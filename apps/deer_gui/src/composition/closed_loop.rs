use bevy::log::debug;

use super::detail_routing::route_to_detail;
use super::scripted_scenarios::first_playable_closed_loop_scenario;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAccess {
    MediatedPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionState {
    Submitted,
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
}

pub fn run_first_playable_closed_loop() -> FirstPlayableLoopResult {
    debug!("composition::closed_loop::run_first_playable_closed_loop");

    let scenario = first_playable_closed_loop_scenario();
    let world_selection = bind_world_selection(scenario.artifact_selection);
    let history_route = route_to_detail(&world_selection.selection_id);

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
        stream_state: StreamState::Live,
        artifact_access: ArtifactAccess::MediatedPreview,
        artifact_detail_source: ARTIFACT_HOST,
        world_selection: world_selection.selection_id,
        world_selection_sources: world_selection.source_hosts,
        intervention_state: InterventionState::Submitted,
        intervention_target: scenario.intervention_target,
        history_target: history_route.target,
        history_source: history_route.source,
    }
}
