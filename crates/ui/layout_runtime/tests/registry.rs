use deer_ui_layout_runtime::{register_panel, PanelDescriptor, PanelRegistry, RegistryError};

#[test]
fn registry_rejects_duplicate_panel_ids_and_tracks_roles() {
    let mut registry = PanelRegistry::default();
    let descriptor = PanelDescriptor::new("chat_panel", vec!["chat_thread_view".into()]);

    register_panel(&mut registry, descriptor.clone()).unwrap();
    let duplicate = register_panel(&mut registry, descriptor);

    assert_eq!(duplicate, Err(RegistryError::DuplicatePanelId));
    assert_eq!(
        registry.panels(),
        &[PanelDescriptor::new(
            "chat_panel",
            vec!["chat_thread_view".into()]
        )]
    );
}
