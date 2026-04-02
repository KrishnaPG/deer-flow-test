use deer_runtime_read_models::{ArtifactPanelState, ArtifactRequestState};

#[test]
fn artifact_panel_state_tracks_explicit_preview_and_open_requests() {
    let state = ArtifactPanelState {
        selected_artifact_id: Some("artifact_2".into()),
        preview_request_state: ArtifactRequestState::Requested {
            artifact_id: "artifact_2".into(),
        },
        open_request_state: ArtifactRequestState::Requested {
            artifact_id: "artifact_2".into(),
        },
    };

    assert_eq!(state.selected_artifact_id.as_deref(), Some("artifact_2"));
    assert_eq!(
        state.preview_request_state,
        ArtifactRequestState::Requested {
            artifact_id: "artifact_2".into(),
        }
    );
    assert_eq!(
        state.open_request_state,
        ArtifactRequestState::Requested {
            artifact_id: "artifact_2".into(),
        }
    );
}
