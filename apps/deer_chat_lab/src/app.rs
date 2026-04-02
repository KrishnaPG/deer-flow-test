use deer_pipeline_derivations::{
    derive_artifact_shelf_vm, derive_composer_vm, derive_thread_header_vm, derive_transcript_vm,
};
use deer_pipeline_normalizers::normalize_stream_batch;
use deer_pipeline_raw_sources::{
    load_stream_fixture, preview_artifact, resume_thread, stage_upload,
};
use deer_runtime_read_models::{
    reduce_chat_state, reduce_temporal_state, ChatAction, ChatDraftState, TemporalAction,
    TemporalState,
};
use deer_view_chat_thread::{
    render_composer_view, render_progress_rail_view, render_thread_header_view,
    render_transcript_view,
};
use deer_view_list_detail::{
    render_artifact_shelf_view, render_file_presenter_view, FilePresenterInput,
};
use serde::Serialize;

use crate::scenarios::live_chat_scenario;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveChatProof {
    pub thread: ThreadProof,
    pub uploads: Vec<String>,
    pub stream: StreamProof,
    pub clarification: ClarificationProof,
    pub artifacts: Vec<ArtifactProof>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreadProof {
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StreamProof {
    pub state: String,
    pub tasks: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClarificationProof {
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactProof {
    pub artifact_id: String,
    pub access: &'static str,
}

pub fn run_live_chat_proof() -> LiveChatProof {
    let scenario = live_chat_scenario();
    let thread = resume_thread(scenario.thread_fixture).expect("thread fixture should parse");
    let upload =
        stage_upload(&thread.thread_id, scenario.upload_name).expect("upload should stage");
    let raw_events =
        load_stream_fixture(scenario.stream_fixture).expect("stream fixture should parse");
    let normalized =
        normalize_stream_batch("session_1", &thread.title, "run_1", "running", &raw_events)
            .expect("stream should normalize");

    let transcript_vm = derive_transcript_vm(&normalized.records);
    let artifact_shelf_vm = derive_artifact_shelf_vm(&normalized.records);
    let temporal =
        reduce_temporal_state(TemporalState::default(), TemporalAction::ReturnToLiveTail);
    let header = render_thread_header_view(&derive_thread_header_vm(
        &thread.thread_id,
        &thread.title,
        temporal.stream_state.as_deref().unwrap_or("idle"),
    ));
    let transcript = render_transcript_view(&transcript_vm, &temporal);
    let progress = render_progress_rail_view(&transcript_vm, &temporal);

    let mut chat = reduce_chat_state(
        ChatDraftState::default(),
        ChatAction::AddAttachment {
            attachment_id: upload.file_name.clone(),
        },
    );
    chat = reduce_chat_state(
        chat,
        ChatAction::StreamStateChanged {
            state: header.stream_state.clone(),
        },
    );
    if transcript.contains_clarification {
        chat = reduce_chat_state(chat, ChatAction::ClarificationRequested);
        chat = reduce_chat_state(chat, ChatAction::ClarificationResponseStarted);
    }
    let composer = render_composer_view(&derive_composer_vm(&chat));
    let artifact_shelf = render_artifact_shelf_view(&artifact_shelf_vm);

    LiveChatProof {
        thread: ThreadProof { state: "resumed" },
        uploads: vec![upload.file_name],
        stream: StreamProof {
            state: header.stream_state,
            tasks: transcript_vm
                .tasks
                .iter()
                .map(|task| task.title.clone())
                .collect(),
            tools: transcript_vm
                .entries
                .iter()
                .filter(|entry| entry.role == "tool")
                .map(|entry| entry.text.clone())
                .collect(),
        },
        clarification: ClarificationProof {
            state: if composer.primary_command == "resume_clarification"
                && composer.attachment_count == 1
                && composer.is_busy
                && progress.contains_clarification_history
            {
                "resumed"
            } else {
                "idle"
            },
        },
        artifacts: artifact_shelf
            .items
            .into_iter()
            .filter_map(|item| {
                if item.preview_action != Some("preview_artifact") {
                    return None;
                }

                let deer_pipeline_raw_sources::ArtifactAccess::PreviewPayload { mime } =
                    preview_artifact(&thread.thread_id, &item.artifact_id).ok()?
                else {
                    return None;
                };
                let presenter = render_file_presenter_view(&FilePresenterInput::Preview { mime });

                Some(ArtifactProof {
                    artifact_id: item.artifact_id,
                    access: if presenter.mode == "preview" {
                        "preview"
                    } else {
                        "pointer"
                    },
                })
            })
            .collect(),
    }
}
