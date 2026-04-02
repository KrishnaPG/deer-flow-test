use deer_foundation_contracts::{AsIsHash, CanonicalLevel, CanonicalPlane, RecordId};
use deer_foundation_domain::{
    AnyRecord, ArtifactRecord, ClarificationRecord, MessageRecord, RunRecord, RuntimeStatusRecord,
    SessionRecord, TaskRecord,
};

use crate::{envelopes::RawEnvelopeBatch, error::NormalizationError};

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
            session.title.clone(),
        )),
        AnyRecord::Run(RunRecord::new(
            RecordId::from(run.run_id.as_str()),
            run.status.clone(),
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
                role.clone(),
                text.clone(),
                CanonicalLevel::L2,
                CanonicalPlane::AsIs,
            ))),
            crate::envelopes::RawEventEnvelope::Task {
                task_id,
                title,
                state,
            } => records.push(AnyRecord::Task(TaskRecord::new(
                RecordId::from(task_id.as_str()),
                title.clone(),
                state.clone(),
            ))),
            crate::envelopes::RawEventEnvelope::Artifact {
                artifact_id,
                name,
                status,
                as_is_hash,
            } => records.push(AnyRecord::Artifact(ArtifactRecord::new(
                RecordId::from(artifact_id.as_str()),
                name.clone(),
                status.clone(),
                Some(AsIsHash::from(as_is_hash.as_str())),
            ))),
            crate::envelopes::RawEventEnvelope::RuntimeStatus { state } => records.push(
                AnyRecord::RuntimeStatus(RuntimeStatusRecord::new(state.clone())),
            ),
            crate::envelopes::RawEventEnvelope::Clarification {
                clarification_id,
                prompt,
            } => records.push(AnyRecord::Clarification(ClarificationRecord::new(
                RecordId::from(clarification_id.as_str()),
                prompt.clone(),
            ))),
        }
    }

    Ok(NormalizedBatch { records })
}
