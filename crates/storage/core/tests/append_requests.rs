use deer_foundation_contracts::{
    AgentId, AppendControlRequest, AppendDataRequest, ArtifactId, CanonicalLevel, CanonicalPlane,
    ControlIntentKind, IdempotencyKey, MissionId, RunId, StorageCorrelationIds,
    StorageHierarchyTag, StorageLayout, StorageLineageRefs, StoragePayloadDescriptor,
    StoragePayloadFormat, StoragePayloadKind, StorageQosPolicy, StorageQosTemplate,
    StorageRequestMetadata, TraceId, WriterId,
};
use deer_storage_core::validation::validate_append_pair;

fn full_metadata(idempotency_key: &str) -> StorageRequestMetadata {
    StorageRequestMetadata {
        idempotency_key: Some(IdempotencyKey::new(idempotency_key)),
        writer_identity: Some(WriterId::new("writer_1")),
        correlation: StorageCorrelationIds {
            mission_id: Some(MissionId::new("m1")),
            run_id: Some(RunId::new("run_1")),
            agent_id: Some(AgentId::new("agent_1")),
            artifact_id: Some(ArtifactId::new("artifact_1")),
            trace_id: Some(TraceId::new("trace_1")),
            extra: Vec::new(),
        },
        lineage: StorageLineageRefs::default(),
        ..Default::default()
    }
}

#[test]
fn validates_data_and_control_requests_separately() {
    let data = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new("A"),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![],
        },
        metadata: full_metadata("idem_1"),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes {
            bytes: b"hello".to_vec(),
        },
    };
    let control = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: vec!["as_is_hash_xyz".into()],
        metadata: full_metadata("idem_2"),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::ConflictIntent),
        rationale: Some("user_request".into()),
    };

    assert!(validate_append_pair(&data, &control).is_ok());
}
