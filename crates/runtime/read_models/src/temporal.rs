use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct TemporalState {
    pub mode: &'static str,
    pub cursor_id: Option<String>,
    pub is_stale: bool,
    pub stream_state: Option<String>,
    pub degraded: bool,
}

impl TemporalState {
    pub fn historical(cursor_id: &str) -> Self {
        Self {
            mode: "historical",
            cursor_id: Some(cursor_id.into()),
            is_stale: false,
            stream_state: None,
            degraded: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalAction {
    LateEventInserted { event_id: String },
    ReturnToLiveTail,
}

pub fn reduce_temporal_state(mut state: TemporalState, action: TemporalAction) -> TemporalState {
    match action {
        TemporalAction::LateEventInserted { .. } if state.mode == "historical" => {
            state.is_stale = true;
            state.stream_state = Some("degraded".into());
            state.degraded = true;
        }
        TemporalAction::LateEventInserted { .. } => {
            state.mode = "live_tail";
            state.stream_state = Some("degraded".into());
            state.degraded = true;
        }
        TemporalAction::ReturnToLiveTail => {
            state.mode = "live_tail";
            state.cursor_id = None;
            state.is_stale = false;
            state.stream_state = Some("live".into());
            state.degraded = false;
        }
    }

    state
}
