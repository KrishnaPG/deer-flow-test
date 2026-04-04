//! Toy example: full storage-service flow from DeerFlow request to FileSaved handoff.
//!
//! Run with: `cargo run --example storage_demo`

use deer_foundation_contracts::{
    AppendControlRequest, CanonicalLevel, CanonicalPlane, ControlIntentKind, FileSavedTarget,
    IdempotencyKey, LogicalWriteId, MissionId, StorageCorrelationIds, StorageHierarchyTag,
    StorageLayout, StorageLineageRefs, StoragePayloadDescriptor, StoragePayloadFormat,
    StoragePayloadKind, StorageQosPolicy, StorageQosTemplate, StorageRequestMetadata,
};
use deer_storage_core::{
    admission::{AdmissionBudget, AdmissionController},
    downstream_handoff::{derive_trigger_from_manifest, make_file_saved},
    path_builder::build_relative_path,
    topics::{route_topic, TopicClass},
    validation::validate_append_pair,
};

fn main() {
    println!("=== Storage Service Demo ===\n");

    // ── Step 1: Build a storage request (as DeerFlow driver would) ──
    let layout = StorageLayout {
        hierarchy: StorageHierarchyTag::new("A"),
        level: CanonicalLevel::L0,
        plane: CanonicalPlane::AsIs,
        payload_kind: StoragePayloadKind::new("chat-note"),
        format: StoragePayloadFormat::new("jsonl"),
        partition_tags: vec![("mission".into(), "mission_1".into())],
    };

    let data_req = AppendDataRequest {
        layout: layout.clone(),
        metadata: StorageRequestMetadata {
            idempotency_key: Some(IdempotencyKey::new("deerflow-transcript-mission_1")),
            correlation: StorageCorrelationIds {
                mission_id: Some(MissionId::new("mission_1")),
                ..Default::default()
            },
            lineage: StorageLineageRefs::default(),
            writer_identity: "deerflow-driver".into(),
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        payload: StoragePayloadDescriptor::InlineBytes {
            bytes: b"Confirm the survey radius.".to_vec(),
        },
    };

    let control_req = AppendControlRequest {
        control_kind: ControlIntentKind::Exclusion,
        target_refs: vec!["hash_xyz".into()],
        metadata: StorageRequestMetadata {
            idempotency_key: Some(IdempotencyKey::new("deerflow-control-mission_1")),
            correlation: StorageCorrelationIds {
                mission_id: Some(MissionId::new("mission_1")),
                ..Default::default()
            },
            lineage: StorageLineageRefs::default(),
            writer_identity: "deerflow-driver".into(),
        },
        qos: StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget),
        rationale: Some("user_request".into()),
    };

    println!("1. Built storage requests:");
    println!("   hierarchy:     {}", data_req.layout.hierarchy.as_str());
    println!("   level:         {:?}", data_req.layout.level);
    println!(
        "   payload_kind:  {}",
        data_req.layout.payload_kind.as_str()
    );
    println!("   format:        {}", data_req.layout.format.extension());
    println!();

    // ── Step 2: Validate the pair ──
    validate_append_pair(&data_req, &control_req).expect("validation failed");
    println!("2. Validation passed ✓");

    // ── Step 3: Check admission thresholds ──
    let controller = AdmissionController::new(AdmissionBudget::new(100, 1024 * 1024 * 1024));
    controller
        .try_accept(42, 1024 * 1024)
        .expect("admission rejected");
    println!("3. Admission accepted (42/100 active writes) ✓");

    // ── Step 4: Route to the correct Redpanda topic ──
    let route = route_topic(TopicClass::WriteIntent, "mission_1");
    println!(
        "4. Topic route: {} (key: {})",
        route.topic_name, route.routing_key
    );

    // ── Step 5: Build the canonical storage path ──
    let canonical = build_relative_path(
        &data_req.layout.hierarchy,
        data_req.layout.level,
        data_req.layout.plane,
        &data_req.layout.payload_kind,
        &data_req.layout.format,
        &data_req.layout.partition_tags,
        "sha256_abc123",
    );
    println!("5. Canonical path: {}", canonical);

    // ── Step 6: Simulate durable publish → FileAccepted ──
    let logical_write_id = LogicalWriteId::new("write_1");
    let idempotency_key = IdempotencyKey::new("deerflow-transcript-mission_1");
    println!(
        "6. FileAccepted: write={} idempotency={} topic={}",
        logical_write_id.as_str(),
        idempotency_key.as_str(),
        route.topic_name,
    );

    // ── Step 7: Emit FileSaved for downstream consumers ──
    let saved = make_file_saved(
        logical_write_id.clone(),
        &canonical,
        StorageHierarchyTag::new("A"),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        StoragePayloadKind::new("chat-note"),
        StoragePayloadFormat::new("jsonl"),
        vec![("mission_id".into(), "mission_1".into())],
        vec![],
        vec![("mission".into(), "mission_1".into())],
    );

    match &saved.target {
        FileSavedTarget::SingleFile { relative_path } => {
            println!("7. FileSaved: target={}", relative_path);
        }
    }
    println!(
        "   mission_id: {:?}",
        saved
            .correlation_ids
            .mission_id
            .as_ref()
            .map(|m| m.as_str())
    );

    // ── Step 8: Derive a trigger from a manifest ──
    let trigger = derive_trigger_from_manifest(
        "write_7",
        "B/L3/chunks/transcript/day/manifest.json",
        vec![("mission_id".into(), "mission_1".into())],
        vec!["write_1".into()],
        vec![("date".into(), "2026-04-04".into())],
    );
    println!(
        "8. DerivationTrigger: target={} from={} parent(s)",
        trigger.relative_target,
        trigger.lineage_refs.parent_refs.len(),
    );

    println!("\n=== Demo complete ===");
}
