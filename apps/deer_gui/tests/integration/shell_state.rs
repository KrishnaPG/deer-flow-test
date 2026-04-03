use bevy::prelude::*;

use deer_gui::shell::{CanonicalEntityRef, CanonicalRecordFamily, ShellPlugin, ShellState};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ShellPlugin);
    app
}

#[test]
fn t_shell_01_shell_state_defaults() {
    let mut app = test_app();
    app.update();

    let shell = app.world().resource::<ShellState>();
    assert!(shell.selection.primary.is_none());
    assert!(shell.selection.ordered.is_empty());
    assert!(shell.focus.target.is_none());
    assert!(shell.filters.global.is_empty());
    assert!(shell.pins.items.is_empty());
    assert!(shell.compare.items.is_empty());
    assert!(shell.intent_prefill.target.is_none());
}

#[test]
fn t_shell_02_shell_state_tracks_typed_selection_focus_filter_pin_compare_prefill() {
    let mut app = test_app();
    app.update();

    let reference = CanonicalEntityRef {
        family: CanonicalRecordFamily::Mission,
        canonical_id: "mission-1".to_string(),
        correlation_id: Some("corr-1".to_string()),
        lineage_id: None,
    };

    let mut shell = app.world_mut().resource_mut::<ShellState>();
    shell.selection.primary = Some(reference.clone());
    shell.selection.ordered.push(reference.clone());
    shell.focus.target = Some(reference.clone());
    shell.pins.items.push(reference.clone());
    shell.compare.items.push(reference.clone());
    shell.intent_prefill.target = Some(reference.clone());

    assert_eq!(shell.selection.ordered.len(), 1);
    assert_eq!(shell.pins.items.len(), 1);
    assert_eq!(shell.compare.items.len(), 1);
    assert_eq!(
        shell.intent_prefill.target.as_ref().unwrap().canonical_id,
        "mission-1"
    );
}

#[test]
fn t_shell_03_panel_participation_defaults_match_control_center_scaffolding() {
    use deer_gui::shell::{InteractionKind, PanelId, PanelParticipationRegistry, PanelRole};
    let registry = PanelParticipationRegistry::default();
    let inspector = registry.get(&PanelId::InspectorPanel);
    assert!(inspector.is_none());
}

#[test]
fn t_shell_04_brokers_apply_requests_and_update_sequences() {
    use bevy::prelude::MessageWriter;
    use deer_gui::shell::{
        CanonicalEntityRef, CanonicalRecordFamily, PanelId, ShellSelectionRequest, ShellState,
    };

    let mut app = test_app();
    app.world_mut().write_message(ShellSelectionRequest {
        next: CanonicalEntityRef {
            family: CanonicalRecordFamily::Mission,
            canonical_id: "mission-9".to_string(),
            correlation_id: None,
            lineage_id: None,
        },
        source: PanelId::MissionRailPanel,
    });

    app.update();

    let shell = app.world().resource::<ShellState>();
    assert_eq!(
        shell.selection.primary.as_ref().unwrap().canonical_id,
        "mission-9"
    );
}
