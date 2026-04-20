use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol as acp;
use bytes::Bytes;
use tempfile::TempDir;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;

use deer_foundation_paths::BaseDir;

use crate::acp_client::{
    AcpClientSessionConfig, AcpResponseStreamEvent, AcpResponseStreamEventKind, ChatRunId,
    ChatSessionId, OurAcpClient, RawEventFanout, RawEventReader, RedbRawEventPublisher,
};

const SMOKE_PROMPT: &str = "what are your skills?";
const OVERALL_TIMEOUT: Duration = Duration::from_secs(120);
const SETTLE_TIMEOUT: Duration = Duration::from_millis(750);
const SHARED_HERMES_ITEMS: &[&str] = &[
    ".env",
    "auth.json",
    "checkpoints",
    "config.yaml",
    "gateway_state.json",
    "hermes-agent",
    "memories",
    "models_dev_cache.json",
    "pastes",
    "sessions",
    "skills",
];
const COPIED_HERMES_ITEMS: &[&str] = &["state.db", "state.db-shm", "state.db-wal"];

#[derive(Debug)]
pub struct HermesSmokeReport {
    pub prompt: String,
    pub acp_session_id: ChatSessionId,
    pub raw_session_id: ChatSessionId,
    pub run_id: ChatRunId,
    pub assistant_text: Option<String>,
    pub live_events: Vec<AcpResponseStreamEvent>,
    pub raw_events: Vec<(ChatSessionId, u64, Bytes)>,
    pub replayed_events: Vec<Bytes>,
}

pub async fn run_hermes_smoke_prompt() -> Result<HermesSmokeReport, std::io::Error> {
    run_hermes_smoke_prompt_with_text(SMOKE_PROMPT).await
}

pub async fn run_hermes_smoke_prompt_with_text(
    prompt: impl Into<String>,
) -> Result<HermesSmokeReport, std::io::Error> {
    let prompt = prompt.into();
    let temp_dir = TempDir::new()?;
    let hermes_home = prepare_hermes_home()?;
    let _hermes_home_guard = ScopedEnvVar::set("HERMES_HOME", hermes_home.path());
    let base_dir = BaseDir::new(temp_dir.path().to_path_buf());
    std::fs::create_dir_all(base_dir.incoming().as_path())?;

    let staging_db_path = base_dir.incoming().staging_db();
    let publisher =
        Arc::new(RedbRawEventPublisher::new(&staging_db_path).map_err(io_error_from_string)?);
    let raw_fanout = RawEventFanout::default();
    let client = OurAcpClient::new(Arc::clone(&publisher) as Arc<_>, raw_fanout);

    let mut live_rx = client.fanout().subscribe();
    let mut raw_rx = client.raw_fanout().subscribe();

    let config =
        AcpClientSessionConfig::discover()?.with_working_directory(temp_dir.path().to_path_buf());
    let acp_session_id = client.connect_session(config).await?;
    let run_id = client
        .start_prompt_run(
            &acp_session_id,
            vec![acp::ContentBlock::from(prompt.clone())],
        )
        .await?;

    let mut live_events = Vec::new();
    let mut raw_events = Vec::new();
    let mut assistant_text = None;
    let mut raw_session_id = None;
    let mut saw_final_text = false;
    let overall_deadline = Instant::now() + OVERALL_TIMEOUT;

    loop {
        let now = Instant::now();
        if now >= overall_deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Hermes smoke prompt timed out waiting for a final answer",
            ));
        }

        let wait = if saw_final_text {
            SETTLE_TIMEOUT.min(overall_deadline - now)
        } else {
            overall_deadline - now
        };

        tokio::select! {
            res = live_rx.recv() => {
                match res {
                    Ok(event) => {
                        if let AcpResponseStreamEventKind::AssistantTextFinal { text } = &event.kind {
                            assistant_text.get_or_insert_with(|| text.clone());
                            saw_final_text = true;
                        }
                        live_events.push(event);
                    }
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                }
            }
            res = raw_rx.recv() => {
                match res {
                    Ok((session_id, sequence, bytes)) => {
                        if raw_session_id.is_none() {
                            raw_session_id = Some(session_id.clone());
                        }
                        raw_events.push((session_id, sequence, bytes));
                    }
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                }
            }
            _ = tokio::time::sleep(wait) => {
                if saw_final_text {
                    break;
                }
            }
        }
    }

    let raw_session_id = raw_session_id.unwrap_or_else(|| acp_session_id.clone());
    let replayed_events = publisher
        .read_session_events(&raw_session_id)
        .await
        .map_err(|err| io_error_from_string(format!("Failed to replay raw events: {err}")))?;

    if replayed_events.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Hermes smoke prompt produced no replayable raw events",
        ));
    }

    Ok(HermesSmokeReport {
        prompt,
        acp_session_id,
        raw_session_id,
        run_id,
        assistant_text,
        live_events,
        raw_events,
        replayed_events,
    })
}

fn io_error_from_string(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, message.into())
}

fn prepare_hermes_home() -> Result<TempDir, std::io::Error> {
    let source_home = source_hermes_home();
    let hermes_home = TempDir::new()?;
    std::fs::create_dir_all(hermes_home.path().join("logs"))?;

    for item in SHARED_HERMES_ITEMS {
        let source = source_home.join(item);
        let target = hermes_home.path().join(item);
        if !source.exists() {
            continue;
        }

        if source.is_dir() {
            symlink_dir(&source, &target)?;
        } else {
            symlink_file(&source, &target)?;
        }
    }

    for item in COPIED_HERMES_ITEMS {
        let source = source_home.join(item);
        let target = hermes_home.path().join(item);
        if source.exists() {
            std::fs::copy(source, target)?;
        }
    }

    Ok(hermes_home)
}

fn source_hermes_home() -> std::path::PathBuf {
    if let Some(home) = std::env::var_os("HERMES_HOME") {
        return home.into();
    }

    let mut home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.push(".hermes");
    home
}

#[cfg(unix)]
fn symlink_dir(source: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(unix)]
fn symlink_file(source: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(source, target)
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}
