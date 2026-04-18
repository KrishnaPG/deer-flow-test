use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! string_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_newtype!(
    ChatSessionId,
    "Stable identifier for one conversational chat session."
);
string_newtype!(
    ChatRunId,
    "Stable identifier for one prompt execution within a chat session."
);
string_newtype!(
    ChatThreadId,
    "Optional identifier for a branched chat thread lineage."
);
string_newtype!(
    AcpSubprocessId,
    "Stable identifier for one ACP subprocess instance."
);
string_newtype!(
    AcpJsonRpcRequestId,
    "Protocol-level JSON-RPC request identifier observed on the ACP boundary."
);

impl ChatRunId {
    pub fn generate() -> Self {
        Self::new(Uuid::now_v7().to_string())
    }

    pub fn bootstrap_for_session(chat_session_id: &ChatSessionId) -> Self {
        Self::new(format!("bootstrap:{}", chat_session_id.as_str()))
    }
}

impl AcpSubprocessId {
    pub fn generate() -> Self {
        Self::new(Uuid::now_v7().to_string())
    }
}

/// Monotonic sequence number scoped to a single `ChatSessionId`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct AcpSessionSequenceNumber(u64);

impl AcpSessionSequenceNumber {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for AcpSessionSequenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_numbers_increment_monotonically() {
        let first = AcpSessionSequenceNumber::new(41);
        let second = first.next();

        assert_eq!(second.get(), 42);
    }
}
