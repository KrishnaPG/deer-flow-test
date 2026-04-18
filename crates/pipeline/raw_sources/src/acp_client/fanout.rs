use tokio::sync::broadcast;

use crate::acp_client::live_event::AcpResponseStreamEvent;

/// Live event fanout for UI-facing ACP response stream projections.
#[derive(Clone, Debug)]
pub struct AcpResponseStreamFanout {
    sender: broadcast::Sender<AcpResponseStreamEvent>,
}

impl Default for AcpResponseStreamFanout {
    fn default() -> Self {
        Self::new(256)
    }
}

impl AcpResponseStreamFanout {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AcpResponseStreamEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: AcpResponseStreamEvent) {
        let _ = self.sender.send(event);
    }
}
