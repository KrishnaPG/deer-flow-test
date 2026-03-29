//! Typed outbound command payloads.
//!
//! Replaces ad-hoc `json!({})` macro usage with zero-allocation
//! serializable structs for each command type.  Each struct borrows
//! its data from the caller, so serialization produces no intermediate
//! `serde_json::Value` heap objects.

use serde::Serialize;

/// Empty payload for commands that take no arguments
/// (e.g. `list_threads`, `create_thread`, `list_models`).
#[derive(Debug, Serialize)]
pub struct EmptyPayload;

/// Payload for commands that operate on a single thread by ID.
#[derive(Debug, Serialize)]
pub struct ThreadIdPayload<'a> {
    pub thread_id: &'a str,
}

/// Payload for the `rename_thread` command.
#[derive(Debug, Serialize)]
pub struct RenameThreadPayload<'a> {
    pub thread_id: &'a str,
    pub title: &'a str,
}

/// Nested context block inside [`SendMessagePayload`].
#[derive(Debug, Serialize)]
pub struct MessageContext<'a> {
    pub model_name: Option<&'a str>,
    pub mode: &'a str,
    pub reasoning_effort: Option<&'a str>,
}

/// Payload for the `send_message` command.
#[derive(Debug, Serialize)]
pub struct SendMessagePayload<'a> {
    pub thread_id: &'a str,
    pub text: &'a str,
    /// Attachment file paths — must be owned because they are converted
    /// from `PathBuf` via `to_string_lossy().to_string()`.
    pub attachments: Vec<String>,
    pub context: MessageContext<'a>,
}

/// Payload for the `resolve_artifact` command.
#[derive(Debug, Serialize)]
pub struct ResolveArtifactPayload<'a> {
    pub thread_id: &'a str,
    pub virtual_path: &'a str,
}
