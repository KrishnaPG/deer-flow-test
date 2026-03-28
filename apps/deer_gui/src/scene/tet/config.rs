//! TET scene configuration — implements [`SceneConfig`] for the TET environment.

use bevy::asset::Assets;
use bevy::ecs::system::{Commands, ResMut};
use bevy::log::info;
use bevy::pbr::StandardMaterial;
use bevy::prelude::Mesh;

use crate::scene::traits::SceneConfig;

// ---------------------------------------------------------------------------
// TetSceneConfig
// ---------------------------------------------------------------------------

/// Configuration for the TET (central tetrahedral structure) scene.
///
/// This is the primary scene showing the monolith, starfield,
/// data trails, and agent particle effects.
#[derive(Debug, Clone, Default)]
pub struct TetSceneConfig;

impl SceneConfig for TetSceneConfig {
    fn name(&self) -> &str {
        "TET"
    }

    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        info!("TetSceneConfig::spawn_environment — spawning TET scene");
        super::setup::spawn_tet_environment(commands, meshes, materials);
    }

    fn ambient_audio_track(&self) -> &str {
        "audio/tet_ambient.ogg"
    }
}
