use deer_pipeline_derivations::ComposerVm;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ComposerViewState {
    pub primary_command: &'static str,
    pub send_state: &'static str,
    pub is_busy: bool,
    pub attachment_count: usize,
}

pub fn render_composer_view(vm: &ComposerVm) -> ComposerViewState {
    ComposerViewState {
        primary_command: if vm.mode == "clarification_response" {
            "resume_clarification"
        } else {
            "send_message"
        },
        send_state: vm.send_state,
        is_busy: vm.send_state != "idle",
        attachment_count: vm.attachments.len(),
    }
}
