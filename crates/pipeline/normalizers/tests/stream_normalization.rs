use deer_pipeline_derivations::{derive_artifact_shelf_vm, derive_transcript_vm};
use deer_pipeline_normalizers::normalize_stream_batch;
use deer_pipeline_raw_sources::load_stream_fixture;

#[test]
fn normalizes_live_stream_events_with_tool_calls_tasks_and_presented_artifacts() {
    let events = load_stream_fixture(include_str!(
        "../../raw_sources/tests/fixtures/live_run_stream.json"
    ))
    .unwrap();

    let normalized =
        normalize_stream_batch("session_1", "Survey the ridge", "run_1", "running", &events)
            .unwrap();

    let transcript = derive_transcript_vm(&normalized.records);
    let shelf = derive_artifact_shelf_vm(&normalized.records);

    assert!(transcript
        .entries
        .iter()
        .any(|entry| entry.role == "tool" && entry.text == "map_scan"));
    assert!(transcript
        .tasks
        .iter()
        .any(|task| task.title == "Gather terrain notes" && task.state == "running"));
    assert!(shelf
        .entries
        .iter()
        .any(|artifact| { artifact.artifact_id == "artifact_2" && artifact.preview_supported }));
}
