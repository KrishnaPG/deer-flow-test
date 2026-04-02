use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreadHeaderVm {
    pub thread_id: String,
    pub title: String,
    pub stream_state: String,
}

pub fn derive_thread_header_vm(thread_id: &str, title: &str, stream_state: &str) -> ThreadHeaderVm {
    ThreadHeaderVm {
        thread_id: thread_id.into(),
        title: title.into(),
        stream_state: stream_state.into(),
    }
}
