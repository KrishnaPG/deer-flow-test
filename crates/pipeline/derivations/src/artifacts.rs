use deer_foundation_domain::AnyRecord;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactEntryVm {
    pub artifact_id: String,
    pub title: String,
    pub status: String,
    pub preview_supported: bool,
    pub retrieval_mode: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactShelfVm {
    pub entries: Vec<ArtifactEntryVm>,
}

pub fn derive_artifact_shelf_vm(records: &[AnyRecord]) -> ArtifactShelfVm {
    let entries = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Artifact(artifact) => Some(ArtifactEntryVm {
                artifact_id: artifact.record_id().to_string(),
                title: artifact.body.label.clone(),
                status: "presented".into(),
                preview_supported: true,
                retrieval_mode: "mediated_pointer",
            }),
            _ => None,
        })
        .collect();

    ArtifactShelfVm { entries }
}
