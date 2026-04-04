use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, StorageHierarchyTag, StoragePayloadFormat, StoragePayloadKind,
};

use crate::path_builder::build_relative_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRefCommitRequest {
    pub source_uri: String,
    pub hierarchy: StorageHierarchyTag,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub payload_kind: StoragePayloadKind,
    pub format: StoragePayloadFormat,
    pub partitions: Vec<(String, String)>,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRefCommitResult {
    pub promoted: bool,
    pub saved_relative_target: String,
}

pub fn commit_external_ref(
    request: ExternalRefCommitRequest,
) -> Result<ExternalRefCommitResult, &'static str> {
    if request.source_uri.is_empty() {
        return Err("missing source uri");
    }

    let target = build_relative_path(
        &request.hierarchy,
        request.level,
        request.plane,
        &request.payload_kind,
        &request.format,
        &request.partitions,
        &request.content_hash,
    );

    Ok(ExternalRefCommitResult {
        promoted: true,
        saved_relative_target: target,
    })
}
