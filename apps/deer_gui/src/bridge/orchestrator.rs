//! OrchestratorClient trait — high-level orchestrator API.
//!
//! Systems depend on this trait rather than a concrete client,
//! enabling transport swaps and test fakes.

use std::path::PathBuf;

/// High-level orchestrator operations.
///
/// Each method returns a `request_id` that can be correlated with
/// an eventual `BridgeEvent::Response`.
pub trait OrchestratorClient: Send + Sync + 'static {
    /// List all threads.
    fn list_threads(&self) -> Result<String, String>;

    /// Get a single thread by ID.
    fn get_thread(&self, thread_id: &str) -> Result<String, String>;

    /// Create a new thread.
    fn create_thread(&self) -> Result<String, String>;

    /// Rename a thread.
    fn rename_thread(&self, thread_id: &str, title: &str) -> Result<String, String>;

    /// Delete a thread.
    fn delete_thread(&self, thread_id: &str) -> Result<String, String>;

    /// List available models.
    fn list_models(&self) -> Result<String, String>;

    /// Send a chat message in a thread.
    fn send_message(
        &self,
        thread_id: &str,
        text: &str,
        attachments: &[PathBuf],
        model_name: Option<&str>,
        mode: &str,
        reasoning_effort: Option<&str>,
    ) -> Result<String, String>;

    /// Resolve an artifact's host path.
    fn resolve_artifact(&self, thread_id: &str, virtual_path: &str) -> Result<String, String>;
}
