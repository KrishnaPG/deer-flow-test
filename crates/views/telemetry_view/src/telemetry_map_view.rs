use deer_runtime_world_projection::WorldProjection;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelemetryMapMarker {
    pub source_record_id: String,
    pub kind: &'static str,
    pub drill_down_target: &'static str,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelemetryMapView {
    pub marker_count: usize,
    pub selected_marker_id: Option<String>,
    pub markers: Vec<TelemetryMapMarker>,
    pub command_armed: bool,
}

pub fn render_telemetry_map_view(
    projection: &WorldProjection,
    selected_marker_id: Option<&str>,
) -> TelemetryMapView {
    let markers = projection
        .objects
        .iter()
        .map(|object| TelemetryMapMarker {
            source_record_id: object.source_record_id.clone(),
            kind: object.kind,
            drill_down_target: object.drill_down_target,
            selected: selected_marker_id == Some(object.source_record_id.as_str()),
        })
        .collect::<Vec<_>>();

    TelemetryMapView {
        marker_count: markers.len(),
        selected_marker_id: selected_marker_id.map(str::to_owned),
        markers,
        command_armed: false,
    }
}
