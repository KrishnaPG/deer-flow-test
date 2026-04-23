//! [`ScenePlugin`] — registers scene resources, startup, and per-frame systems.

use bevy::asset::Assets;
use bevy::log::{debug, info};
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
use super::tet::systems::{data_trail_system, tet_glow_system};
use super::traits::SceneConfig;
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

        // Build and register the SceneManager.
        // Scenes are loaded dynamically from .scene.ron files in the startup system
        // based on the InitialScene resource (from CLI args or env vars).
        let manager = SceneManager::new();

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

/// Activates the requested scene via [`SceneManager`] based on [`InitialScene`].
/// Fails loudly (panics) if the scene cannot be loaded — no silent fallbacks.
fn scene_startup_system(
    initial_scene: Res<super::InitialScene>,
    mut manager: ResMut<SceneManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Option<Res<ThemeManager>>,
    mut audio_state: ResMut<SceneAudioState>,
    generators: Res<GeneratorRegistry>,
) {
    let requested_scene = &initial_scene.0;
    info!(
        "scene_startup_system — activating initial scene: {}",
        requested_scene
    );
    let theme_ref = theme.as_deref();

    // Determine the scene name to activate.
    // If the scene file defines a different name (e.g., case) than the CLI argument,
    // we use the name declared inside the file.
    let requested_scene_str = requested_scene.as_str();
    let scene_name_to_activate = if manager.available_scenes().contains(&requested_scene_str) {
        requested_scene_str
    } else {
        // Load from file and get the actual declared scene name
        let config = DescriptorSceneConfig::from_file(requested_scene_str).unwrap_or_else(|e| {
            panic!(
                "Failed to load scene '{}' from file: {}. \
                     Check that the .scene.ron file exists and is valid.",
                requested_scene, e
            )
        });
        // Store the actual name before moving config into Box
        let actual_name = config.name().to_string();
        manager.register(Box::new(config));
        // We can't return a reference to local String, so we leak it for 'static
        Box::leak(actual_name.into_boxed_str())
    };

    // Activate the scene — panic if activation fails
    let activated = manager.activate(
        scene_name_to_activate,
        &mut commands,
        &mut meshes,
        &mut materials,
        theme_ref,
        Some(&mut audio_state),
        &generators,
    );

    if !activated {
        panic!(
            "Scene '{}' (requested as '{}') not found after registration. \
             This should never happen — the scene was just registered.",
            scene_name_to_activate, requested_scene
        );
    }

    info!(
        "scene_startup_system — successfully activated scene '{}'",
        requested_scene
    );
}
