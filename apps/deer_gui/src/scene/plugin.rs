//! [`ScenePlugin`] — registers scene resources, startup, and per-frame systems.

use bevy::asset::Assets;
use bevy::log::{debug, info, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{App, Commands, Mesh, Plugin, Res, ResMut, Startup, Update};

use super::audio_bridge::{scene_audio_bridge_system, SceneAudioState};
use super::common::atmosphere::{atmosphere_transition_system, AtmosphereConfig};
use super::common::parallax::PreviousCameraPosition;
use super::common::weather::{weather_transition_system, weather_update_system, WeatherMachine};
use super::descriptor_config::DescriptorSceneConfig;
use super::generators::registry::GeneratorRegistry;
use super::generators::{barge_system, cloud_system, drop_pod_system, traveller_system};
use super::manager::SceneManager;
use super::tet::config::TetSceneConfig;
use super::tet::systems::{data_trail_system, tet_glow_system};
use crate::theme::ThemeManager;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers scene management, startup activation, and per-frame systems.
///
/// **Resources:** [`SceneManager`], [`AtmosphereConfig`],
///                [`PreviousCameraPosition`], [`WeatherMachine`],
///                [`SceneAudioState`]
/// **Startup:** activates the first registered scene (TET)
/// **Update:** glow, data trails, atmosphere, weather, audio bridge
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        info!("ScenePlugin::build — registering scene systems");

        // Build and register the SceneManager with known scene configs.
        let mut manager = SceneManager::new();
        manager.register(Box::new(TetSceneConfig));

        // The actual initial scene will be loaded dynamically in the startup system
        // based on the InitialScene resource (from CLI args or env vars).

        debug!(
            "ScenePlugin::build — registered scenes: {:?}",
            manager.available_scenes(),
        );

        app.insert_resource(manager)
            .insert_resource(GeneratorRegistry::with_builtins())
            .init_resource::<AtmosphereConfig>()
            .init_resource::<PreviousCameraPosition>()
            .init_resource::<WeatherMachine>()
            .init_resource::<SceneAudioState>()
            .add_systems(Startup, scene_startup_system)
            .add_systems(
                Update,
                (
                    tet_glow_system,
                    data_trail_system,
                    barge_system,
                    traveller_system,
                    cloud_system,
                    drop_pod_system,
                    atmosphere_transition_system,
                    weather_update_system,
                    weather_transition_system,
                    scene_audio_bridge_system,
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// Startup system
// ---------------------------------------------------------------------------

/// Activates the first registered scene via [`SceneManager`] based on [`InitialScene`].
fn scene_startup_system(
    initial_scene: Res<super::InitialScene>,
    mut manager: ResMut<SceneManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Option<Res<ThemeManager>>,
    mut audio_state: ResMut<SceneAudioState>,
) {
    info!(
        "scene_startup_system — activating initial scene: {}",
        initial_scene.0
    );
    let theme_ref = theme.as_deref();

    // First, try to load it from file dynamically if it's not registered
    let requested_scene_str = initial_scene.0.as_str();
    if !manager.available_scenes().contains(&requested_scene_str) {
        match DescriptorSceneConfig::from_file(requested_scene_str) {
            Ok(config) => manager.register(Box::new(config)),
            Err(e) => warn!(
                "Failed to dynamically load requested scene {}: {}",
                requested_scene_str, e
            ),
        }
    }

    // Attempt to activate the requested scene
    let activated = manager.activate(
        requested_scene_str,
        &mut commands,
        &mut meshes,
        &mut materials,
        theme_ref,
        Some(&mut audio_state),
    );

    if !activated {
        // Fallback to TET if the requested scene is not found
        warn!(
            "scene_startup_system — {} not found, falling back to TET",
            initial_scene.0
        );
        let _ = manager.activate(
            "TET",
            &mut commands,
            &mut meshes,
            &mut materials,
            theme_ref,
            Some(&mut audio_state),
        );
    }
    debug!("scene_startup_system — activated scene");
}
