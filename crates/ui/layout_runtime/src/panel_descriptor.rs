use deer_ui_panel_shells::{PanelContract, PanelParticipation};
use thiserror::Error;

const WORLD_PANEL_ID: &str = "world_viewport";
const MINIMAP_PANEL_ID: &str = "minimap_panel";
const WORLD_SCENE_VIEW_ID: &str = "world_scene_view";
const MINIMAP_VIEW_ID: &str = "minimap_view";
const VIEWPORT_JOIN_KEY: &str = "viewport_id";
const CAMERA_JOIN_KEY: &str = "camera_id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    contract: PanelContract,
    participation: PanelParticipation,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PanelDescriptorError {
    #[error("invalid panel contract for '{panel_id}': {reason}")]
    InvalidPanelContract {
        panel_id: String,
        reason: &'static str,
    },
}

impl PanelDescriptor {
    pub fn new(contract: PanelContract) -> Result<Self, PanelDescriptorError> {
        let participation = PanelParticipation::from_contract(&contract).map_err(|reason| {
            PanelDescriptorError::InvalidPanelContract {
                panel_id: contract.panel_id.clone(),
                reason,
            }
        })?;

        Ok(Self {
            contract,
            participation,
        })
    }

    pub fn panel_id(&self) -> &str {
        &self.contract.panel_id
    }

    pub fn contract(&self) -> &PanelContract {
        &self.contract
    }

    pub fn participation(&self) -> &PanelParticipation {
        &self.participation
    }

    pub fn declares_hosted_view(&self, hosted_view_id: &str) -> bool {
        self.contract
            .required_hosted_views
            .iter()
            .any(|declared| declared == hosted_view_id)
    }

    pub fn can_drive(&self, other: &PanelDescriptor) -> bool {
        let self_roles = self.participation.roles();
        let other_roles = other.participation.roles();
        let self_join_keys = self.participation.join_keys();
        let other_join_keys = other.participation.join_keys();

        let roles_are_compatible = self_roles.iter().any(|role| {
            matches!(
                role,
                deer_ui_panel_shells::PanelRole::Source | deer_ui_panel_shells::PanelRole::Broker
            )
        }) && other_roles.iter().any(|role| {
            matches!(
                role,
                deer_ui_panel_shells::PanelRole::Sink
                    | deer_ui_panel_shells::PanelRole::Mirror
                    | deer_ui_panel_shells::PanelRole::Broker
            )
        });

        roles_are_compatible
            && self_join_keys.iter().any(|join_key| {
                other_join_keys
                    .iter()
                    .any(|other_key| other_key == join_key)
            })
            && !self.contract.required_hosted_views.is_empty()
            && !other.contract.required_hosted_views.is_empty()
    }
}

pub fn world_panel_descriptor() -> Result<PanelDescriptor, PanelDescriptorError> {
    PanelDescriptor::new(PanelContract {
        panel_id: WORLD_PANEL_ID.into(),
        required_hosted_views: vec![WORLD_SCENE_VIEW_ID.into()],
        roles: vec![
            deer_ui_panel_shells::PanelRole::Source,
            deer_ui_panel_shells::PanelRole::Broker,
        ],
        join_keys: vec![VIEWPORT_JOIN_KEY.into(), CAMERA_JOIN_KEY.into()],
    })
}

pub fn minimap_panel_descriptor() -> Result<PanelDescriptor, PanelDescriptorError> {
    PanelDescriptor::new(PanelContract {
        panel_id: MINIMAP_PANEL_ID.into(),
        required_hosted_views: vec![MINIMAP_VIEW_ID.into()],
        roles: vec![
            deer_ui_panel_shells::PanelRole::Sink,
            deer_ui_panel_shells::PanelRole::Mirror,
            deer_ui_panel_shells::PanelRole::Broker,
        ],
        join_keys: vec![VIEWPORT_JOIN_KEY.into(), CAMERA_JOIN_KEY.into()],
    })
}
