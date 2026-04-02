use serde::{Deserialize, Serialize};

use crate::panel_roles::PanelRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelContract {
    pub panel_id: String,
    pub required_hosted_views: Vec<String>,
    pub roles: Vec<PanelRole>,
    pub join_keys: Vec<String>,
}
