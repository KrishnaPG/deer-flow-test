use dashmap::DashMap;

use crate::acp_client::ids::{AcpSessionSequenceNumber, ChatSessionId};

#[derive(Debug, Default)]
pub struct AcpSequenceAllocator {
    next_by_session: DashMap<ChatSessionId, AcpSessionSequenceNumber>,
}

impl AcpSequenceAllocator {
    pub fn next_for_session(&self, chat_session_id: &ChatSessionId) -> AcpSessionSequenceNumber {
        let mut entry = self
            .next_by_session
            .entry(chat_session_id.clone())
            .or_insert(AcpSessionSequenceNumber::new(0));
        let current = *entry;
        *entry = current.next();
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_allocator_scopes_numbers_per_session() {
        let allocator = AcpSequenceAllocator::default();
        let first_session = ChatSessionId::from("session-a");
        let second_session = ChatSessionId::from("session-b");

        assert_eq!(allocator.next_for_session(&first_session).get(), 0);
        assert_eq!(allocator.next_for_session(&first_session).get(), 1);
        assert_eq!(allocator.next_for_session(&second_session).get(), 0);
    }
}
