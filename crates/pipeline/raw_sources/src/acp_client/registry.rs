use dashmap::DashMap;

use crate::acp_client::ids::{AcpSubprocessId, ChatRunId, ChatSessionId, ChatThreadId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpChatSessionState {
    pub chat_session_id: ChatSessionId,
    pub acp_subprocess_id: AcpSubprocessId,
    pub chat_thread_id: Option<ChatThreadId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpChatRunState {
    pub chat_run_id: ChatRunId,
    pub chat_session_id: ChatSessionId,
}

#[derive(Default)]
pub struct AcpChatSessionRegistry {
    entries: DashMap<ChatSessionId, AcpChatSessionState>,
}

impl AcpChatSessionRegistry {
    pub fn insert(&self, state: AcpChatSessionState) {
        self.entries.insert(state.chat_session_id.clone(), state);
    }

    pub fn get(&self, chat_session_id: &ChatSessionId) -> Option<AcpChatSessionState> {
        self.entries.get(chat_session_id).map(|entry| entry.clone())
    }
}

#[derive(Default)]
pub struct AcpChatRunRegistry {
    entries: DashMap<ChatRunId, AcpChatRunState>,
}

impl AcpChatRunRegistry {
    pub fn insert(&self, state: AcpChatRunState) {
        self.entries.insert(state.chat_run_id.clone(), state);
    }

    pub fn get(&self, chat_run_id: &ChatRunId) -> Option<AcpChatRunState> {
        self.entries.get(chat_run_id).map(|entry| entry.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_registry_round_trips_state() {
        let registry = AcpChatSessionRegistry::default();
        let state = AcpChatSessionState {
            chat_session_id: ChatSessionId::from("session-1"),
            acp_subprocess_id: AcpSubprocessId::from("proc-1"),
            chat_thread_id: None,
        };

        registry.insert(state.clone());

        assert_eq!(registry.get(&state.chat_session_id), Some(state));
    }
}
