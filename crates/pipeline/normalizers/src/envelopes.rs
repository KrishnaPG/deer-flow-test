use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "generator", rename_all = "snake_case")]
pub enum RawEnvelopeBatch {
    Deerflow(DeerFlowBatch),
    Hermes(HermesBatch),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeerFlowBatch {
    pub session: RawSessionEnvelope,
    pub run: RawRunEnvelope,
    pub events: Vec<RawEventEnvelope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HermesBatch {
    pub session: RawSessionEnvelope,
    pub run: RawRunEnvelope,
    pub events: Vec<RawEventEnvelope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSessionEnvelope {
    pub session_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRunEnvelope {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RawEventEnvelope {
    Message {
        message_id: String,
        role: String,
        text: String,
        level: String,
    },
    Task {
        task_id: String,
        title: String,
        state: String,
    },
    Artifact {
        artifact_id: String,
        name: String,
        status: String,
        as_is_hash: String,
    },
    RuntimeStatus {
        state: String,
    },
    Clarification {
        clarification_id: String,
        prompt: String,
    },
}
