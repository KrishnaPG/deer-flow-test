//! HUD Transclusion — stateless data-only fragments for external systems.
//!
//! Any subsystem can push content fragments into the Center Canvas without
//! importing HUD internals. Fragments are registered via [`HudFragmentRegistry`]
//! and rendered after built-in mode content.
//!
//! # Example
//! ```rust,ignore
//! // Register a fragment from any system
//! fn my_system(mut registry: ResMut<HudFragmentRegistry>) {
//!     registry.register(HudFragment::new(
//!         "my_module",
//!         "My Panel",
//!         |ui| {
//!             ui.label("Custom content");
//!         },
//!     ));
//! }
//! ```

use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::egui;

// ---------------------------------------------------------------------------
// HudFragment — individual content fragment
// ---------------------------------------------------------------------------

/// A stateless content fragment that can be rendered in the Center Canvas.
///
/// Fragments are registered by external systems to inject UI content without
/// coupling to HUD internals. Each fragment has a provider identifier,
/// display title, and a render callback.
#[derive(Clone)]
pub struct HudFragment {
    /// Static identifier for the provider subsystem (e.g., "bridge", "diagnostics").
    pub provider: &'static str,

    /// Display title shown in the fragment's collapsible header.
    pub title: String,

    /// Render callback — draws the fragment's content into the provided UI.
    pub render: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>,
}

impl HudFragment {
    /// Creates a new HUD fragment with the given provider, title, and render function.
    ///
    /// # Arguments
    /// * `provider` - Static string identifier for the providing subsystem
    /// * `title` - Display title for the fragment
    /// * `render` - Callback that renders the fragment's content
    ///
    /// # Example
    /// ```rust,ignore
    /// let fragment = HudFragment::new(
    ///     "diagnostics",
    ///     "System Metrics",
    ///     |ui| { ui.label("CPU: 45%"); },
    /// );
    /// ```
    pub fn new<F>(provider: &'static str, title: impl Into<String>, render: F) -> Self
    where
        F: Fn(&mut egui::Ui) + Send + Sync + 'static,
    {
        Self {
            provider,
            title: title.into(),
            render: Arc::new(render),
        }
    }
}

// ---------------------------------------------------------------------------
// HudFragmentRegistry — central registry for fragments
// ---------------------------------------------------------------------------

/// Resource that holds all registered HUD fragments.
///
/// External systems register fragments here, and the center canvas system
/// reads from this resource to render them. Fragments are rendered in
/// registration order after built-in mode content.
#[derive(Resource, Default)]
pub struct HudFragmentRegistry {
    fragments: Vec<HudFragment>,
}

impl HudFragmentRegistry {
    /// Registers a new fragment with the registry.
    ///
    /// If a fragment with the same provider already exists, it will be replaced.
    /// This allows subsystems to update their fragments by re-registering.
    ///
    /// # Arguments
    /// * `fragment` - The fragment to register
    ///
    /// # Example
    /// ```rust,ignore
    /// registry.register(HudFragment::new(
    ///     "my_module",
    ///     "Panel Title",
    ///     |ui| { ui.label("Content"); },
    /// ));
    /// ```
    pub fn register(&mut self, fragment: HudFragment) {
        // Remove any existing fragment from the same provider
        self.fragments.retain(|f| f.provider != fragment.provider);
        self.fragments.push(fragment);
    }

    /// Unregisters all fragments from a given provider.
    ///
    /// # Arguments
    /// * `provider` - The provider identifier to remove
    ///
    /// # Example
    /// ```rust,ignore
    /// registry.unregister("my_module");
    /// ```
    pub fn unregister(&mut self, provider: &'static str) {
        self.fragments.retain(|f| f.provider != provider);
    }

    /// Returns a slice of all registered fragments.
    ///
    /// Used by the center canvas system to iterate and render fragments.
    pub fn fragments(&self) -> &[HudFragment] {
        &self.fragments
    }

    /// Returns the number of registered fragments.
    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    /// Returns true if no fragments are registered.
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Clears all registered fragments.
    pub fn clear(&mut self) {
        self.fragments.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_new() {
        let fragment = HudFragment::new("test", "Test Title", |_ui| {});
        assert_eq!(fragment.provider, "test");
        assert_eq!(fragment.title, "Test Title");
    }

    #[test]
    fn test_registry_default_empty() {
        let registry = HudFragmentRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register_fragment() {
        let mut registry = HudFragmentRegistry::default();
        registry.register(HudFragment::new("test", "Test", |_ui| {}));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let fragments = registry.fragments();
        assert_eq!(fragments[0].provider, "test");
        assert_eq!(fragments[0].title, "Test");
    }

    #[test]
    fn test_registry_unregister_removes() {
        let mut registry = HudFragmentRegistry::default();
        registry.register(HudFragment::new("test1", "Test 1", |_ui| {}));
        registry.register(HudFragment::new("test2", "Test 2", |_ui| {}));
        assert_eq!(registry.len(), 2);

        registry.unregister("test1");
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.fragments()[0].provider, "test2");
    }

    #[test]
    fn test_registry_replace_same_provider() {
        let mut registry = HudFragmentRegistry::default();
        registry.register(HudFragment::new("test", "First", |_ui| {}));
        registry.register(HudFragment::new("test", "Second", |_ui| {}));

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.fragments()[0].title, "Second");
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = HudFragmentRegistry::default();
        registry.register(HudFragment::new("test1", "Test 1", |_ui| {}));
        registry.register(HudFragment::new("test2", "Test 2", |_ui| {}));
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
