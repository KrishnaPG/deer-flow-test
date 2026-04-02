pub struct LiveChatScenario {
    pub thread_fixture: &'static str,
    pub stream_fixture: &'static str,
    pub upload_name: &'static str,
}

pub fn live_chat_scenario() -> LiveChatScenario {
    LiveChatScenario {
        thread_fixture: r#"{"thread_id":"thread_1","title":"Resume ridge survey"}"#,
        stream_fixture: r#"[
  {"kind":"runtime_status","state":"live"},
  {"kind":"message_delta","message_id":"msg_1","text":"Uploading briefing"},
  {"kind":"tool_call","tool_call_id":"tool_1","tool_name":"map_scan"},
  {"kind":"task_progress","task_id":"task_1","state":"running","label":"Gather terrain notes"},
  {"kind":"clarification","clarification_id":"clar_1","prompt":"Confirm the ridge path."},
  {"kind":"artifact_presented","artifact_id":"artifact_2","name":"scan.png"}
]"#,
        upload_name: "briefing.pdf",
    }
}
