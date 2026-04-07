use deer_foundation_contracts::{
    AppendControlRequest, AppendDataRequest, CanonicalLevel, CanonicalPlane, ControlIntentKind,
    DerivationTrigger, FileAccepted, FileSaved, FileSavedTarget, IdempotencyKey, LogicalWriteId,
    StorageCorrelationIds, StorageHierarchyTag, StorageLayout, StorageLineageRefs,
    StoragePayloadDescriptor, StoragePayloadFormat, StoragePayloadKind, StorageQosPolicy,
    StorageQosTemplate, StorageRequestMetadata, WriterId,
};

fn full_correlation() -> StorageCorrelationIds {
    StorageCorrelationIds {
        mission_id: Some("mission_7".into()),
        run_id: Some("run_7".into()),
        agent_id: Some("agent_7".into()),
        artifact_id: Some("artifact_7".into()),
        trace_id: Some("trace_7".into()),
        extra: Vec::new(),
    }
}

fn full_lineage() -> StorageLineageRefs {
    StorageLineageRefs {
        parent_refs: vec!["parent_1".into()],
        derived_from_refs: vec!["source_1".into()],
    }
}

fn test_content_hash() -> String {
    "7bWpKq9xR3mNvHf2Tc8Yd".to_string()
}

#[test]
fn file_accepted_represents_durable_intent_handoff() {
    let accepted = FileAccepted {
        logical_write_id: LogicalWriteId::new("write_123"),
        idempotency_key: IdempotencyKey::new("idem_123"),
        topic_class: "write-intent".into(),
        routing_key: "mission_7".into(),
    };

    assert_eq!(accepted.topic_class, "write-intent");
}

#[test]
fn file_saved_requires_full_downstream_handoff_metadata() {
    let saved = FileSaved {
        logical_write_id: LogicalWriteId::new("write_123"),
        target: FileSavedTarget::SingleFile {
            relative_path: "B/L4/as-is/thumbnail/64x64/hash.png".into(),
        },
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L4,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("thumbnail"),
        format: StoragePayloadFormat::new("png"),
        correlation_ids: full_correlation(),
        lineage_refs: full_lineage(),
        routing_tags: vec![("size".into(), "64x64".into())],
        content_hash: test_content_hash(),
    };

    match saved.target {
        FileSavedTarget::SingleFile { relative_path } => {
            assert_eq!(relative_path, "B/L4/as-is/thumbnail/64x64/hash.png");
        }
        _ => panic!("expected single-file target"),
    }

    assert_eq!(
        saved.correlation_ids.mission_id.as_ref().unwrap().as_str(),
        "mission_7"
    );

    assert_eq!(saved.content_hash, "7bWpKq9xR3mNvHf2Tc8Yd");
}

#[test]
fn file_saved_carries_content_identity_and_location() {
    let saved = FileSaved {
        logical_write_id: LogicalWriteId::new("write_789"),
        target: FileSavedTarget::SingleFile {
            relative_path: "view/path/file.mp3".into(),
        },
        hierarchy: StorageHierarchyTag::new("A"),
        level: CanonicalLevel::L0,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("mp3"),
        format: StoragePayloadFormat::new("mp3"),
        correlation_ids: full_correlation(),
        lineage_refs: full_lineage(),
        routing_tags: vec![],
        content_hash: test_content_hash(),
    };

    assert_eq!(saved.content_hash, "7bWpKq9xR3mNvHf2Tc8Yd");
    assert_eq!(
        saved.target,
        FileSavedTarget::SingleFile {
            relative_path: "view/path/file.mp3".into(),
        }
    );
}

#[test]
fn append_data_requires_full_metadata_and_qos_policy() {
    let request = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new("A"),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![("mission".into(), "mission_7".into())],
        },
        metadata: StorageRequestMetadata::default(),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes {
            bytes: b"hello".to_vec(),
        },
    };

    assert_eq!(request.validate(), Err("missing idempotency key"));
}

#[test]
fn append_data_validation_requires_named_correlation_and_writer_identity() {
    let request = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new("A"),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![],
        },
        metadata: StorageRequestMetadata {
            idempotency_key: Some(IdempotencyKey::new("idem_1")),
            writer_identity: Some(WriterId::new("writer_1")),
            correlation: StorageCorrelationIds {
                mission_id: Some("mission_1".into()),
                run_id: Some("run_1".into()),
                agent_id: Some("agent_1".into()),
                artifact_id: Some("artifact_1".into()),
                trace_id: Some("trace_1".into()),
                extra: Vec::new(),
            },
            lineage: StorageLineageRefs::default(),
            ..Default::default()
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes {
            bytes: b"ok".to_vec(),
        },
    };

    assert!(request.validate().is_ok());
}

