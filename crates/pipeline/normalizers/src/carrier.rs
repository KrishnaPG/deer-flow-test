use deer_foundation_contracts::{AsIsHash, RecordId};
use deer_foundation_domain::{
    AnyRecord, ArtifactBody, ArtifactRecord, ClarificationBody, ClarificationRecord, MessageBody,
    MessageRecord, RunBody, RunRecord, RuntimeStatusBody, RuntimeStatusRecord, SessionBody,
    SessionRecord, TaskBody, TaskRecord,
};

use crate::{envelopes::RawEnvelopeBatch, error::NormalizationError};

fn infer_media_type(name: &str) -> &'static str {
    if name.ends_with(".md") {
        "text/markdown"
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
            crate::envelopes::RawEventEnvelope::Message {
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
            crate::envelopes::RawEventEnvelope::Task {
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
            crate::envelopes::RawEventEnvelope::Artifact {
                artifact_id,
                name,
                status,
                as_is_hash,
            } => records.push(AnyRecord::Artifact(ArtifactRecord::new(
                RecordId::from(artifact_id.as_str()),
                deer_foundation_contracts::IdentityMeta::hash_anchored(
                    RecordId::from(artifact_id.as_str()),
                    Some(AsIsHash::from(as_is_hash.as_str())),
                    None,
                    None,
                ),
                deer_foundation_contracts::LineageMeta::root(),
                ArtifactBody {
                    label: name.clone(),
                    media_type: infer_media_type(name).to_string(),
                },
            ))),
            crate::envelopes::RawEventEnvelope::RuntimeStatus { state } => {
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
            crate::envelopes::RawEventEnvelope::Clarification {
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
