use deer_view_telemetry_view::{
    link_cameras, sync_camera, CameraLinkMode, NavigationEvent, ViewportCameraState,
};

#[test]
fn viewport_camera_state_preserves_fractional_center_precision() {
    let world = ViewportCameraState::new("world", [10.25, 20.75], 3);

    assert_eq!(world.center, [10.25, 20.75]);
}

#[test]
fn camera_sync_reports_bidirectional_propagated_camera_state() {
    let world = ViewportCameraState::new("world", [10.25, 20.75], 3);
    let minimap = ViewportCameraState::new("minimap", [2.5, 4.5], 1);

    let link = link_cameras("world", "minimap", CameraLinkMode::Bidirectional);
    let synced = sync_camera(&link, &world, &minimap).expect("expected linked camera sync");

    assert_eq!(synced.mode, CameraLinkMode::Bidirectional);
    assert_eq!(synced.propagated_from.viewport_id, "world");
    assert_eq!(synced.propagated_from.center, [10.25, 20.75]);
    assert_eq!(synced.propagated_from.zoom_level, 3);
    assert_eq!(synced.propagated_to.viewport_id, "minimap");
    assert_eq!(synced.propagated_to.center, [10.25, 20.75]);
    assert_eq!(synced.propagated_to.zoom_level, 3);
}

#[test]
fn camera_sync_propagates_one_way_center_and_zoom_without_mutating_source_id() {
    let world = ViewportCameraState::new("world", [31.125, -8.625], 7);
    let minimap = ViewportCameraState::new("minimap", [0.0, 0.0], 1);

    let link = link_cameras("world", "minimap", CameraLinkMode::OneWay);
    let synced = sync_camera(&link, &world, &minimap).expect("expected linked camera sync");

    assert_eq!(synced.propagated_from.viewport_id, "world");
    assert_eq!(synced.propagated_to.viewport_id, "minimap");
    assert_eq!(synced.propagated_to.center, [31.125, -8.625]);
    assert_eq!(synced.propagated_to.zoom_level, 7);
}

#[test]
fn camera_sync_returns_none_for_mismatched_viewport_wiring() {
    let world = ViewportCameraState::new("observer", [31.125, -8.625], 7);
    let minimap = ViewportCameraState::new("overview", [0.0, 0.0], 1);

    let link = link_cameras("world", "minimap", CameraLinkMode::OneWay);

    assert!(sync_camera(&link, &world, &minimap).is_none());
}

#[test]
fn viewport_navigation_never_arms_command_submission() {
    let navigation = NavigationEvent::viewport_navigated("minimap");

    assert_eq!(navigation.viewport_id, "minimap");
    assert!(!navigation.command_armed);
}
