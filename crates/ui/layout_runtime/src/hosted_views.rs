#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostedViewRegistration {
    view_id: &'static str,
}

impl HostedViewRegistration {
    pub const fn new(view_id: &'static str) -> Self {
        Self { view_id }
    }

    pub fn view_id(self) -> &'static str {
        self.view_id
    }
}

pub const CHAT_THREAD: HostedViewRegistration = HostedViewRegistration::new("chat_thread_view");
pub const ARTIFACT_SHELF: HostedViewRegistration =
    HostedViewRegistration::new("artifact_shelf_view");
pub const INSPECTOR: HostedViewRegistration = HostedViewRegistration::new("inspector_view");
