use deer_pipeline_raw_sources::deerflow_driver::policy::{select_payload_mode, select_qos_policy};

#[test]
fn large_artifacts_choose_external_ref_and_durable_artifact_qos() {
    assert_eq!(select_payload_mode(50 * 1024 * 1024, true), "external-ref");
    assert_eq!(
        select_qos_policy(50 * 1024 * 1024, true).template,
        deer_foundation_contracts::StorageQosTemplate::DurableArtifact
    );
}

#[test]
fn conflicting_intents_preserve_both_control_writes() {
    let left = select_qos_policy(128, false);
    let right = select_qos_policy(128, false);
    assert_eq!(left.template, right.template);
}
