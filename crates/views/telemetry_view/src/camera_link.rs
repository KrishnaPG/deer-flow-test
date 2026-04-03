use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CameraLinkMode {
    OneWay,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CameraLink {
    pub source_viewport_id: String,
    pub target_viewport_id: String,
    pub mode: CameraLinkMode,
}

pub fn link_cameras(
    source_viewport_id: &str,
    target_viewport_id: &str,
    mode: CameraLinkMode,
) -> CameraLink {
    CameraLink {
        source_viewport_id: source_viewport_id.to_owned(),
        target_viewport_id: target_viewport_id.to_owned(),
        mode,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ViewportCameraState {
    pub viewport_id: String,
    pub center: [f32; 2],
    pub zoom_level: u8,
}

impl ViewportCameraState {
    pub fn new(viewport_id: &str, center: [f32; 2], zoom_level: u8) -> Self {
        Self {
            viewport_id: viewport_id.to_owned(),
            center,
            zoom_level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncedCameraLink {
    pub propagated_from: ViewportCameraState,
    pub propagated_to: ViewportCameraState,
    pub mode: CameraLinkMode,
}

pub fn sync_camera(
    link: &CameraLink,
    source: &ViewportCameraState,
    target: &ViewportCameraState,
) -> Option<SyncedCameraLink> {
    if source.viewport_id != link.source_viewport_id
        || target.viewport_id != link.target_viewport_id
    {
        return None;
    }

    let propagated_from = source.clone();
    let propagated_to = ViewportCameraState {
        viewport_id: target.viewport_id.clone(),
        center: source.center,
        zoom_level: source.zoom_level,
    };

    Some(SyncedCameraLink {
        propagated_from,
        propagated_to,
        mode: link.mode.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavigationEvent {
    pub viewport_id: String,
    pub command_armed: bool,
}

impl NavigationEvent {
    pub fn viewport_navigated(viewport_id: &str) -> Self {
        Self {
            viewport_id: viewport_id.to_owned(),
            command_armed: false,
        }
    }
}
