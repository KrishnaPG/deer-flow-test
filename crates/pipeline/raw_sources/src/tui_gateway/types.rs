use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

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
    TuiGatewaySessionId,
    "Stable identifier for one TUI gateway session."
);
string_newtype!(
    TuiGatewayRequestId,
    "JSON-RPC request identifier for one TUI gateway command."
);
string_newtype!(
    TuiGatewayRunId,
    "Stable identifier for one prompt submitted through a TUI gateway."
);
string_newtype!(
    TuiGatewayProcessId,
    "Stable identifier for one TUI gateway subprocess."
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TerminalColumnCount(u16);

impl TerminalColumnCount {
    pub fn new(value: u16) -> Self {
        Self(value.max(1))
    }

    pub fn get(self) -> u16 {
        self.0
    }
}

impl Default for TerminalColumnCount {
    fn default() -> Self {
        Self::new(80)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiGatewayCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub working_directory: PathBuf,
    pub env: Vec<(OsString, OsString)>,
}

impl TuiGatewayCommand {
    pub fn new(program: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            program,
            args: Vec::new(),
            working_directory,
            env: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<OsString>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, env: Vec<(OsString, OsString)>) -> Self {
        self.env = env;
        self
    }
}

impl TuiGatewayProcessId {
    pub fn generate() -> Self {
        Self::new(Uuid::now_v7().to_string())
    }
}

impl TuiGatewayRequestId {
    pub fn generate() -> Self {
        Self::new(format!("r{}", Uuid::now_v7()))
    }
}

impl TuiGatewayRunId {
    pub fn generate() -> Self {
        Self::new(Uuid::now_v7().to_string())
    }
}
