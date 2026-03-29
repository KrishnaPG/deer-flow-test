//! Integration tests for HUD Transclusion (PR I).
//!
//! Tests the HudFragmentRegistry and HudFragment types without requiring
//! bevy_egui or the full HUD plugin.

use bevy::prelude::*;

use deer_gui::hud::{HudFragment, HudFragmentRegistry};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<HudFragmentRegistry>();
    app
}

// ---------------------------------------------------------------------------
// T-TRANS-01  Registry default is empty
// ---------------------------------------------------------------------------

#[test]
fn t_trans_01_registry_default_empty() {
    let mut app = test_app();
    app.update();

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert!(registry.is_empty(), "Registry should be empty by default");
    assert_eq!(registry.len(), 0, "Registry length should be 0");
    assert!(
        registry.fragments().is_empty(),
        "Fragments slice should be empty"
    );
}

// ---------------------------------------------------------------------------
// T-TRANS-02  Registering a fragment works
// ---------------------------------------------------------------------------

#[test]
fn t_trans_02_register_fragment() {
    let mut app = test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.register(HudFragment::new("test_provider", "Test Fragment", |_ui| {
            // Render callback
        }));
    }

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert_eq!(registry.len(), 1, "Registry should have 1 fragment");
    assert!(!registry.is_empty(), "Registry should not be empty");

    let fragments = registry.fragments();
    assert_eq!(fragments[0].provider, "test_provider");
    assert_eq!(fragments[0].title, "Test Fragment");
}

// ---------------------------------------------------------------------------
// T-TRANS-03  Unregistering removes fragments
// ---------------------------------------------------------------------------

#[test]
fn t_trans_03_unregister_removes() {
    let mut app = test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.register(HudFragment::new("provider_a", "Fragment A", |_ui| {}));
        registry.register(HudFragment::new("provider_b", "Fragment B", |_ui| {}));
        registry.register(HudFragment::new("provider_c", "Fragment C", |_ui| {}));
    }

    {
        let registry = app.world().resource::<HudFragmentRegistry>();
        assert_eq!(registry.len(), 3, "Should have 3 fragments initially");
    }

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.unregister("provider_b");
    }

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert_eq!(
        registry.len(),
        2,
        "Should have 2 fragments after unregister"
    );

    let providers: Vec<_> = registry.fragments().iter().map(|f| f.provider).collect();
    assert!(providers.contains(&"provider_a"));
    assert!(providers.contains(&"provider_c"));
    assert!(!providers.contains(&"provider_b"));
}

// ---------------------------------------------------------------------------
// T-TRANS-04  Multiple providers can register fragments
// ---------------------------------------------------------------------------

#[test]
fn t_trans_04_multiple_providers() {
    let mut app = test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();

        // Register fragments from different providers
        registry.register(HudFragment::new("bridge", "Bridge Status", |_ui| {}));
        registry.register(HudFragment::new("diagnostics", "System Metrics", |_ui| {}));
        registry.register(HudFragment::new("world", "Entity Count", |_ui| {}));
    }

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert_eq!(
        registry.len(),
        3,
        "Should have 3 fragments from 3 providers"
    );

    let fragments = registry.fragments();
    assert_eq!(fragments[0].provider, "bridge");
    assert_eq!(fragments[0].title, "Bridge Status");
    assert_eq!(fragments[1].provider, "diagnostics");
    assert_eq!(fragments[1].title, "System Metrics");
    assert_eq!(fragments[2].provider, "world");
    assert_eq!(fragments[2].title, "Entity Count");
}

// ---------------------------------------------------------------------------
// T-TRANS-05  Re-registering replaces existing fragment
// ---------------------------------------------------------------------------

#[test]
fn t_trans_05_reregister_replaces() {
    let mut app = test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.register(HudFragment::new("test", "First Title", |_ui| {}));
    }

    {
        let registry = app.world().resource::<HudFragmentRegistry>();
        assert_eq!(registry.fragments()[0].title, "First Title");
    }

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.register(HudFragment::new("test", "Updated Title", |_ui| {}));
    }

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert_eq!(registry.len(), 1, "Should still have only 1 fragment");
    assert_eq!(registry.fragments()[0].title, "Updated Title");
}

// ---------------------------------------------------------------------------
// T-TRANS-06  Clear removes all fragments
// ---------------------------------------------------------------------------

#[test]
fn t_trans_06_clear_removes_all() {
    let mut app = test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.register(HudFragment::new("a", "A", |_ui| {}));
        registry.register(HudFragment::new("b", "B", |_ui| {}));
        registry.register(HudFragment::new("c", "C", |_ui| {}));
    }

    {
        let registry = app.world().resource::<HudFragmentRegistry>();
        assert_eq!(registry.len(), 3);
    }

    {
        let mut registry = app.world_mut().resource_mut::<HudFragmentRegistry>();
        registry.clear();
    }

    let registry = app.world().resource::<HudFragmentRegistry>();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}
