use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn from_static(value: &'static str) -> Self {
                Self(value.to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_newtype!(RecordId);
string_newtype!(MissionId);
string_newtype!(RunId);
string_newtype!(AgentId);
string_newtype!(ArtifactId);
string_newtype!(TraceId);
string_newtype!(ThreadId);
string_newtype!(TaskId);
string_newtype!(EventId);
string_newtype!(AsIsHash);
string_newtype!(ChunkHash);
string_newtype!(EmbeddingBasisHash);
