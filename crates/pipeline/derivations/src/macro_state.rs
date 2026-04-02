use deer_foundation_contracts::CanonicalRecord;
use deer_foundation_domain::AnyRecord;
use serde::Serialize;

use crate::common::VmBacklink;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroStateRowVm {
    pub kind: &'static str,
    pub source_record_id: String,
    pub panel_target: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroStateVm {
    pub rows: Vec<MacroStateRowVm>,
    pub backlinks: Vec<VmBacklink>,
}

pub fn derive_macro_state_vm(records: &[AnyRecord]) -> MacroStateVm {
    let mut rows = Vec::new();
    let mut backlinks = Vec::new();

    for record in records {
        match record {
            AnyRecord::Task(task) => {
                rows.push(MacroStateRowVm {
                    kind: "task",
                    source_record_id: task.record_id().to_string(),
                    panel_target: "task_detail",
                });
                backlinks.push(VmBacklink {
                    source_record_id: task.record_id().to_string(),
                    level: format!("{:?}", task.header().level),
                    plane: format!("{:?}", task.header().plane),
                    panel_target: "task_detail",
                });
            }
            AnyRecord::Artifact(artifact) => {
                rows.push(MacroStateRowVm {
                    kind: "artifact",
                    source_record_id: artifact.record_id().to_string(),
                    panel_target: "artifact_detail",
                });
                backlinks.push(VmBacklink {
                    source_record_id: artifact.record_id().to_string(),
                    level: format!("{:?}", artifact.header().level),
                    plane: format!("{:?}", artifact.header().plane),
                    panel_target: "artifact_detail",
                });
            }
            _ => {}
        }
    }

    MacroStateVm { rows, backlinks }
}
