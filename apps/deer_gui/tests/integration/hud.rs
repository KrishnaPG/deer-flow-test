//! Integration tests for the HUD subsystem.
//!
//! Covers HudState registration, default values, and field updates.
//! Does NOT instantiate the full HudPlugin (requires bevy_egui),
//! instead tests the state resource and data types directly.

use bevy::prelude::*;

use deer_gui::hud::{
    CenterCanvasMode, CommandMode, EntityInspectorData, EventLogEntry, EventSeverity, HudState,
    InspectorDetails, MissionStatus, MissionSummary,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<HudState>();
    app
}

// ---------------------------------------------------------------------------
// T-HUD-01  HudState resource is registered
// ---------------------------------------------------------------------------

#[test]
fn t_hud_01_hud_state_registered() {
    let mut app = test_app();
    app.update();

    assert!(
        app.world().get_resource::<HudState>().is_some(),
        "HudState resource must be present"
    );
}

// ---------------------------------------------------------------------------
// T-HUD-02  HudState defaults are correct
// ---------------------------------------------------------------------------

#[test]
fn t_hud_02_hud_state_defaults() {
    let mut app = test_app();
    app.update();

    let hud = app.world().resource::<HudState>();
    assert_eq!(hud.active_agents, 0);
    assert_eq!(hud.retrying_agents, 0);
    assert_eq!(hud.failed_agents, 0);
    assert_eq!(hud.tokens_per_sec, 0.0);
    assert_eq!(hud.cost_per_hour, 0.0);
    assert!(!hud.system_online);
    assert!(hud.missions.is_empty());
    assert!(hud.selected_entity.is_none());
    assert!(hud.event_log.is_empty());
    assert!(hud.command_input.is_empty());
    assert_eq!(hud.command_mode, CommandMode::Direct);
    assert_eq!(hud.center_mode, CenterCanvasMode::WorldView);
    assert!(hud.show_modal.is_none());
    assert!(hud.threads.is_empty());
    assert!(hud.selected_thread_id.is_none());
    assert!(hud.thread_cache.is_none());
    assert!(hud.models.is_empty());
    assert!(hud.selected_model.is_none());
    assert!(hud.streaming_thread_id.is_none());
}

// ---------------------------------------------------------------------------
// T-HUD-03  HudState metrics can be updated
// ---------------------------------------------------------------------------

#[test]
fn t_hud_03_hud_state_metrics_update() {
    let mut app = test_app();
    app.update();

    let mut hud = app.world_mut().resource_mut::<HudState>();
    hud.active_agents = 5;
    hud.tokens_per_sec = 120.5;
    hud.system_online = true;

    let hud = app.world().resource::<HudState>();
    assert_eq!(hud.active_agents, 5);
    assert_eq!(hud.tokens_per_sec, 120.5);
    assert!(hud.system_online);
}

// ---------------------------------------------------------------------------
// T-HUD-04  MissionSummary and status types
// ---------------------------------------------------------------------------

#[test]
fn t_hud_04_mission_summary() {
    let mission = MissionSummary {
        name: "Deploy v2".to_string(),
        status: MissionStatus::Active,
        progress: 0.75,
        agent_count: 3,
    };

    assert_eq!(mission.name, "Deploy v2");
    assert_eq!(mission.status, MissionStatus::Active);
    assert_eq!(mission.progress, 0.75);
    assert_eq!(mission.agent_count, 3);
}

// ---------------------------------------------------------------------------
// T-HUD-05  EventLogEntry and severity
// ---------------------------------------------------------------------------

#[test]
fn t_hud_05_event_log_entry() {
    let entry = EventLogEntry {
        timestamp: "12:34:56".to_string(),
        severity: EventSeverity::Warning,
        message: "High latency detected".to_string(),
        age_secs: 2.5,
    };

    assert_eq!(entry.severity, EventSeverity::Warning);
    assert_eq!(entry.age_secs, 2.5);
}

// ---------------------------------------------------------------------------
// T-HUD-06  EntityInspectorData and InspectorDetails
// ---------------------------------------------------------------------------

#[test]
fn t_hud_06_inspector_data() {
    let data = EntityInspectorData {
        entity_id: "agent-42".to_string(),
        display_name: "Agent 42".to_string(),
        details: InspectorDetails::Agent {
            state: "Working".to_string(),
            tokens_used: 1500,
            context_size: 4096,
            pending_actions: vec!["code_review".to_string()],
        },
    };

    assert_eq!(data.entity_id, "agent-42");
    match &data.details {
        InspectorDetails::Agent {
            state,
            tokens_used,
            context_size,
            pending_actions,
        } => {
            assert_eq!(state, "Working");
            assert_eq!(*tokens_used, 1500);
            assert_eq!(*context_size, 4096);
            assert_eq!(pending_actions.len(), 1);
        }
        _ => panic!("expected Agent variant"),
    }
}
