use std::process::Stdio;

use tokio::process::{Child, Command};

use crate::tui_gateway::{TuiGatewayCommand, TuiGatewayProcessId};

#[derive(Debug)]
pub struct TuiGatewayProcess {
    pub process_id: TuiGatewayProcessId,
    pub child: Child,
}

pub async fn spawn_tui_gateway(
    command: &TuiGatewayCommand,
) -> Result<TuiGatewayProcess, std::io::Error> {
    let process_id = TuiGatewayProcessId::generate();
    let mut child_command = Command::new(&command.program);
    child_command
        .args(&command.args)
        .current_dir(&command.working_directory)
        .envs(command.env.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = child_command.spawn()?;
    Ok(TuiGatewayProcess { process_id, child })
}
