use deer_pipeline_derivations::TranscriptVm;
use deer_runtime_read_models::TemporalState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TranscriptRowViewState {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TranscriptViewState {
    pub row_count: usize,
    pub contains_clarification: bool,
    pub degraded: bool,
    pub rows: Vec<TranscriptRowViewState>,
}

pub fn render_transcript_view(vm: &TranscriptVm, temporal: &TemporalState) -> TranscriptViewState {
    let rows: Vec<TranscriptRowViewState> = vm
        .entries
        .iter()
        .map(|entry| TranscriptRowViewState {
            kind: entry.role.clone(),
            text: entry.text.clone(),
        })
        .collect();

    TranscriptViewState {
        row_count: rows.len(),
        contains_clarification: vm.entries.iter().any(|entry| entry.role == "clarification"),
        degraded: temporal.degraded,
        rows,
    }
}
