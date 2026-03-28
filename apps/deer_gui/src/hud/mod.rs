//! HUD overlay module — semi-transparent panels rendered via `bevy_egui`.
//!
//! The HUD sits at Z1 (overlay layer) and provides the tactical
//! command interface: top bar, side panels, console, event ticker,
//! center canvas, and modal overlays.

mod bottom_console;
mod center_canvas;
mod event_ticker;
mod left_panel;
mod modal;
mod plugin;
mod right_inspector;
mod styles;
mod top_bar;

// Re-export the plugin and the shared state.
pub use plugin::HudPlugin;
pub use styles::{
    agent_state_color, draw_progress_bar, glass_panel_frame, mission_status_color,
    side_panel_frame, top_bar_frame,
};

use bevy::prelude::*;

use crate::models::{ChatMessage, ModelInfo, ThreadRecord, ThreadSummary};

// ---------------------------------------------------------------------------
// HUD state resource
// ---------------------------------------------------------------------------

/// Central state resource for all HUD panels.
///
/// Populated by bridge events and world systems; read by each HUD panel
/// system every frame. Keeps the HUD decoupled from the bridge protocol.
#[derive(Resource)]
pub struct HudState {
    // -- Top bar metrics --
    /// Number of agents currently active.
    pub active_agents: u32,
    /// Number of agents retrying.
    pub retrying_agents: u32,
    /// Number of agents in error state.
    pub failed_agents: u32,
    /// Current token throughput (tokens/sec).
    pub tokens_per_sec: f32,
    /// Estimated cost rate (USD/hour).
    pub cost_per_hour: f32,
    /// Whether the backend system is online.
    pub system_online: bool,

    // -- Left panel: missions --
    /// Active missions for the left panel.
    pub missions: Vec<MissionSummary>,

    // -- Right inspector --
    /// Currently selected entity's inspector data (if any).
    pub selected_entity: Option<EntityInspectorData>,

    // -- Event ticker --
    /// Rolling event log entries.
    pub event_log: Vec<EventLogEntry>,

    // -- Bottom console --
    /// Current text in the command input.
    pub command_input: String,
    /// Active command mode.
    pub command_mode: CommandMode,

    // -- Center canvas --
    /// Which center canvas view is active.
    pub center_mode: CenterCanvasMode,

    // -- Modals --
    /// Which modal is currently shown (if any).
    pub show_modal: Option<ModalKind>,

    // -- Thread/model data from bridge --
    /// Available threads from the bridge.
    pub threads: Vec<ThreadSummary>,
    /// Currently selected thread ID.
    pub selected_thread_id: Option<String>,
    /// Cached full thread record for the selected thread.
    pub thread_cache: Option<ThreadRecord>,
    /// Available AI models.
    pub models: Vec<ModelInfo>,
    /// Currently selected model name.
    pub selected_model: Option<String>,
    /// Thread ID of the currently streaming conversation.
    pub streaming_thread_id: Option<String>,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            active_agents: 0,
            retrying_agents: 0,
            failed_agents: 0,
            tokens_per_sec: 0.0,
            cost_per_hour: 0.0,
            system_online: false,
            missions: Vec::new(),
            selected_entity: None,
            event_log: Vec::new(),
            command_input: String::new(),
            command_mode: CommandMode::Direct,
            center_mode: CenterCanvasMode::WorldView,
            show_modal: None,
            threads: Vec::new(),
            selected_thread_id: None,
            thread_cache: None,
            models: Vec::new(),
            selected_model: None,
            streaming_thread_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Summary of a mission for the left panel list.
#[derive(Debug, Clone)]
pub struct MissionSummary {
    /// Display name of the mission.
    pub name: String,
    /// Current status.
    pub status: MissionStatus,
    /// Progress fraction (0.0–1.0).
    pub progress: f32,
    /// Number of agents assigned to this mission.
    pub agent_count: u32,
}

/// Status of a mission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Active,
    Blocked,
    Idle,
    Completed,
}

/// Inspector data for a selected entity.
#[derive(Debug, Clone)]
pub struct EntityInspectorData {
    /// Domain ID of the entity.
    pub entity_id: String,
    /// Display name.
    pub display_name: String,
    /// Detail variant.
    pub details: InspectorDetails,
}

/// Variant-specific inspector details.
#[derive(Debug, Clone)]
pub enum InspectorDetails {
    Agent {
        state: String,
        tokens_used: u64,
        context_size: u64,
        pending_actions: Vec<String>,
    },
    Mission {
        progress: f32,
        assigned_agents: u32,
        description: String,
    },
    Artifact {
        artifact_type: String,
        size_bytes: u64,
        path: String,
    },
}

/// An entry in the scrolling event log.
#[derive(Debug, Clone)]
pub struct EventLogEntry {
    /// Timestamp string (ISO or relative).
    pub timestamp: String,
    /// Severity level.
    pub severity: EventSeverity,
    /// Event message body.
    pub message: String,
    /// Seconds since this entry was created (for fade-out).
    pub age_secs: f32,
}

/// Severity level for event log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
}

/// Command mode for the bottom console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    /// Direct command to a specific agent.
    Direct,
    /// Brainstorm / open-ended discussion.
    Brainstorm,
    /// Query / information retrieval.
    Query,
    /// Emergency halt.
    Halt,
}

/// Which view is shown in the center canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterCanvasMode {
    /// Passthrough — show the 3D world behind the HUD.
    WorldView,
    /// Multi-agent conversation log.
    ExpertsMeeting,
    /// Aggregated swarm particle/hex grid.
    SwarmMonitor,
    /// Artifact lineage DAG.
    ArtifactGraph,
    /// Debug forensics view.
    Forensics,
}

/// Kind of modal overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    /// Application settings.
    Settings,
    /// Agent tuning / parameter adjustment.
    AgentTuning,
    /// Artifact lineage detail view.
    ArtifactLineage,
}
