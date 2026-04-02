use deer_pipeline_derivations::TranscriptVm;
use deer_runtime_read_models::TemporalState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProgressRailViewState {
    pub run_state: String,
    pub task_count: usize,
    pub contains_clarification_history: bool,
    pub banner: &'static str,
}

pub fn render_progress_rail_view(
    vm: &TranscriptVm,
    temporal: &TemporalState,
) -> ProgressRailViewState {
    ProgressRailViewState {
        run_state: vm.run_status.state.clone(),
        task_count: vm.tasks.len(),
        contains_clarification_history: vm
            .entries
            .iter()
            .any(|entry| entry.role == "clarification"),
        banner: if temporal.mode == "historical" {
            if temporal.is_stale {
                "historical_stale"
            } else {
                "historical"
            }
        } else if temporal.degraded {
            "connection_degraded"
        } else {
            "live"
        },
    }
}
