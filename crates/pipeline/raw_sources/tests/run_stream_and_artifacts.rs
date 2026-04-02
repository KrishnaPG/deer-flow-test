use deer_pipeline_raw_sources::{
    load_stream_fixture, preview_artifact, AdapterEvent, ArtifactAccess, RawStreamEvent,
};

#[test]
fn loads_live_stream_events_and_keeps_artifact_access_mediated() {
    let events = load_stream_fixture(include_str!("fixtures/live_run_stream.json")).unwrap();
    let preview = preview_artifact("thread_1", "artifact_2").unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        AdapterEvent::Deerflow(RawStreamEvent::Clarification { .. })
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AdapterEvent::Deerflow(RawStreamEvent::TaskProgress { .. })
    )));
    assert_eq!(
        preview,
        ArtifactAccess::PreviewPayload {
            mime: "image/png".into()
        }
    );
}
