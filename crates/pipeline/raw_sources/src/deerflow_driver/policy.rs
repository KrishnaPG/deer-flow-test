use deer_foundation_contracts::{StorageQosPolicy, StorageQosTemplate};

/// Size threshold above which artifacts use external-ref instead of inline payload.
pub const EXTERNAL_REF_THRESHOLD: usize = 1024 * 1024;

pub fn select_payload_mode(size_bytes: usize, is_artifact: bool) -> &'static str {
    if is_artifact && size_bytes > EXTERNAL_REF_THRESHOLD {
        "external-ref"
    } else {
        "inline"
    }
}

pub fn select_qos_policy(size_bytes: usize, is_artifact: bool) -> StorageQosPolicy {
    if is_artifact && size_bytes > EXTERNAL_REF_THRESHOLD {
        StorageQosPolicy::from_template(StorageQosTemplate::DurableArtifact)
    } else {
        StorageQosPolicy::from_template(StorageQosTemplate::FireAndForget)
    }
}
