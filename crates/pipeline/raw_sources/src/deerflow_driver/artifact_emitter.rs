use crate::deerflow_driver::policy::EXTERNAL_REF_THRESHOLD;

pub fn artifact_is_external_ref(size_bytes: usize) -> bool {
    size_bytes > EXTERNAL_REF_THRESHOLD
}
