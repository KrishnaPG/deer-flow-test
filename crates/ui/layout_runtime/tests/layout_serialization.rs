use deer_ui_layout_runtime::{deserialize_layout, serialize_layout, LayoutSnapshot};
use insta::assert_yaml_snapshot;

#[test]
fn layout_snapshot_round_trips_with_panel_ids_and_mode() {
    let snapshot = LayoutSnapshot::new(
        "live_meeting",
        vec!["chat_panel".into(), "inspector_panel".into()],
    );
    let encoded = serialize_layout(&snapshot).unwrap();
    let restored = deserialize_layout(&encoded).unwrap();

    assert_eq!(restored, snapshot);

    assert_yaml_snapshot!(restored, @r#"
mode: live_meeting
panels:
  - chat_panel
  - inspector_panel
"#);
}
