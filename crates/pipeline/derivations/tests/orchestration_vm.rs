use deer_foundation_contracts::{IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{
    AnyRecord, ClarificationBody, ClarificationRecord, MessageBody, MessageRecord, RunBody,
    RunRecord, RuntimeStatusBody, RuntimeStatusRecord, SessionBody, SessionRecord, TaskBody,
    TaskRecord, ToolCallBody, ToolCallRecord,
};
use deer_pipeline_derivations::derive_transcript_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_transcript_and_run_status_vms_from_canonical_records() {
    let records = vec![
        AnyRecord::Session(SessionRecord::new(
            RecordId::from_static("session_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("session_1"), None, None, None),
            LineageMeta::root(),
            SessionBody {
                name: "Survey the ridge".into(),
            },
        )),
        AnyRecord::Run(RunRecord::new(
            RecordId::from_static("run_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("run_1"), None, None, None),
            LineageMeta::root(),
            RunBody {
                title: "Survey the ridge".into(),
                status: "running".into(),
            },
        )),
        AnyRecord::Message(MessageRecord::new(
            RecordId::from_static("msg_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("msg_1"), None, None, None),
            LineageMeta::root(),
            MessageBody {
                role: "operator".into(),
                text: "Survey the ridge".into(),
            },
        )),
        AnyRecord::Task(TaskRecord::new(
            RecordId::from_static("task_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("task_1"), None, None, None),
            LineageMeta::root(),
            TaskBody {
                label: "Gather terrain notes".into(),
                status: "running".into(),
            },
        )),
        AnyRecord::ToolCall(ToolCallRecord::new(
            RecordId::from_static("tool_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("tool_1"), None, None, None),
            LineageMeta::root(),
            ToolCallBody {
                tool_name: "map_scan".into(),
                status: "running".into(),
            },
        )),
        AnyRecord::Clarification(ClarificationRecord::new(
            RecordId::from_static("clar_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("clar_1"), None, None, None),
            LineageMeta::root(),
            ClarificationBody {
                prompt: "Confirm the survey radius.".into(),
                resolved: false,
            },
        )),
        AnyRecord::RuntimeStatus(RuntimeStatusRecord::new(
            RecordId::from_static("runtime_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("runtime_1"), None, None, None),
            LineageMeta::root(),
            RuntimeStatusBody {
                status: "live".into(),
                detail: "stream connected".into(),
            },
        )),
    ];

    let transcript = derive_transcript_vm(&records);

    assert_yaml_snapshot!(transcript, @r#"
entries:
  - record_id: msg_1
    role: operator
    text: Survey the ridge
  - record_id: tool_1
    role: tool
    text: map_scan
  - record_id: clar_1
    role: clarification
    text: Confirm the survey radius.
run_status:
  run_id: run_1
  state: live
tasks:
  - task_id: task_1
    title: Gather terrain notes
    state: running
"#);
}

#[test]
fn applies_runtime_status_even_when_it_arrives_before_run_record() {
    let records = vec![
        AnyRecord::RuntimeStatus(RuntimeStatusRecord::new(
            RecordId::from_static("runtime_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("runtime_1"), None, None, None),
            LineageMeta::root(),
            RuntimeStatusBody {
                status: "live".into(),
                detail: "stream connected".into(),
            },
        )),
        AnyRecord::Run(RunRecord::new(
            RecordId::from_static("run_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("run_1"), None, None, None),
            LineageMeta::root(),
            RunBody {
                title: "Survey the ridge".into(),
                status: "running".into(),
            },
        )),
    ];

    let transcript = derive_transcript_vm(&records);

    assert_yaml_snapshot!(transcript, @r#"
entries: []
run_status:
  run_id: run_1
  state: live
tasks: []
"#);
}
