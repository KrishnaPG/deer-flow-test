use thiserror::Error;

#[derive(Debug, Error)]
pub enum RawSourceError {
    #[error("invalid fixture payload")]
    InvalidFixture,
    #[error("artifact access must remain mediated")]
    UnmediatedArtifactAccess,
}
