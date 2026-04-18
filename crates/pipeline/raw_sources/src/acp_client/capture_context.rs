use std::sync::Arc;

use dashmap::DashMap;

use crate::acp_client::{AcpSubprocessId, ChatRunId, ChatSessionId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpCaptureContext {
    pub chat_session_id: ChatSessionId,
    pub chat_run_id: ChatRunId,
    pub acp_subprocess_id: AcpSubprocessId,
}

#[derive(Default)]
pub struct AcpCaptureContextStore {
    entries: DashMap<AcpSubprocessId, AcpCaptureContext>,
}

impl AcpCaptureContextStore {
    pub fn initialize_for_subprocess(
        &self,
        acp_subprocess_id: AcpSubprocessId,
    ) -> AcpCaptureContext {
        let context = AcpCaptureContext {
            chat_session_id: ChatSessionId::pending_for_subprocess(&acp_subprocess_id),
            chat_run_id: ChatRunId::new(format!("bootstrap:{}", acp_subprocess_id.as_str())),
            acp_subprocess_id: acp_subprocess_id.clone(),
        };
        self.entries.insert(acp_subprocess_id, context.clone());
        context
    }

    pub fn assign_session(
        &self,
        acp_subprocess_id: &AcpSubprocessId,
        chat_session_id: ChatSessionId,
    ) -> Option<AcpCaptureContext> {
        let mut entry = self.entries.get_mut(acp_subprocess_id)?;
        entry.chat_session_id = chat_session_id;
        Some(entry.clone())
    }

    pub fn assign_run(
        &self,
        acp_subprocess_id: &AcpSubprocessId,
        chat_run_id: ChatRunId,
    ) -> Option<AcpCaptureContext> {
        let mut entry = self.entries.get_mut(acp_subprocess_id)?;
        entry.chat_run_id = chat_run_id;
        Some(entry.clone())
    }

    pub fn get(&self, acp_subprocess_id: &AcpSubprocessId) -> Option<AcpCaptureContext> {
        self.entries
            .get(acp_subprocess_id)
            .map(|entry| entry.clone())
    }

    pub fn first(&self) -> Option<AcpCaptureContext> {
        self.entries
            .iter()
            .next()
            .map(|entry| entry.value().clone())
    }
}

pub type SharedAcpCaptureContextStore = Arc<AcpCaptureContextStore>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_context_is_initialized_for_subprocess() {
        let store = AcpCaptureContextStore::default();
        let process_id = AcpSubprocessId::from("proc-1");

        let context = store.initialize_for_subprocess(process_id.clone());

        assert_eq!(context.acp_subprocess_id, process_id);
        assert!(context.chat_session_id.as_str().starts_with("pending:"));
    }
}
