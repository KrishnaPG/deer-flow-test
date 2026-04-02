use serde::Deserialize;

use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThreadSnapshot {
    pub thread_id: String,
    pub title: String,
}

pub fn create_thread(title: &str) -> Result<ThreadSnapshot, RawSourceError> {
    Ok(ThreadSnapshot {
        thread_id: "thread_created_1".into(),
        title: title.into(),
    })
}

pub fn resume_thread(fixture_json: &str) -> Result<ThreadSnapshot, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}
