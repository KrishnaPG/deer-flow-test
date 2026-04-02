pub mod hosted_views;
pub mod layout_model;
pub mod layout_persistence;
pub mod linked_brokers;
pub mod panel_descriptor;
pub mod panel_registry;
pub mod runtime;
pub mod view_host;

pub use hosted_views::{
    hosted_view_registration, HostedViewRegistration, ARTIFACT_SHELF, CHAT_THREAD, INSPECTOR,
};
pub use layout_model::{
    DockNode, LayoutModal, LayoutSnapshot, SplitAxis, CURRENT_LAYOUT_SNAPSHOT_VERSION,
};
pub use layout_persistence::{deserialize_layout, serialize_layout, LayoutPersistenceError};
pub use linked_brokers::{
    LinkedBrokerState, LinkedInteractionPropagation, LinkedInteractionUpdate,
};
pub use panel_descriptor::{PanelDescriptor, PanelDescriptorError};
pub use panel_registry::{register_panel, remove_panel, PanelRegistry, RegistryError};
pub use runtime::{LayoutRuntimeError, LayoutRuntimeState};
pub use view_host::{HostedViewHost, HostedViewSlot, ViewHostError};
