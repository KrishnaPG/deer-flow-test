use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpatialCameraState {
    pub center: [f32; 2],
    pub zoom_level: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct SpatialShellState {
    pub viewport_id: Option<String>,
    pub camera_state: Option<SpatialCameraState>,
    pub last_camera_sync_source: Option<String>,
    pub command_armed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpatialShellAction {
    ViewportNavigated {
        viewport_id: String,
    },
    CameraSynced {
        source: String,
    },
    WorldMoved {
        viewport_id: String,
        center: [f32; 2],
        zoom_level: u8,
    },
}

pub fn reduce_spatial_shell_state(
    mut state: SpatialShellState,
    action: SpatialShellAction,
) -> SpatialShellState {
    match action {
        SpatialShellAction::ViewportNavigated { viewport_id } => {
            state.viewport_id = Some(viewport_id)
        }
        SpatialShellAction::CameraSynced { source } => state.last_camera_sync_source = Some(source),
        SpatialShellAction::WorldMoved {
            viewport_id,
            center,
            zoom_level,
        } => {
            state.viewport_id = Some(viewport_id);
            state.camera_state = Some(SpatialCameraState { center, zoom_level });
        }
    }

    state.command_armed = false;
    state
}
