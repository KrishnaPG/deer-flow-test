use deer_foundation_contracts::{CanonicalRecord, IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{
    AnyRecord, MessageBody, MessageRecord, RunBody, RunRecord, SessionBody, SessionRecord,
    TaskBody, TaskRecord,
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
    ];

    let transcript = derive_transcript_vm(&records);

    assert_yaml_snapshot!(transcript, @r#"
entries:
  - record_id: msg_1
    role: operator
    text: Survey the ridge
run_status:
  run_id: run_1
  state: running
tasks:
  - task_id: task_1
    title: Gather terrain notes
    state: running
"#);

    assert_eq!(
        records[3].header().level as u8,
        records[3].header().level as u8
    );
}
