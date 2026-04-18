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

impl ChatSessionId {
    pub fn pending_for_subprocess(acp_subprocess_id: &AcpSubprocessId) -> Self {
        Self::new(format!("pending:{}", acp_subprocess_id.as_str()))
    }

    pub fn from_first_event(first_event_bytes: &[u8], pid: u32, timestamp_ns: i64) -> Self {
        let hash = blake3::hash(first_event_bytes);
        Self::new(format!("{}_{}_{}", hash.to_hex(), timestamp_ns, pid))
    }
}

impl AcpSubprocessId {
    pub fn generate() -> Self {
        Self::new(Uuid::now_v7().to_string())
    }
}