#[test]
fn append_control_is_separate_from_append_data() {
    let request = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: vec!["as_is_hash_xyz".into()],
        metadata: StorageRequestMetadata::default(),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::ConflictIntent),
        rationale: Some("user_request".into()),
    };

    assert_eq!(request.target_refs.len(), 1);
    assert_eq!(request.qos.template, StorageQosTemplate::ConflictIntent);
}

#[test]
fn append_control_validation_requires_identity_and_targets() {
    let request = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: Vec::new(),
        metadata: StorageRequestMetadata::default(),
        qos: StorageQosPolicy::from_template(StorageQosTemplate::ConflictIntent),
        rationale: None,
    };

    assert_eq!(request.validate(), Err("missing idempotency key"));
}

#[test]
fn derivation_trigger_reuses_full_saved_context() {
    let trigger = DerivationTrigger {
        logical_write_id: LogicalWriteId::new("write_123"),
        relative_target: "B/L3/chunks/transcript/date/hash.parquet".into(),
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L3,
        plane: CanonicalPlane::Chunks,
        payload_kind: StoragePayloadKind::new("transcript"),
        format: StoragePayloadFormat::new("parquet"),
        correlation_ids: full_correlation(),
        lineage_refs: full_lineage(),
        routing_tags: vec![("date".into(), "2026-04-04".into())],
        content_hash: test_content_hash(),
    };

    assert_eq!(
        trigger.relative_target,
        "B/L3/chunks/transcript/date/hash.parquet"
    );
    assert_eq!(trigger.content_hash, "7bWpKq9xR3mNvHf2Tc8Yd");
}

#[test]
fn file_saved_supports_manifest_targets_for_multi_file_commits() {
    let saved = FileSaved {
        logical_write_id: LogicalWriteId::new("write_456"),
        target: FileSavedTarget::CommitManifest {
            manifest_path: "B/L3/chunks/transcript/date/hash.manifest.json".into(),
            member_count: 3,
        },
        hierarchy: StorageHierarchyTag::new("B"),
        level: CanonicalLevel::L3,
        plane: CanonicalPlane::Chunks,
        payload_kind: StoragePayloadKind::new("transcript"),
        format: StoragePayloadFormat::new("json"),
        correlation_ids: full_correlation(),
        lineage_refs: full_lineage(),
        routing_tags: vec![("date".into(), "2026-04-04".into())],
        content_hash: test_content_hash(),
    };

    match saved.target {
        FileSavedTarget::CommitManifest {
            manifest_path,
            member_count,
        } => {
            assert_eq!(
                manifest_path,
                "B/L3/chunks/transcript/date/hash.manifest.json"
            );
            assert_eq!(member_count, 3);
        }
        _ => panic!("expected commit-manifest target"),
    }
}

#[test]
fn qos_policy_supports_template_plus_overrides() {
    let mut qos = StorageQosPolicy::from_template(StorageQosTemplate::DurableArtifact);
    qos.worker_priority = "burst-high".into();

    assert_eq!(qos.template, StorageQosTemplate::DurableArtifact);
    assert_eq!(qos.durability_class, "strong");
    assert_eq!(qos.worker_priority, "burst-high");
}

#[test]
fn append_control_validation_rejects_missing_targets_after_metadata_is_valid() {
    let request = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: Vec::new(),
        metadata: StorageRequestMetadata {
            idempotency_key: Some(IdempotencyKey::new("idem_ok")),
            writer_identity: Some(WriterId::new("writer_1")),
            correlation: StorageCorrelationIds {
                mission_id: Some("mission_1".into()),
                run_id: Some("run_1".into()),
                agent_id: Some("agent_1".into()),
                artifact_id: Some("artifact_1".into()),
                trace_id: Some("trace_1".into()),
                extra: Vec::new(),
            },
            lineage: StorageLineageRefs::default(),
            ..Default::default()
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::ConflictIntent),
        rationale: None,
    };

    assert_eq!(request.validate(), Err("missing control targets"));
}

#[test]
fn append_data_validation_rejects_empty_layout_fields() {
    let request = AppendDataRequest {
        layout: StorageLayout {
            hierarchy: StorageHierarchyTag::new(""),
            level: CanonicalLevel::L0,
            plane: CanonicalPlane::AsIs,
            payload_kind: StoragePayloadKind::new("chat-note"),
            format: StoragePayloadFormat::new("jsonl"),
            partition_tags: vec![],
        },
        metadata: StorageRequestMetadata {
            idempotency_key: Some(IdempotencyKey::new("idem_1")),
            writer_identity: Some(WriterId::new("writer_1")),
            correlation: StorageCorrelationIds {
                mission_id: Some("mission_1".into()),
                run_id: Some("run_1".into()),
                agent_id: Some("agent_1".into()),
                artifact_id: Some("artifact_1".into()),
                trace_id: Some("trace_1".into()),
                extra: Vec::new(),
            },
            lineage: StorageLineageRefs::default(),
            ..Default::default()
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes {
            bytes: b"ok".to_vec(),
        },
    };

    assert_eq!(request.validate(), Err("missing layout field"));
}
