use serde::{Deserialize, Serialize};

pub const CURRENT_LAYOUT_SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DockNode {
    Tabs {
        panels: Vec<String>,
    },
    Split {
        axis: SplitAxis,
        ratio_bps: u16,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

impl DockNode {
    pub fn tabs(panels: Vec<String>) -> Self {
        Self::Tabs { panels }
    }

    pub fn split(axis: SplitAxis, ratio_bps: u16, first: DockNode, second: DockNode) -> Self {
        Self::Split {
            axis,
            ratio_bps,
            first: Box::new(first),
            second: Box::new(second),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutModal {
    pub panel_id: String,
}

impl LayoutModal {
    pub fn new(panel_id: &str) -> Self {
        Self {
            panel_id: panel_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    pub version: u32,
    pub mode: String,
    pub dock: DockNode,
    pub modals: Vec<LayoutModal>,
}

impl LayoutSnapshot {
    pub fn new(mode: &str, dock: DockNode, modals: Vec<LayoutModal>) -> Self {
        Self {
            version: CURRENT_LAYOUT_SNAPSHOT_VERSION,
            mode: mode.into(),
            dock,
            modals,
        }
    }
}
