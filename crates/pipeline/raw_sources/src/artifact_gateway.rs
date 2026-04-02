use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactAccess {
    PreviewPayload { mime: String },
    AuthorizedPointer { href: String },
}

pub fn preview_artifact(
    thread_id: &str,
    artifact_id: &str,
) -> Result<ArtifactAccess, RawSourceError> {
    if thread_id.is_empty() || artifact_id.is_empty() {
        return Err(RawSourceError::UnmediatedArtifactAccess);
    }

    Ok(ArtifactAccess::PreviewPayload {
        mime: "image/png".into(),
    })
}
