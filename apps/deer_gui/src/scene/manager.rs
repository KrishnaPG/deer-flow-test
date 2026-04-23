//! Scene manager — resource for registering, activating, and
//! deactivating scene configurations at runtime.
//!
//! [`SceneManager`] owns a registry of [`SceneConfig`] trait objects
//! and tracks the currently active scene's root entity for clean
//! teardown via recursive despawn.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, trace, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Component, Entity, Mesh, Resource};

use super::audio_bridge::SceneAudioState;
use super::generators::GeneratorRegistry;
use super::traits::SceneConfig;
use crate::theme::ThemeManager;

// ---------------------------------------------------------------------------
// SceneRoot component
// ---------------------------------------------------------------------------

/// Component tag for the root entity of a scene hierarchy.
///
/// Every scene spawns a single root entity with this tag; all
/// scene-specific entities are children. Despawning the root
/// recursively removes the entire scene.
#[derive(Component, Debug, Default)]
pub struct SceneRoot;

// ---------------------------------------------------------------------------
// SceneManager resource
// ---------------------------------------------------------------------------

/// Central registry and lifecycle controller for scene configurations.
///
/// Owns a `Vec` of boxed [`SceneConfig`] trait objects and tracks
/// which scene is currently active (by index) along with its root
/// [`Entity`].
#[derive(Resource)]
pub struct SceneManager {
    configs: Vec<Box<dyn SceneConfig>>,
    active_index: Option<usize>,
    active_root: Option<Entity>,
}

impl SceneManager {
    /// Create an empty scene manager with no registered scenes.
    pub fn new() -> Self {
        trace!("SceneManager::new — created empty manager");
        Self {
            configs: Vec::new(),
            active_index: None,
            active_root: None,
        }
    }

    /// Register a scene configuration.
    ///
    /// Scenes are identified by their [`SceneConfig::name()`].
    pub fn register(&mut self, config: Box<dyn SceneConfig>) {
        info!("SceneManager::register — name='{}'", config.name());
        self.configs.push(config);
    }

    /// Activate a scene by name.
    ///
    /// Deactivates the current scene (if any), spawns the new scene's
    /// environment, and stores the root entity. Returns `true` on
    /// success, `false` if no scene with the given name is registered.
    ///
    /// When `theme` is provided, the scene receives theme colours for
    /// world materials. When `audio_state` is provided, the scene's
    /// ambient audio track is requested via [`SceneAudioState`].
    pub fn activate(
        &mut self,
        name: &str,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        theme: Option<&ThemeManager>,
        audio_state: Option<&mut SceneAudioState>,
        generators: &GeneratorRegistry,
    ) -> bool {
        debug!("SceneManager::activate — requested scene='{name}'");

        let Some(index) = self.find_index(name) else {
            warn!("SceneManager::activate — scene '{name}' not found");
            return false;
        };

        // Deactivate current scene first.
        self.deactivate(commands, None);

        // Spawn new scene environment.
        let root =
            self.configs[index].spawn_environment(commands, meshes, materials, theme, generators);
        self.configs[index].on_activate();

        self.active_index = Some(index);
        self.active_root = Some(root);

        // Wire ambient audio for the new scene.
        if let Some(state) = audio_state {
            let track = self.configs[index].ambient_audio_track();
            state.request_ambient(track);
        }

        info!(
            "SceneManager::activate — activated '{}', root={root:?}",
            self.configs[index].name(),
        );
        true
    }

    /// Deactivate the current scene, despawning all its entities.
    ///
    /// Calls [`SceneConfig::on_deactivate`] on the active config,
    /// then recursively despawns the root entity. No-ops if no
    /// scene is active.
    ///
    /// When `audio_state` is provided, stops the current ambient audio.
    pub fn deactivate(
        &mut self,
        commands: &mut Commands,
        audio_state: Option<&mut SceneAudioState>,
    ) {
        let Some(index) = self.active_index.take() else {
            trace!("SceneManager::deactivate — no active scene");
            return;
        };

        self.configs[index].on_deactivate();

        if let Some(root) = self.active_root.take() {
            debug!(
                "SceneManager::deactivate — despawning '{}', root={root:?}",
                self.configs[index].name(),
            );
            // Manually despawn all children first, then the root
            // Bevy 0.18 doesn't have despawn_recursive on EntityCommands
            commands.queue(move |world: &mut bevy::prelude::World| {
                despawn_recursive(world, root);
            });
        }

        // Stop ambient audio for the departing scene.
        if let Some(state) = audio_state {
            state.request_stop();
        }

        info!(
            "SceneManager::deactivate — deactivated '{}'",
            self.configs[index].name(),
        );
    }

    /// Reference to the currently active scene config, if any.
    pub fn current(&self) -> Option<&dyn SceneConfig> {
        let index = self.active_index?;
        let config = self.configs[index].as_ref();
        trace!("SceneManager::current — '{}'", config.name());
        Some(config)
    }

    /// Convenience: name of the currently active scene.
    pub fn current_name(&self) -> Option<&str> {
        let config = self.current()?;
        Some(config.name())
    }

    /// List the names of all registered scene configurations.
    pub fn available_scenes(&self) -> Vec<&str> {
        let names: Vec<&str> = self.configs.iter().map(|c| c.name()).collect();
        trace!("SceneManager::available_scenes — {:?}", names);
        names
    }

    /// The root entity of the currently active scene, if any.
    pub fn active_root(&self) -> Option<Entity> {
        trace!("SceneManager::active_root — {:?}", self.active_root);
        self.active_root
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Find the index of a config by name.
    fn find_index(&self, name: &str) -> Option<usize> {
        self.configs.iter().position(|c| c.name() == name)
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper: recursive despawn
// ---------------------------------------------------------------------------

/// Recursively despawn an entity and all its descendants.
fn despawn_recursive(world: &mut bevy::prelude::World, entity: bevy::prelude::Entity) {
    // Get children before despawning (to avoid borrow issues)
    let children: Vec<bevy::prelude::Entity> = world
        .get::<bevy::prelude::Children>(entity)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();

    // Recursively despawn children first
    for child in children {
        despawn_recursive(world, child);
    }

    // Despawn the entity itself
    world.despawn(entity);
}
