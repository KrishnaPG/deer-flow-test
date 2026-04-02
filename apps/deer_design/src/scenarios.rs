use deer_pipeline_derivations::{
    ArtifactEntryVm, ArtifactShelfVm, RunStatusVm, TranscriptEntryVm, TranscriptVm,
};
use deer_runtime_read_models::TemporalState;
use deer_view_chat_thread::{render_transcript_view, transcript_view::TranscriptViewState};
use deer_view_list_detail::{
    artifact_shelf_view::ArtifactShelfViewState, render_artifact_shelf_view,
};

pub struct RuntimeScenario {
    pub transcript: TranscriptViewState,
    pub artifacts: ArtifactShelfViewState,
    pub temporal: TemporalState,
}

pub fn live_meeting_runtime_scenario() -> RuntimeScenario {
    let temporal = TemporalState {
        mode: "live_tail",
        stream_state: Some("live".into()),
        ..TemporalState::default()
    };
    let transcript_vm = TranscriptVm {
        entries: vec![TranscriptEntryVm {
            record_id: "message_1".into(),
            role: "assistant".into(),
            text: "Layout runtime proof is live.".into(),
        }],
        run_status: RunStatusVm {
            run_id: "run_1".into(),
            state: "live".into(),
        },
        tasks: Vec::new(),
    };
    let artifact_vm = ArtifactShelfVm {
        entries: vec![ArtifactEntryVm {
            artifact_id: "artifact_1".into(),
            title: "Runtime sketch".into(),
            status: "presented".into(),
            preview_supported: true,
            retrieval_mode: "mediated_pointer",
            provenance: None,
            presentation_state: "presented",
        }],
    };

    RuntimeScenario {
        transcript: render_transcript_view(&transcript_vm, &temporal),
        artifacts: render_artifact_shelf_view(&artifact_vm),
        temporal,
    }
}
