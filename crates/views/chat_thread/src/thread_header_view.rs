use deer_pipeline_derivations::ThreadHeaderVm;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ThreadHeaderViewState {
    pub thread_id: String,
    pub title: String,
    pub stream_state: String,
    pub primary_command: &'static str,
    pub secondary_command: &'static str,
}

pub fn render_thread_header_view(vm: &ThreadHeaderVm) -> ThreadHeaderViewState {
    ThreadHeaderViewState {
        thread_id: vm.thread_id.clone(),
        title: vm.title.clone(),
        stream_state: vm.stream_state.clone(),
        primary_command: "resume_thread",
        secondary_command: "refresh_stream",
    }
}
