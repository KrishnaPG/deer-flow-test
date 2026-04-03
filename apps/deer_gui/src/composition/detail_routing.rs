use bevy::log::debug;

use super::scripted_scenarios::FIRST_ARTIFACT_SELECTION;

pub const ARTIFACT_DETAIL_TARGET: &str = "artifact_detail";
pub const INSPECTOR_TARGET: &str = "inspector";
pub const DETAIL_ROUTE_SOURCE: &str = "selection_broker";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailRoute {
    pub target: &'static str,
    pub source: &'static str,
}

pub fn route_to_detail(selection_id: &str) -> DetailRoute {
    debug!("composition::detail_routing::route_to_detail",);

    match selection_id {
        FIRST_ARTIFACT_SELECTION => DetailRoute {
            target: ARTIFACT_DETAIL_TARGET,
            source: DETAIL_ROUTE_SOURCE,
        },
        _ => DetailRoute {
            target: INSPECTOR_TARGET,
            source: DETAIL_ROUTE_SOURCE,
        },
    }
}
