use deer_runtime_world_projection::WorldProjection;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MinimapView {
    pub viewport_id: String,
    pub marker_count: usize,
    pub command_armed: bool,
}

pub fn render_minimap_view(projection: &WorldProjection, viewport_id: &str) -> MinimapView {
    MinimapView {
        viewport_id: viewport_id.to_owned(),
        marker_count: projection.objects.len(),
        command_armed: false,
    }
}
