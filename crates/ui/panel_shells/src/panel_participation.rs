use crate::{panel_contract::PanelContract, panel_roles::PanelRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelParticipation {
    required_hosted_views: Vec<String>,
    roles: Vec<PanelRole>,
    join_keys: Vec<String>,
}

impl PanelParticipation {
    pub fn from_contract(contract: &PanelContract) -> Result<Self, &'static str> {
        if contract.required_hosted_views.is_empty()
            || contract.roles.is_empty()
            || contract.join_keys.is_empty()
        {
            return Err("panel participation requires hosted views, declared roles, and join keys");
        }

        Ok(Self {
            required_hosted_views: contract.required_hosted_views.clone(),
            roles: contract.roles.clone(),
            join_keys: contract.join_keys.clone(),
        })
    }

    pub fn required_hosted_views(&self) -> Vec<String> {
        self.required_hosted_views.clone()
    }

    pub fn roles(&self) -> Vec<PanelRole> {
        self.roles.clone()
    }

    pub fn join_keys(&self) -> Vec<String> {
        self.join_keys.clone()
    }
}
