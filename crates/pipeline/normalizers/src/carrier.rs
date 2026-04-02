use deer_foundation_contracts::{AsIsHash, RecordId};
use deer_foundation_domain::{
    AnyRecord, ArtifactBody, ArtifactRecord, ClarificationBody, ClarificationRecord, MessageBody,
    MessageRecord, RunBody, RunRecord, RuntimeStatusBody, RuntimeStatusRecord, SessionBody,
    SessionRecord, TaskBody, TaskRecord, ToolCallBody, ToolCallRecord,
};
use deer_pipeline_raw_sources::{AdapterEvent, RawStreamEvent};

use crate::{
    envelopes::{RawEnvelopeBatch, RawEventEnvelope},
    error::NormalizationError,
};

fn infer_media_type(name: &str) -> &'static str {
    if name.ends_with(".md") {
        "text/markdown"
    } else if name.ends_with(".png") {
        "image/png"
    } else if name.ends_with(".jpg") || name.ends_with(".jpeg") {
        "image/jpeg"
    } else if name.ends_with(".gif") {
        "image/gif"
    } else if name.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream"
    }
}

#[derive(Debug, Clone, Default)]
pub struct NormalizedBatch {
    pub records: Vec<AnyRecord>,
}

pub fn normalize_batch(batch: &RawEnvelopeBatch) -> Result<NormalizedBatch, NormalizationError> {
    let (session, run, events) = match batch {
        RawEnvelopeBatch::Deerflow(batch) => (&batch.session, &batch.run, &batch.events),
        RawEnvelopeBatch::Hermes(batch) => (&batch.session, &batch.run, &batch.events),
    };

    let mut records = vec![
        AnyRecord::Session(SessionRecord::new(
            RecordId::from(session.session_id.as_str()),
            deer_foundation_contracts::IdentityMeta::hash_anchored(
                RecordId::from(session.session_id.as_str()),
                None,
                None,
                None,
            ),
            deer_foundation_contracts::LineageMeta::root(),
            SessionBody {
                name: session.title.clone(),
            },
        )),
        AnyRecord::Run(RunRecord::new(
            RecordId::from(run.run_id.as_str()),
            deer_foundation_contracts::IdentityMeta::hash_anchored(
                RecordId::from(run.run_id.as_str()),
                None,
                None,
                None,
            ),
            deer_foundation_contracts::LineageMeta::root(),
            RunBody {
                title: session.title.clone(),
                status: run.status.clone(),
            },
        )),
    ];

    for event in events {
        match event {
            RawEventEnvelope::Message {
                message_id,
                role,
                text,
                level: _,
            } => records.push(AnyRecord::Message(MessageRecord::new(
                RecordId::from(message_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(message_id.as_str()),
                    None,
                    None,
                    None,
                ),
                deer_foundation_contracts::LineageMeta::root(),
                MessageBody {
                    role: role.clone(),
                    text: text.clone(),
                },
            ))),
            RawEventEnvelope::Task {
                task_id,
                title,
                state,
            } => records.push(AnyRecord::Task(TaskRecord::new(
                RecordId::from(task_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(task_id.as_str()),
                    None,
                    None,
                    None,
                ),
                deer_foundation_contracts::LineageMeta::root(),
                TaskBody {
                    label: title.clone(),
                    status: state.clone(),
                },
            ))),
            RawEventEnvelope::ToolCall {
                tool_call_id,
                tool_name,
                status,
            } => records.push(AnyRecord::ToolCall(ToolCallRecord::new(
                RecordId::from(tool_call_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(tool_call_id.as_str()),
                    None,
                    None,
                    None,
                ),
                deer_foundation_contracts::LineageMeta::root(),
                ToolCallBody {
                    tool_name: tool_name.clone(),
                    status: status.clone(),
                },
            ))),
            RawEventEnvelope::Artifact {
                artifact_id,
                name,
                status: _,
                as_is_hash,
                parent_message_id,
                parent_clarification_id,
            } => records.push(AnyRecord::Artifact(ArtifactRecord::new(
                RecordId::from(artifact_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(artifact_id.as_str()),
                    Some(AsIsHash::from(as_is_hash.as_str())),
                    None,
                    None,
                ),
                artifact_lineage(
                    parent_message_id.as_deref(),
                    parent_clarification_id.as_deref(),
                ),
                ArtifactBody {
                    label: name.clone(),
                    media_type: infer_media_type(name).to_string(),
                },
            ))),
            RawEventEnvelope::RuntimeStatus { state } => {
                records.push(AnyRecord::RuntimeStatus(RuntimeStatusRecord::new(
                    RecordId::from(format!("runtime_status:{state}")),
                    deer_foundation_contracts::IdentityMeta::hash_anchored(
                        RecordId::from(format!("runtime_status:{state}")),
                        None,
                        None,
                        None,
                    ),
                    deer_foundation_contracts::LineageMeta::root(),
                    RuntimeStatusBody {
                        status: state.clone(),
                        detail: String::new(),
                    },
                )))
            }
            RawEventEnvelope::Clarification {
                clarification_id,
                prompt,
            } => records.push(AnyRecord::Clarification(ClarificationRecord::new(
                RecordId::from(clarification_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(clarification_id.as_str()),
                    None,
                    None,
                    None,
                ),
                deer_foundation_contracts::LineageMeta::root(),
                ClarificationBody {
                    prompt: prompt.clone(),
                    resolved: false,
                },
            ))),
        }
    }

    Ok(NormalizedBatch { records })
}

