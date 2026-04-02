use deer_foundation_contracts::RecordFamily;
use deer_foundation_domain::AnyRecord;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactEntryVm {
    pub artifact_id: String,
    pub title: String,
    pub status: String,
    pub preview_supported: bool,
    pub retrieval_mode: &'static str,
    pub provenance: Option<String>,
    pub presentation_state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactShelfVm {
    pub entries: Vec<ArtifactEntryVm>,
}

pub fn derive_artifact_shelf_vm(records: &[AnyRecord]) -> ArtifactShelfVm {
    let entries = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Artifact(artifact) if is_presented(artifact) => Some(ArtifactEntryVm {
                artifact_id: artifact.record_id().to_string(),
                title: artifact.body.label.clone(),
                status: "presented".into(),
                preview_supported: is_preview_supported(&artifact.body.media_type),
                retrieval_mode: "mediated_pointer",
                provenance: artifact
                    .header
                    .identity
                    .as_is_hash
                    .as_ref()
                    .map(|hash| hash.as_str().to_string()),
                presentation_state: "presented",
            }),
            _ => None,
        })
        .collect();

    ArtifactShelfVm { entries }
}

fn is_presented(artifact: &deer_foundation_domain::ArtifactRecord) -> bool {
    artifact.header.lineage.derived_from.iter().any(|record| {
        matches!(
            record.family,
            RecordFamily::Message | RecordFamily::Clarification
        )
    })
}

fn is_preview_supported(media_type: &str) -> bool {
    media_type.starts_with("image/") || media_type.starts_with("text/")
}
