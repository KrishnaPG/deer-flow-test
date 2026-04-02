use deer_foundation_domain::AnyRecord;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptEntryVm {
    pub record_id: String,
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunStatusVm {
    pub run_id: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskProgressVm {
    pub task_id: String,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptVm {
    pub entries: Vec<TranscriptEntryVm>,
    pub run_status: RunStatusVm,
    pub tasks: Vec<TaskProgressVm>,
}

pub fn derive_transcript_vm(records: &[AnyRecord]) -> TranscriptVm {
    let mut entries = Vec::new();
    let mut run_status = RunStatusVm {
        run_id: String::new(),
        state: String::new(),
    };
    let mut tasks = Vec::new();

    for record in records {
        match record {
            AnyRecord::Message(message) => entries.push(TranscriptEntryVm {
                record_id: message.record_id().to_string(),
                role: message.body.role.clone(),
                text: message.body.text.clone(),
            }),
            AnyRecord::Run(run) => {
                run_status = RunStatusVm {
                    run_id: run.record_id().to_string(),
                    state: run.body.status.clone(),
                };
            }
            AnyRecord::Task(task) => tasks.push(TaskProgressVm {
                task_id: task.record_id().to_string(),
                title: task.body.label.clone(),
                state: task.body.status.clone(),
            }),
            _ => {}
        }
    }

    TranscriptVm {
        entries,
        run_status,
        tasks,
    }
}
