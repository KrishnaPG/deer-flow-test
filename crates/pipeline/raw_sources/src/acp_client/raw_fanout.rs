use bytes::Bytes;
use tokio::sync::broadcast;

use crate::acp_client::ids::ChatSessionId;

#[derive(Clone)]
pub struct RawEventFanout {
    sender: broadcast::Sender<(ChatSessionId, u64, Bytes)>,
}

impl Default for RawEventFanout {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }
}

impl RawEventFanout {
    pub fn publish(&self, session_id: ChatSessionId, sequence: u64, raw_bytes: Bytes) {
        let _ = self.sender.send((session_id, sequence, raw_bytes));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(ChatSessionId, u64, Bytes)> {
        self.sender.subscribe()
    }
}
