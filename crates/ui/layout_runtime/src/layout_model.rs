use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    pub mode: String,
    pub panels: Vec<String>,
}

impl LayoutSnapshot {
    pub fn new(mode: &str, panels: Vec<String>) -> Self {
        Self {
            mode: mode.into(),
            panels,
        }
    }
}
