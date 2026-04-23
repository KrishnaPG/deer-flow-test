//! TET scene configuration — implements [`SceneConfig`] for the TET environment.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::info;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use crate::scene::generators::GeneratorRegistry;
use crate::scene::traits::SceneConfig;
use crate::theme::ThemeManager;

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
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        theme: Option<&ThemeManager>,
        _generators: &GeneratorRegistry,
    ) -> Entity {
        info!("TetSceneConfig::spawn_environment — spawning TET scene");
        super::setup::spawn_tet_environment(commands, meshes, materials, theme)
    }

    fn ambient_audio_track(&self) -> &'static str {
        "audio/tet_ambient.ogg"
    }
}
