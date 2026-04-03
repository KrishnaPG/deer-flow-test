use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldOverlayFreshness {
    pub status: &'static str,
    pub stale_reason: Option<&'static str>,
    pub source_event_id: Option<String>,
}

impl Default for WorldOverlayFreshness {
    fn default() -> Self {
        Self {
            status: "fresh",
            stale_reason: None,
            source_event_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct TemporalState {
    pub layout_instance: u64,
    pub mode: &'static str,
    pub cursor_id: Option<String>,
    pub is_stale: bool,
    pub stream_state: Option<String>,
    pub degraded: bool,
    pub world_overlay_freshness: WorldOverlayFreshness,
}

impl TemporalState {
    pub fn historical(cursor_id: &str) -> Self {
        Self {
            layout_instance: 0,
            mode: "historical",
            cursor_id: Some(cursor_id.into()),
            is_stale: false,
            stream_state: None,
            degraded: false,
            world_overlay_freshness: WorldOverlayFreshness::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalAction {
    LateEventInserted { event_id: String },
    LayoutRestored { layout_instance: u64 },
    ReturnToLiveTail,
}

pub fn reduce_temporal_state(mut state: TemporalState, action: TemporalAction) -> TemporalState {
    match action {
        TemporalAction::LateEventInserted { event_id } if state.mode == "historical" => {
            state.is_stale = true;
            state.stream_state = Some("degraded".into());
            state.degraded = true;
            state.world_overlay_freshness = WorldOverlayFreshness {
                status: "stale",
                stale_reason: Some("late_event_inserted"),
                source_event_id: Some(event_id),
            };
        }
        TemporalAction::LateEventInserted { event_id } => {
            state.mode = "live_tail";
            state.stream_state = Some("degraded".into());
            state.degraded = true;
            state.world_overlay_freshness = WorldOverlayFreshness {
                status: "stale",
                stale_reason: Some("late_event_inserted"),
                source_event_id: Some(event_id),
            };
        }
        TemporalAction::LayoutRestored { layout_instance } => {
            state.layout_instance = layout_instance;
        }
        TemporalAction::ReturnToLiveTail => {
            state.mode = "live_tail";
            state.cursor_id = None;
            state.is_stale = false;
            state.stream_state = Some("live".into());
            state.degraded = false;
            state.world_overlay_freshness = WorldOverlayFreshness::default();
        }
    }

    state
}
