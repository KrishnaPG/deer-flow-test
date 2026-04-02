use deer_runtime_read_models::{
    reduce_layout_runtime_state, reduce_linked_shell_state, LayoutRuntimeAction,
    LayoutRuntimeReadModel, LinkedShellAction, LinkedShellPanelRole, LinkedShellState,
};
use deer_ui_layout_runtime::{
    hosted_view_registration, register_panel, HostedViewHost, LayoutRuntimeState,
    LinkedBrokerState, LinkedInteractionUpdate, PanelDescriptor, PanelRegistry,
};
use deer_ui_panel_shells::PanelRole;
use serde::Serialize;

use crate::layout_presets::{restore_live_meeting_layout, LIVE_MEETING_MODE};
use crate::panel_catalog::panel_descriptors;
use crate::scenarios::live_meeting_runtime_scenario;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayoutRuntimeProof {
    pub mode: String,
    pub panels: Vec<String>,
    pub saved_layout: String,
    pub selection_broker: String,
}

pub fn run_layout_runtime_proof() -> LayoutRuntimeProof {
    let scenario = live_meeting_runtime_scenario();
    let descriptors = panel_descriptors();
    let mut registry = PanelRegistry::default();

    for descriptor in &descriptors {
        register_panel(&mut registry, descriptor.clone()).expect("panel should register");
    }

    let hosted_panels = host_views(&descriptors);
    let restored_layout = restore_live_meeting_layout();
    let restored_panels = flatten_panels(&restored_layout.dock);
    let runtime = LayoutRuntimeState::with_brokers(vec![LinkedBrokerState::new(
        "selection",
        &hosted_panels[0],
    )])
    .expect("broker should initialize");
    let propagation = runtime
        .propagate(LinkedInteractionUpdate::new(
            "selection",
            "artifact_1",
            &hosted_panels[1],
        ))
        .expect("selection should broker");
    let runtime_read_model = reduce_layout_runtime_state(
        LayoutRuntimeReadModel::default(),
        LayoutRuntimeAction::PresetLoaded {
            mode: restored_layout.mode.clone(),
        },
    );
    let linked_shell = restore_linked_shell(&registry, &restored_panels);

    assert_eq!(scenario.transcript.row_count, 1);
    assert_eq!(scenario.artifacts.item_count, 1);
    assert_eq!(scenario.temporal.stream_state.as_deref(), Some("live"));
    assert_eq!(
        runtime_read_model.active_mode.as_deref(),
        Some(LIVE_MEETING_MODE)
    );
    assert!(linked_shell.panel_roles.contains_key("chat_panel"));

    LayoutRuntimeProof {
        mode: runtime_read_model
            .active_mode
            .unwrap_or_else(|| LIVE_MEETING_MODE.into()),
        panels: restored_panels,
        saved_layout: "restored".into(),
        selection_broker: propagation.broker_panel_id,
    }
}

fn host_views(descriptors: &[PanelDescriptor]) -> Vec<String> {
    let mut host = HostedViewHost::default();

    for descriptor in descriptors {
        let hosted_view = hosted_view_registration(
            descriptor
                .participation()
                .required_hosted_views()
                .first()
                .expect("panel should declare hosted view"),
        )
        .expect("hosted view should be registered");
        host.attach_panel(descriptor, hosted_view)
            .expect("hosted view should attach");
    }

    host.slots()
        .iter()
        .map(|slot| slot.panel_id().to_string())
        .collect()
}

fn restore_linked_shell(registry: &PanelRegistry, restored_panels: &[String]) -> LinkedShellState {
    let mut shell = LinkedShellState::default();

    for descriptor in registry.panels() {
        shell = reduce_linked_shell_state(
            shell,
            LinkedShellAction::PanelParticipationDeclared {
                panel_id: descriptor.panel_id().to_string(),
                roles: descriptor
                    .participation()
                    .roles()
                    .into_iter()
                    .map(map_panel_role)
                    .collect(),
            },
        );
    }

    reduce_linked_shell_state(
        shell,
        LinkedShellAction::LayoutPanelsRestored {
            panel_ids: restored_panels.to_vec(),
        },
    )
}

fn map_panel_role(role: PanelRole) -> LinkedShellPanelRole {
    match role {
        PanelRole::Source | PanelRole::Broker => LinkedShellPanelRole::Source,
        PanelRole::Sink => LinkedShellPanelRole::Sink,
        PanelRole::Mirror => LinkedShellPanelRole::Mirror,
    }
}

fn flatten_panels(dock: &deer_ui_layout_runtime::DockNode) -> Vec<String> {
    match dock {
        deer_ui_layout_runtime::DockNode::Tabs { panels } => panels.clone(),
        deer_ui_layout_runtime::DockNode::Split { first, second, .. } => {
            let mut panels = flatten_panels(first);
            panels.extend(flatten_panels(second));
            panels
        }
    }
}
