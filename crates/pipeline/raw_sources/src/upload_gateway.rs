use crate::error::RawSourceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceipt {
    pub thread_id: String,
    pub file_name: String,
}

pub fn stage_upload(thread_id: &str, file_name: &str) -> Result<UploadReceipt, RawSourceError> {
    Ok(UploadReceipt {
        thread_id: thread_id.into(),
        file_name: file_name.into(),
    })
}