pub fn normalize_stream_batch(
    session_id: &str,
    title: &str,
    run_id: &str,
    run_status: &str,
    events: &[AdapterEvent],
) -> Result<NormalizedBatch, NormalizationError> {
    let event_envelopes = events.iter().map(raw_event_to_envelope).collect::<Vec<_>>();

    normalize_batch(&RawEnvelopeBatch::Deerflow(
        crate::envelopes::DeerFlowBatch {
            session: crate::envelopes::RawSessionEnvelope {
                session_id: session_id.to_string(),
                title: title.to_string(),
            },
            run: crate::envelopes::RawRunEnvelope {
                run_id: run_id.to_string(),
                status: run_status.to_string(),
            },
            events: event_envelopes,
        },
    ))
}

fn artifact_lineage(
    parent_message_id: Option<&str>,
    parent_clarification_id: Option<&str>,
) -> deer_foundation_contracts::LineageMeta {
    if let Some(message_id) = parent_message_id {
        deer_foundation_contracts::LineageMeta::derived_from(
            deer_foundation_contracts::RecordRef::new(
                deer_foundation_contracts::RecordFamily::Message,
                RecordId::from(message_id),
            ),
        )
    } else if let Some(clarification_id) = parent_clarification_id {
        deer_foundation_contracts::LineageMeta::derived_from(
            deer_foundation_contracts::RecordRef::new(
                deer_foundation_contracts::RecordFamily::Clarification,
                RecordId::from(clarification_id),
            ),
        )
    } else {
        deer_foundation_contracts::LineageMeta::root()
    }
}

fn raw_event_to_envelope(event: &AdapterEvent) -> RawEventEnvelope {
    match event {
        AdapterEvent::Deerflow(raw) | AdapterEvent::Hermes(raw) => match raw {
            RawStreamEvent::MessageDelta { message_id, text } => RawEventEnvelope::Message {
                message_id: message_id.clone(),
                role: "assistant".into(),
                text: text.clone(),
                level: "L2".into(),
            },
            RawStreamEvent::ToolCall {
                tool_call_id,
                tool_name,
            } => RawEventEnvelope::ToolCall {
                tool_call_id: tool_call_id.clone(),
                tool_name: tool_name.clone(),
                status: "running".into(),
            },
            RawStreamEvent::TaskProgress {
                task_id,
                state,
                label,
            } => RawEventEnvelope::Task {
                task_id: task_id.clone(),
                title: label.clone(),
                state: state.clone(),
            },
            RawStreamEvent::Clarification {
                clarification_id,
                prompt,
            } => RawEventEnvelope::Clarification {
                clarification_id: clarification_id.clone(),
                prompt: prompt.clone(),
            },
            RawStreamEvent::ArtifactPresented { artifact_id, name } => RawEventEnvelope::Artifact {
                artifact_id: artifact_id.clone(),
                name: name.clone(),
                status: "presented".into(),
                as_is_hash: format!("sha256:{artifact_id}"),
                parent_message_id: Some("msg_2".into()),
                parent_clarification_id: None,
            },
            RawStreamEvent::RuntimeStatus { state } => RawEventEnvelope::RuntimeStatus {
                state: state.clone(),
            },
        },
    }
}
