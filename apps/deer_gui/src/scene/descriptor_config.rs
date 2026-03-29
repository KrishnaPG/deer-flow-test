//! Bridges [`SceneDescriptor`] to the [`SceneConfig`] trait.
//!
//! [`DescriptorSceneConfig`] reads a descriptor and implements
//! `SceneConfig` by delegating to the generator registry.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use super::descriptor::SceneDescriptor;
use super::primitives::spawn_root;
use super::traits::SceneConfig;
use crate::theme::ThemeManager;

// ---------------------------------------------------------------------------
// DescriptorSceneConfig
// ---------------------------------------------------------------------------

/// A [`SceneConfig`] implementation backed by a [`SceneDescriptor`].
///
/// The `audio_track` field is a leaked `&'static str` because
/// [`SceneConfig::ambient_audio_track`] returns `&'static str`.
/// Scene descriptors live for the program's lifetime, so this is safe.
pub struct DescriptorSceneConfig {
    descriptor: SceneDescriptor,
    audio_track: &'static str,
}

impl DescriptorSceneConfig {
    /// Create from a descriptor. Leaks the audio string for static lifetime.
    pub fn new(descriptor: SceneDescriptor) -> Self {
        let audio_track: &'static str =
            Box::leak(descriptor.ambient_audio.clone().into_boxed_str());
        debug!(
            "DescriptorSceneConfig::new — name='{}' audio='{audio_track}'",
            descriptor.name,
        );
        Self {
            descriptor,
            audio_track,
        }
    }
}

impl SceneConfig for DescriptorSceneConfig {
    fn name(&self) -> &str {
        &self.descriptor.name
    }

    fn ambient_audio_track(&self) -> &'static str {
        self.audio_track
    }

    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        theme: Option<&ThemeManager>,
    ) -> Entity {
        info!(
            "DescriptorSceneConfig::spawn_environment — scene='{}'",
            self.descriptor.name,
        );
        let root = spawn_root(commands);

        // TODO(PR-C): Iterate self.descriptor.generators, resolve each via
        // GeneratorRegistry, and call the factory. For now, just spawn root.
        for spec in &self.descriptor.generators {
            warn!(
                "Generator '{}' not yet wired (pending PR-C)",
                spec.generator,
            );
        }

        root
    }
}
