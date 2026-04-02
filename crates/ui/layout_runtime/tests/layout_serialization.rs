use deer_ui_layout_runtime::{
    deserialize_layout, serialize_layout, DockNode, LayoutModal, LayoutSnapshot, SplitAxis,
    CURRENT_LAYOUT_SNAPSHOT_VERSION,
};
use insta::assert_yaml_snapshot;

#[test]
fn layout_snapshot_round_trips_versioned_dock_and_modal_structure() {
    let snapshot = LayoutSnapshot::new(
        "live_meeting",
        DockNode::split(
            SplitAxis::Horizontal,
            6000,
            DockNode::tabs(vec!["chat_panel".into(), "artifact_panel".into()]),
            DockNode::tabs(vec!["inspector_panel".into()]),
        ),
        vec![LayoutModal::new("detail_modal")],
    );
    let encoded = serialize_layout(&snapshot).unwrap();
    let restored = deserialize_layout(&encoded).unwrap();

    assert_eq!(restored, snapshot);
    assert_eq!(restored.version, CURRENT_LAYOUT_SNAPSHOT_VERSION);

    assert_yaml_snapshot!(restored, @r#"
version: 1
mode: live_meeting
dock:
  kind: split
  axis: horizontal
  ratio_bps: 6000
  first:
    kind: tabs
    panels:
      - chat_panel
      - artifact_panel
  second:
    kind: tabs
    panels:
      - inspector_panel
modals:
  - panel_id: detail_modal
"#);
}
