/// Internal lifecycle events for storage diagnostics; not public storage facts.
pub enum InternalLifecycleEvent {
    WriteStarted,
    RetryScheduled,
    WorkerFailure,
}

impl InternalLifecycleEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WriteStarted => "WriteStarted",
            Self::RetryScheduled => "RetryScheduled",
            Self::WorkerFailure => "WorkerFailure",
        }
    }
}
