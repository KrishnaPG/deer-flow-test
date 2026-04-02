use deer_ui_layout_runtime::{
    deserialize_layout, serialize_layout, DockNode, LayoutSnapshot, SplitAxis,
};

use crate::panel_catalog::{ARTIFACT_PANEL, CHAT_PANEL, INSPECTOR_PANEL};

pub const LIVE_MEETING_MODE: &str = "live_meeting";

pub fn live_meeting_layout() -> LayoutSnapshot {
    LayoutSnapshot::new(
        LIVE_MEETING_MODE,
        DockNode::split(
            SplitAxis::Horizontal,
            6000,
            DockNode::tabs(vec![CHAT_PANEL.into(), ARTIFACT_PANEL.into()]),
            DockNode::tabs(vec![INSPECTOR_PANEL.into()]),
        ),
        Vec::new(),
    )
}

pub fn restore_live_meeting_layout() -> LayoutSnapshot {
    let encoded = serialize_layout(&live_meeting_layout()).expect("layout should serialize");

    deserialize_layout(&encoded).expect("layout should deserialize")
}
