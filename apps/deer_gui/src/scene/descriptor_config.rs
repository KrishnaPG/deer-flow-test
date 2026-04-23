//! Bridges [`SceneDescriptor`] to the [`SceneConfig`] trait.
//!
//! [`DescriptorSceneConfig`] reads a descriptor and implements
//! `SceneConfig` by delegating to the generator registry.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use bevy::log::{debug, info};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{AssetServer, ChildOf, Entity, Handle, Mesh, Scene, SceneRoot};

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

    /// Create from a scene descriptor file (e.g., "medieval_open" → scenes/medieval_open.scene.ron).
    pub fn from_file(scene_name: &str) -> std::io::Result<Self> {
        use super::loader::LoaderError;

        let scenes_dir = std::path::PathBuf::from("apps/deer_gui/assets/scenes");
        let scene_file = scenes_dir.join(format!("{}.scene.ron", scene_name));

        let content = std::fs::read_to_string(&scene_file)?;
        let descriptor: SceneDescriptor = ron::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Self::new(descriptor))
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
        _meshes: &mut Assets<Mesh>,
        _materials: &mut Assets<StandardMaterial>,
        _theme: Option<&ThemeManager>,
    ) -> Entity {
        info!(
            "DescriptorSceneConfig::spawn_environment — scene='{}'",
            self.descriptor.name,
        );
        let root = spawn_root(commands);

        // Load base glTF scene if specified
        if let Some(gltf_path) = &self.descriptor.gltf_scene {
            debug!("DescriptorSceneConfig: loading base glTF scene '{gltf_path}'");
            let path = gltf_path.clone();
            commands.queue(move |world: &mut World| {
                let asset_server = world.resource::<AssetServer>();
                let scene_handle: Handle<Scene> = asset_server.load(format!("{path}#Scene0"));
                world
                    .commands()
                    .spawn((ChildOf(root), SceneRoot(scene_handle)));
                debug!("DescriptorSceneConfig: queued base glTF '{path}' under root={root:?}");
            });
        }

        // Generators are resolved and invoked by the plugin system after
        // scene activation. Here we just log the generators for verification.
        for spec in &self.descriptor.generators {
            info!(
                "DescriptorSceneConfig: generator='{}' (will be resolved by plugin)",
                spec.generator,
            );
        }

        root
    }
}
