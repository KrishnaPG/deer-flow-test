use thiserror::Error;

use crate::{hosted_views::HostedViewRegistration, panel_descriptor::PanelDescriptor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedViewSlot {
    panel_id: String,
    hosted_view: HostedViewRegistration,
}

impl HostedViewSlot {
    pub fn new(panel_id: String, hosted_view: HostedViewRegistration) -> Self {
        Self {
            panel_id,
            hosted_view,
        }
    }

    pub fn panel_id(&self) -> &str {
        &self.panel_id
    }

    pub fn hosted_view(&self) -> HostedViewRegistration {
        self.hosted_view
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HostedViewHost {
    slots: Vec<HostedViewSlot>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ViewHostError {
    #[error("panel '{panel_id}' does not declare hosted view '{hosted_view_id}'")]
    HostedViewNotDeclared {
        panel_id: String,
        hosted_view_id: String,
    },
}

impl HostedViewHost {
    pub fn attach_panel(
        &mut self,
        descriptor: &PanelDescriptor,
        hosted_view: HostedViewRegistration,
    ) -> Result<HostedViewSlot, ViewHostError> {
        if !descriptor.declares_hosted_view(hosted_view.view_id()) {
            return Err(ViewHostError::HostedViewNotDeclared {
                panel_id: descriptor.panel_id().to_string(),
                hosted_view_id: hosted_view.view_id().to_string(),
            });
        }

        let slot = HostedViewSlot::new(descriptor.panel_id().to_string(), hosted_view);
        self.slots.push(slot.clone());
        Ok(slot)
    }

    pub fn slots(&self) -> &[HostedViewSlot] {
        &self.slots
    }
}
