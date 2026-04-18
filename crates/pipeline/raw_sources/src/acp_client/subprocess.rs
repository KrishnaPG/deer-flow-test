use std::path::PathBuf;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tracing::instrument;

use crate::acp_client::ids::AcpSubprocessId;

/// Material lifecycle states for one Hermes ACP subprocess instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpSubprocessLifecycleKind {
    Spawned,
    Exited,
    Crashed,
    TimedOut,
}

/// Command-line configuration for spawning Hermes ACP.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpSubprocessCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
}

impl AcpSubprocessCommand {
    pub fn new(executable: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            executable,
            args: vec!["-m".to_string(), "acp_adapter.entry".to_string()],
            working_directory,
        }
    }
}

/// Running Hermes ACP child process wired for stdio JSON-RPC.
#[derive(Debug)]
pub struct AcpSubprocessEvent {
    pub acp_subprocess_id: AcpSubprocessId,
    pub child: Child,
}

#[instrument(skip(command), fields(acp_subprocess_id = %AcpSubprocessId::generate()))]
pub async fn spawn_acp_subprocess(
    command: &AcpSubprocessCommand,
) -> Result<AcpSubprocessEvent, std::io::Error> {
    let acp_subprocess_id = AcpSubprocessId::generate();
    let mut child_command = Command::new(&command.executable);
    child_command
        .args(&command.args)
        .current_dir(&command.working_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = child_command.spawn()?;

    Ok(AcpSubprocessEvent {
        acp_subprocess_id,
        child,
    })
}
