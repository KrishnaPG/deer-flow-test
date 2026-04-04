use deer_foundation_contracts::{
    AppendControlRequest, AppendDataRequest, CanonicalLevel, CanonicalPlane, ControlIntentKind,
    IdempotencyKey, MissionId, StorageCorrelationIds, StorageLayout, StoragePayloadDescriptor,
    StoragePayloadFormat, StoragePayloadKind, StorageQosPolicy, StorageQosTemplate,
    StorageRequestMetadata,
};

use crate::deerflow_driver::mapping::mission_partition_tag;
use crate::deerflow_driver::transcript_emitter::transcript_payload;

#[derive(Default)]
pub struct DeerFlowDriver;

impl DeerFlowDriver {
    pub fn map_transcript_line(&self, mission_id: &str, line: &str) -> AppendDataRequest {
        let correlation = StorageCorrelationIds {
            mission_id: Some(MissionId::new(mission_id)),
            ..Default::default()
        };

        AppendDataRequest {
            layout: StorageLayout {
                hierarchy: deer_foundation_contracts::StorageHierarchyTag::new("A"),
                level: CanonicalLevel::L0,
                plane: CanonicalPlane::AsIs,
                payload_kind: StoragePayloadKind::new("chat-note"),
                format: StoragePayloadFormat::new("jsonl"),
                partition_tags: vec![mission_partition_tag(mission_id)],
            },
            metadata: StorageRequestMetadata {
                idempotency_key: Some(IdempotencyKey::new(format!(
                    "deerflow-transcript-{mission_id}"
                ))),
                correlation,
                ..Default::default()
            },
            qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
            payload: StoragePayloadDescriptor::InlineBytes {
                bytes: transcript_payload(line),
            },
        }
    }

    pub fn map_exclusion(&self, mission_id: &str, target_ref: &str) -> AppendControlRequest {
        let correlation = StorageCorrelationIds {
            mission_id: Some(MissionId::new(mission_id)),
            ..Default::default()
        };

        AppendControlRequest {
            control_kind: ControlIntentKind::Exclusion,
            target_refs: vec![target_ref.into()],
            metadata: StorageRequestMetadata {
                idempotency_key: Some(IdempotencyKey::new(format!(
                    "deerflow-control-{mission_id}"
                ))),
                correlation,
                ..Default::default()
            },
            qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
            rationale: Some("user_request".into()),
        }
    }
}
