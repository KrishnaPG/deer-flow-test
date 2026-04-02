use deer_foundation_contracts::{IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{
    AnyRecord, IntentBody, IntentRecord, WriteOperationBody, WriteOperationRecord,
};

use crate::error::NormalizationError;

pub fn emit_intent_records(stage: &str) -> Result<Vec<AnyRecord>, NormalizationError> {
    Ok(match stage {
        "submitted" => vec![
            AnyRecord::Intent(IntentRecord::new(
                RecordId::from_static("intent_submitted"),
                IdentityMeta::hash_anchored(
                    RecordId::from_static("intent_submitted"),
                    None,
                    None,
                    None,
                ),
                LineageMeta::root(),
                IntentBody {
                    action: "submitted".into(),
                },
            )),
            AnyRecord::WriteOperation(WriteOperationRecord::new(
                RecordId::from_static("write_op_queued"),
                IdentityMeta::hash_anchored(
                    RecordId::from_static("write_op_queued"),
                    None,
                    None,
                    None,
                ),
                LineageMeta::root(),
                WriteOperationBody {
                    op: "queued".into(),
                },
            )),
        ],
        "prefill_seed" | "prefill" | "draft" | "validated" => Vec::new(),
        other => return Err(NormalizationError::UnsupportedEventKind(other.to_string())),
    })
}
