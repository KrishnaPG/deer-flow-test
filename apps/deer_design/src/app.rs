use std::collections::BTreeMap;

use deer_foundation_contracts::{AsIsHash, IdentityMeta, LineageMeta, RecordId};
use deer_foundation_domain::{AnyRecord, ArtifactBody, ArtifactRecord, TaskBody, TaskRecord};
use deer_runtime_read_models::{
    reduce_layout_runtime_state, reduce_linked_shell_state, LayoutRuntimeAction,
    LayoutRuntimeReadModel, LinkedShellAction, LinkedShellPanelRole, LinkedShellState,
};
use deer_runtime_world_projection::project_world_objects;
use deer_ui_layout_runtime::{
    hosted_view_registration, register_panel, HostedViewHost, LayoutRuntimeState,
    LinkedBrokerState, LinkedInteractionUpdate, PanelDescriptor, PanelRegistry,
};
use deer_ui_panel_shells::PanelRole;
use deer_view_scene3d::{emit_world_pick, render_actor_cloud, SceneHost, SpatialRay, Vec3};
use deer_view_telemetry_view::{
    link_cameras, render_minimap_view, render_telemetry_map_view, sync_camera, CameraLinkMode,
    ViewportCameraState,
};
use serde::Serialize;

use crate::layout_presets::{
    restore_live_meeting_layout, restore_spatial_analysis_layout, LIVE_MEETING_MODE,
    SPATIAL_ANALYSIS_MODE,
};
use crate::panel_catalog::{
    panel_descriptors, spatial_panel_ids, ARTIFACT_PANEL, CHAT_PANEL, INSPECTOR_PANEL,
};
use crate::scenarios::{
    live_meeting_runtime_scenario, LAYOUT_RUNTIME_PROOF_SCENARIO, SPATIAL_PROJECTION_PROOF_SCENARIO,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpatialSelection {
    pub source_record_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpatialProjectionProof {
    pub mode: String,
    pub world_objects: Vec<String>,
    pub selection: SpatialSelection,
    pub camera_sync: String,
    pub drill_down_target: String,
    pub scene_anchor_count: usize,
    pub telemetry_selected_marker: String,
    pub minimap_viewport_id: String,
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
        LAYOUT_RUNTIME_PROOF_SCENARIO,
        "chat + artifact + inspector runtime with save/restore"
    );
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

pub fn run_spatial_projection_proof() -> SpatialProjectionProof {
    let projection = project_world_objects(&spatial_records());
    let layout = restore_spatial_analysis_layout();
    let descriptors = panel_descriptors();
    let (world_panel, minimap_panel) = spatial_panel_ids();
    let mut registry = PanelRegistry::default();

    for descriptor in &descriptors {
        register_panel(&mut registry, descriptor.clone()).expect("panel should register");
    }

    let restored_panels = flatten_panels(&layout.dock);
    let linked_shell = restore_linked_shell(&registry, &restored_panels);
    let actor_cloud = render_actor_cloud(&projection);
    let world_host = SceneHost::from_projection(&world_panel, &projection);
    let pick = emit_world_pick(
        world_host.spatial_index(),
        SpatialRay::new(Vec3::new(2.5, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0)),
    )
    .expect("artifact world object should be pickable");
    let scene_anchor_count = actor_cloud.anchors.len();
    let telemetry = render_telemetry_map_view(&projection, Some(&pick.selection_id));
    let minimap = render_minimap_view(&projection, &minimap_panel);
    let camera_sync = sync_camera(
        &link_cameras(&world_panel, &minimap_panel, CameraLinkMode::Bidirectional),
        &ViewportCameraState::new(&world_panel, [2.5, 0.0], 3),
        &ViewportCameraState::new(&minimap_panel, [0.0, 0.0], 1),
    )
    .expect("camera link should match declared world/minimap pair");

    assert_eq!(
        SPATIAL_PROJECTION_PROOF_SCENARIO,
        "project canonical task and artifact state into world selection and drill-down"
    );
    assert_eq!(layout.mode, SPATIAL_ANALYSIS_MODE);
    assert!(restored_panels.iter().any(|panel| panel == &world_panel));
    assert!(restored_panels.iter().any(|panel| panel == &minimap_panel));
    assert_eq!(
        linked_shell.panel_roles.get(&minimap_panel),
        Some(&vec![
            LinkedShellPanelRole::Broker,
            LinkedShellPanelRole::Sink,
            LinkedShellPanelRole::Mirror,
        ])
    );
    assert_eq!(pick.selection_id, "artifact_1");
    assert_eq!(pick.focus_target, "artifact_detail");
    assert_eq!(scene_anchor_count, 2);
    assert_eq!(telemetry.selected_marker_id.as_deref(), Some("artifact_1"));
    assert_eq!(minimap.viewport_id, minimap_panel);

    SpatialProjectionProof {
        mode: layout.mode,
        world_objects: projection
            .objects
            .iter()
            .map(|object| object.kind.to_string())
            .collect(),
        selection: SpatialSelection {
            source_record_id: pick.selection_id,
        },
        camera_sync: match camera_sync.mode {
            CameraLinkMode::Bidirectional => "bidirectional".into(),
            CameraLinkMode::OneWay => "one_way".into(),
        },
        drill_down_target: pick.focus_target.to_string(),
        scene_anchor_count,
        telemetry_selected_marker: telemetry
            .selected_marker_id
            .expect("telemetry selection should be preserved"),
        minimap_viewport_id: minimap.viewport_id,
    }
}

fn spatial_records() -> Vec<AnyRecord> {
    vec![
        AnyRecord::Task(TaskRecord::new(
            RecordId::from_static("task_1"),
            IdentityMeta::hash_anchored(RecordId::from_static("task_1"), None, None, None),
            LineageMeta::root(),
            TaskBody {
                label: "Survey ridge".into(),
                status: "running".into(),
            },
        )),
        AnyRecord::Artifact(ArtifactRecord::new(
            RecordId::from_static("artifact_1"),
            IdentityMeta::hash_anchored(
                RecordId::from_static("artifact_1"),
                Some(AsIsHash::from_static("sha256:artifact-1")),
                None,
                None,
            ),
            LineageMeta::root(),
            ArtifactBody {
                label: "terrain.md".into(),
                media_type: "text/markdown".into(),
            },
        )),
    ]
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
