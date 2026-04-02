use thiserror::Error;

#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("unsupported event kind: {0}")]
    UnsupportedEventKind(String),
    #[error("invalid level promotion: {from} -> {to}")]
    InvalidPromotion { from: String, to: String },
    #[error("missing required hash anchor for {family}")]
    MissingHashAnchor { family: &'static str },
}
