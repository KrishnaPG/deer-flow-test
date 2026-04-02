use deer_ui_layout_runtime::{
    register_panel, HostedViewHost, PanelDescriptor, PanelDescriptorError, PanelRegistry,
    RegistryError, ViewHostError, ARTIFACT_SHELF, CHAT_THREAD,
};
use deer_ui_panel_shells::{PanelContract, PanelRole};

#[test]
fn registry_rejects_duplicate_panel_ids_and_tracks_panel_contracts() {
    let mut registry = PanelRegistry::default();
    let contract = PanelContract {
        panel_id: "chat_panel".into(),
        required_hosted_views: vec!["chat_thread_view".into()],
        roles: vec![PanelRole::Source, PanelRole::Broker],
        join_keys: vec!["thread_id".into()],
    };
    let descriptor = PanelDescriptor::new(contract.clone()).unwrap();

    register_panel(&mut registry, descriptor.clone()).unwrap();
    let duplicate = register_panel(&mut registry, descriptor);

    assert_eq!(duplicate, Err(RegistryError::DuplicatePanelId));

    let registered = registry.panel("chat_panel").unwrap();
    assert_eq!(registered.contract(), &contract);
    assert_eq!(
        registered.participation().roles(),
        vec![PanelRole::Source, PanelRole::Broker]
    );
    assert_eq!(
        registered.participation().join_keys(),
        vec!["thread_id".to_string()]
    );
}

#[test]
fn panel_descriptor_and_host_bridge_enforce_contract_hosted_views() {
    let contract = PanelContract {
        panel_id: "chat_panel".into(),
        required_hosted_views: vec!["chat_thread_view".into()],
        roles: vec![PanelRole::Source],
        join_keys: vec!["thread_id".into()],
    };
    let descriptor = PanelDescriptor::new(contract).unwrap();
    let mut host = HostedViewHost::default();

    let slot = host.attach_panel(&descriptor, CHAT_THREAD).unwrap();
    let wrong_view = host.attach_panel(&descriptor, ARTIFACT_SHELF);

    assert_eq!(slot.panel_id(), "chat_panel");
    assert_eq!(slot.hosted_view().view_id(), "chat_thread_view");
    assert_eq!(host.slots(), &[slot.clone()]);
    assert_eq!(
        wrong_view,
        Err(ViewHostError::HostedViewNotDeclared {
            panel_id: "chat_panel".into(),
            hosted_view_id: "artifact_shelf_view".into(),
        })
    );
}

#[test]
fn panel_descriptor_rejects_invalid_panel_participation() {
    let invalid_contract = PanelContract {
        panel_id: "chat_panel".into(),
        required_hosted_views: Vec::new(),
        roles: vec![PanelRole::Source],
        join_keys: vec!["thread_id".into()],
    };

    let error = PanelDescriptor::new(invalid_contract).unwrap_err();

    assert_eq!(
        error,
        PanelDescriptorError::InvalidPanelContract {
            panel_id: "chat_panel".into(),
            reason: "panel participation requires hosted views, declared roles, and join keys",
        }
    );
}
