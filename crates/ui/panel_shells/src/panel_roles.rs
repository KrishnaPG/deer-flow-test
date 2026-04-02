use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelRole {
    Source,
    Sink,
    Broker,
    Mirror,
}
