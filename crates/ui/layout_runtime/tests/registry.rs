use deer_ui_layout_runtime::{
    hosted_view_registration, register_panel, remove_panel, HostedViewHost, PanelDescriptor,
    PanelDescriptorError, PanelRegistry, RegistryError, ViewHostError, ARTIFACT_SHELF, CHAT_THREAD,
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

#[test]
fn registry_removes_panels_and_reports_missing_removals() {
    let mut registry = PanelRegistry::default();
    let descriptor = PanelDescriptor::new(PanelContract {
        panel_id: "chat_panel".into(),
        required_hosted_views: vec!["chat_thread_view".into()],
        roles: vec![PanelRole::Source],
        join_keys: vec!["thread_id".into()],
    })
    .unwrap();

    register_panel(&mut registry, descriptor).unwrap();

    let removed = remove_panel(&mut registry, "chat_panel").unwrap();
    let missing = remove_panel(&mut registry, "chat_panel");

    assert_eq!(removed.panel_id(), "chat_panel");
    assert!(registry.panels().is_empty());
    assert_eq!(missing, Err(RegistryError::UnknownPanelId));
}

#[test]
fn registry_checks_contract_compatibility_through_roles_join_keys_and_views() {
    let source = PanelDescriptor::new(PanelContract {
        panel_id: "chat_panel".into(),
        required_hosted_views: vec!["chat_thread_view".into()],
        roles: vec![PanelRole::Source],
        join_keys: vec!["thread_id".into()],
    })
    .unwrap();
    let compatible_sink = PanelDescriptor::new(PanelContract {
        panel_id: "inspector_panel".into(),
        required_hosted_views: vec!["inspector_view".into()],
        roles: vec![PanelRole::Sink],
        join_keys: vec!["thread_id".into()],
    })
    .unwrap();
    let incompatible_sink = PanelDescriptor::new(PanelContract {
        panel_id: "artifact_panel".into(),
        required_hosted_views: vec!["artifact_shelf_view".into()],
        roles: vec![PanelRole::Sink],
        join_keys: vec!["artifact_id".into()],
    })
    .unwrap();

    assert!(source.can_drive(&compatible_sink));
    assert!(!compatible_sink.can_drive(&source));
    assert!(!source.can_drive(&incompatible_sink));
}

#[test]
fn hosted_view_bridge_uses_canonical_registration_helpers() {
    let chat = hosted_view_registration("chat_thread_view");
    let missing = hosted_view_registration("custom_view");

    assert_eq!(chat.unwrap().view_id(), "chat_thread_view");
    assert_eq!(missing, None);
}

#[test]
fn hosted_view_bridge_registers_spatial_view_ids() {
    let world = hosted_view_registration("world_scene_view");
    let minimap = hosted_view_registration("minimap_view");

    assert_eq!(world.unwrap().view_id(), "world_scene_view");
    assert_eq!(minimap.unwrap().view_id(), "minimap_view");
}

#[test]
fn spatial_panel_descriptors_declare_explicit_world_and_minimap_linkage() {
    let world = deer_ui_layout_runtime::panel_descriptor::world_panel_descriptor().unwrap();
    let minimap = deer_ui_layout_runtime::panel_descriptor::minimap_panel_descriptor().unwrap();
    let mut host = HostedViewHost::default();

    let world_slot = host
        .attach_panel(&world, deer_ui_layout_runtime::hosted_views::WORLD_SCENE)
        .unwrap();
    let minimap_slot = host
        .attach_panel(&minimap, deer_ui_layout_runtime::hosted_views::MINIMAP)
        .unwrap();

    assert_eq!(world.panel_id(), "world_viewport");
    assert_eq!(
        world.contract().required_hosted_views,
        vec!["world_scene_view".to_string()]
    );
    assert_eq!(
        world.participation().roles(),
        vec![PanelRole::Source, PanelRole::Broker]
    );
    assert_eq!(
        world.participation().join_keys(),
        vec!["viewport_id".to_string(), "camera_id".to_string()]
    );
    assert_eq!(minimap.panel_id(), "minimap_panel");
    assert_eq!(
        minimap.contract().required_hosted_views,
        vec!["minimap_view".to_string()]
    );
    assert_eq!(
        minimap.participation().roles(),
        vec![PanelRole::Sink, PanelRole::Mirror, PanelRole::Broker]
    );
    assert_eq!(
        minimap.participation().join_keys(),
        vec!["viewport_id".to_string(), "camera_id".to_string()]
    );
    assert!(world.can_drive(&minimap));
    assert_eq!(world_slot.hosted_view().view_id(), "world_scene_view");
    assert_eq!(minimap_slot.hosted_view().view_id(), "minimap_view");
}
