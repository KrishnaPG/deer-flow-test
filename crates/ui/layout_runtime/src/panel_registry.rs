use thiserror::Error;

use crate::panel_descriptor::PanelDescriptor;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PanelRegistry {
    panels: Vec<PanelDescriptor>,
}

impl PanelRegistry {
    pub fn panels(&self) -> &[PanelDescriptor] {
        &self.panels
    }

    pub fn panel(&self, panel_id: &str) -> Option<&PanelDescriptor> {
        self.panels
            .iter()
            .find(|panel| panel.panel_id() == panel_id)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("duplicate panel id")]
    DuplicatePanelId,
    #[error("unknown panel id")]
    UnknownPanelId,
}

pub fn register_panel(
    registry: &mut PanelRegistry,
    descriptor: PanelDescriptor,
) -> Result<(), RegistryError> {
    if registry
        .panels
        .iter()
        .any(|existing| existing.panel_id() == descriptor.panel_id())
    {
        return Err(RegistryError::DuplicatePanelId);
    }

    registry.panels.push(descriptor);
    Ok(())
}

pub fn remove_panel(
    registry: &mut PanelRegistry,
    panel_id: &str,
) -> Result<PanelDescriptor, RegistryError> {
    let index = registry
        .panels
        .iter()
        .position(|descriptor| descriptor.panel_id() == panel_id)
        .ok_or(RegistryError::UnknownPanelId)?;

    Ok(registry.panels.remove(index))
}
