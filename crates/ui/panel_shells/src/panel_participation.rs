use crate::{panel_contract::PanelContract, panel_roles::PanelRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelParticipation {
    pub roles: Vec<PanelRole>,
    pub join_keys: Vec<String>,
}

impl PanelParticipation {
    pub fn from_contract(contract: &PanelContract) -> Result<Self, &'static str> {
        if contract.roles.is_empty() || contract.join_keys.is_empty() {
            return Err("panel participation requires declared roles and join keys");
        }

        Ok(Self {
            roles: contract.roles.clone(),
            join_keys: contract.join_keys.clone(),
        })
    }
}
