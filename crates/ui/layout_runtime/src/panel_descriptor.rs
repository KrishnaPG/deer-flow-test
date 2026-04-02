use deer_ui_panel_shells::{PanelContract, PanelParticipation};
use thiserror::Error;

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
}
