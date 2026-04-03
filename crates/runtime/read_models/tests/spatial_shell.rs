use deer_runtime_read_models::{reduce_spatial_shell_state, SpatialShellAction, SpatialShellState};

#[test]
fn spatial_shell_tracks_viewport_navigation_and_camera_sync_without_command_targeting() {
    let state = SpatialShellState::default();

    let state = reduce_spatial_shell_state(
        state,
        SpatialShellAction::ViewportNavigated {
            viewport_id: "world".into(),
        },
    );
    let state = reduce_spatial_shell_state(
        state,
        SpatialShellAction::CameraSynced {
            source: "minimap".into(),
        },
    );

    assert_eq!(state.viewport_id.as_deref(), Some("world"));
    assert_eq!(state.last_camera_sync_source.as_deref(), Some("minimap"));
    assert!(!state.command_armed);
}

#[test]
fn spatial_shell_keeps_world_movement_shell_local_and_intent_neutral() {
    let state = SpatialShellState {
        command_armed: true,
        ..Default::default()
    };

    let state = reduce_spatial_shell_state(
        state,
        SpatialShellAction::WorldMoved {
            viewport_id: "world".into(),
            center: [128.25, 64.5],
            zoom_level: 3,
        },
    );

    assert_eq!(state.viewport_id.as_deref(), Some("world"));
    assert_eq!(
        state.camera_state.as_ref().map(|camera| camera.center),
        Some([128.25, 64.5])
    );
    assert_eq!(
        state.camera_state.as_ref().map(|camera| camera.zoom_level),
        Some(3)
    );
    assert!(!state.command_armed);
}
