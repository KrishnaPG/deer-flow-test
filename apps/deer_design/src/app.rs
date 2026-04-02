use std::collections::BTreeMap;

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
use crate::panel_catalog::{panel_descriptors, ARTIFACT_PANEL, CHAT_PANEL, INSPECTOR_PANEL};
use crate::scenarios::live_meeting_runtime_scenario;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostedPanelsProof {
    pub chat: String,
    pub artifact: String,
    pub inspector: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayoutRuntimeProof {
    pub mode: String,
    pub panels: Vec<String>,
    pub saved_layout: String,
    pub selection_broker: String,
    #[serde(skip_serializing)]
    pub hosted_panels: HostedPanelsProof,
    #[serde(skip_serializing)]
    pub linked_panel_roles: BTreeMap<String, Vec<String>>,
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
    let runtime =
        LayoutRuntimeState::with_brokers(vec![LinkedBrokerState::new("selection", CHAT_PANEL)])
            .expect("broker should initialize");
    let propagation = runtime
        .propagate(LinkedInteractionUpdate::new(
            "selection",
            "artifact_1",
            ARTIFACT_PANEL,
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
    assert_eq!(hosted_panels.chat, CHAT_PANEL);
    assert_eq!(hosted_panels.artifact, ARTIFACT_PANEL);
    assert_eq!(hosted_panels.inspector, INSPECTOR_PANEL);
    assert_eq!(
        linked_shell.panel_roles.get(CHAT_PANEL),
        Some(&vec![
            LinkedShellPanelRole::Source,
            LinkedShellPanelRole::Broker
        ])
    );

    LayoutRuntimeProof {
        mode: runtime_read_model
            .active_mode
            .unwrap_or_else(|| LIVE_MEETING_MODE.into()),
        panels: restored_panels,
        saved_layout: "restored".into(),
        selection_broker: propagation.broker_panel_id,
        hosted_panels,
        linked_panel_roles: serialize_roles(&linked_shell),
    }
}

fn host_views(descriptors: &[PanelDescriptor]) -> HostedPanelsProof {
    let mut host = HostedViewHost::default();

    for descriptor in descriptors {
        for hosted_view_id in descriptor.participation().required_hosted_views() {
            let hosted_view = hosted_view_registration(&hosted_view_id)
                .expect("hosted view should be registered");
            host.attach_panel(descriptor, hosted_view)
                .expect("hosted view should attach");
        }
    }

    HostedPanelsProof {
        chat: host_slot_for(&host, CHAT_PANEL, "chat_thread_view"),
        artifact: host_slot_for(&host, ARTIFACT_PANEL, "artifact_shelf_view"),
        inspector: host_slot_for(&host, INSPECTOR_PANEL, "inspector_view"),
    }
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
        PanelRole::Source => LinkedShellPanelRole::Source,
        PanelRole::Broker => LinkedShellPanelRole::Broker,
        PanelRole::Sink => LinkedShellPanelRole::Sink,
        PanelRole::Mirror => LinkedShellPanelRole::Mirror,
    }
}

fn host_slot_for(host: &HostedViewHost, panel_id: &str, hosted_view_id: &str) -> String {
    host.slots()
        .iter()
        .find(|slot| slot.panel_id() == panel_id && slot.hosted_view().view_id() == hosted_view_id)
        .map(|slot| slot.panel_id().to_string())
        .expect("expected hosted view slot")
}

fn serialize_roles(shell: &LinkedShellState) -> BTreeMap<String, Vec<String>> {
    shell
        .panel_roles
        .iter()
        .map(|(panel_id, roles)| {
            (
                panel_id.clone(),
                roles
                    .iter()
                    .map(|role| role_name(role).to_string())
                    .collect(),
            )
        })
        .collect()
}

fn role_name(role: &LinkedShellPanelRole) -> &'static str {
    match role {
        LinkedShellPanelRole::Source => "source",
        LinkedShellPanelRole::Broker => "broker",
        LinkedShellPanelRole::Sink => "sink",
        LinkedShellPanelRole::Mirror => "mirror",
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
