use deer_pipeline_raw_sources::{run_hermes_smoke_prompt_with_text, AcpResponseStreamEventKind};

#[tokio::test]
#[ignore]
async fn hermes_smoke_prompt_produces_answer_and_intermediate_data() {
    let report = run_hermes_smoke_prompt_with_text("what are your skills?")
        .await
        .expect("Hermes smoke run should succeed");

    assert!(
        report.live_events.iter().any(|event| matches!(
            event.kind,
            AcpResponseStreamEventKind::AssistantTextFinal { .. }
        )),
        "expected at least one final assistant text event"
    );
    assert!(
        report.assistant_text.is_some(),
        "expected a final assistant answer"
    );
    assert!(!report.raw_events.is_empty(), "expected raw capture events");
    assert!(
        !report.replayed_events.is_empty(),
        "expected replayable raw events"
    );
}
