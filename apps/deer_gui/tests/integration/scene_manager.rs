//! Integration tests for [`SceneManager`] — registration, activation,
//! deactivation, and scene switching.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::audio_bridge::SceneAudioState;
use deer_gui::scene::tet::config::TetSceneConfig;
use deer_gui::scene::tet::setup::{Star, TetMonolith};
use deer_gui::scene::{SceneConfig, SceneManager, SceneRoot};

// ---------------------------------------------------------------------------
// Dummy scene for multi-scene tests
// ---------------------------------------------------------------------------

/// Minimal scene config used to test scene switching.
struct DummySceneConfig;

impl SceneConfig for DummySceneConfig {
    fn name(&self) -> &str {
        "Dummy"
    }

    fn spawn_environment(
        &self,
        commands: &mut Commands,
        _meshes: &mut Assets<Mesh>,
        _materials: &mut Assets<StandardMaterial>,
        _theme: Option<&deer_gui::theme::ThemeManager>,
    ) -> Entity {
        commands.spawn((SceneRoot, Transform::default())).id()
    }

    fn ambient_audio_track(&self) -> &'static str {
        "audio/dummy.ogg"
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with asset support and a [`SceneManager`]
/// pre-loaded with [`TetSceneConfig`].
fn build_scene_mgr_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();

    let mut mgr = SceneManager::new();
    mgr.register(Box::new(TetSceneConfig));
    app.insert_resource(mgr);
    app
}

/// Activates a scene by name within the app's world.
///
/// Runs an exclusive system that calls [`SceneManager::activate`],
/// then applies deferred commands via `app.update()`.
fn activate_scene(app: &mut App, name: &'static str) -> bool {
    let mut result = false;
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(
                    |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                        let mut commands = world.commands();
                        result = mgr.activate(
                            name,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            None, // no theme in tests
                            None, // no audio state in tests
                        );
                    },
                );
            });
        });
    // Flush deferred commands.
    app.world_mut().flush();
    result
}

/// Deactivates the current scene within the app's world.
fn deactivate_scene(app: &mut App) {
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            let mut commands = world.commands();
            mgr.deactivate(&mut commands, None);
        });
    app.world_mut().flush();
}

/// Counts entities with a given component.
fn count_entities<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-01  Register and list
// ---------------------------------------------------------------------------

#[test]
fn t_scene_mgr_01_register_and_list() {
    let mut mgr = SceneManager::new();
    mgr.register(Box::new(TetSceneConfig));
    mgr.register(Box::new(DummySceneConfig));

    let scenes = mgr.available_scenes();
    assert_eq!(scenes.len(), 2);
    assert!(scenes.contains(&"TET"));
    assert!(scenes.contains(&"Dummy"));
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-02  Activate spawns SceneRoot
// ---------------------------------------------------------------------------

#[test]
fn t_scene_mgr_02_activate_spawns_root() {
    let mut app = build_scene_mgr_app();

    let activated = activate_scene(&mut app, "TET");
    assert!(
        activated,
        "activate should return true for registered scene"
    );

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(root_count, 1, "should have exactly one SceneRoot entity");

    // Verify child entities were spawned.
    let star_count = count_entities::<Star>(&mut app);
    assert!(star_count > 0, "should have spawned star entities");

    let monolith_count = count_entities::<TetMonolith>(&mut app);
    assert_eq!(monolith_count, 1, "should have one TetMonolith");
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-03  Deactivate removes entities
// ---------------------------------------------------------------------------

#[test]
fn t_scene_mgr_03_deactivate_removes_entities() {
    let mut app = build_scene_mgr_app();

    activate_scene(&mut app, "TET");
    assert_eq!(count_entities::<SceneRoot>(&mut app), 1);

    deactivate_scene(&mut app);

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(
        root_count, 0,
        "SceneRoot should be despawned after deactivate"
    );

    let star_count = count_entities::<Star>(&mut app);
    assert_eq!(star_count, 0, "Stars should be despawned with root");

    let monolith_count = count_entities::<TetMonolith>(&mut app);
    assert_eq!(
        monolith_count, 0,
        "TetMonolith should be despawned with root"
    );
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-04  Switch scenes
// ---------------------------------------------------------------------------

#[test]
fn t_scene_mgr_04_switch_scenes() {
    let mut app = build_scene_mgr_app();

    // Also register the dummy scene.
    app.world_mut()
        .resource_mut::<SceneManager>()
        .register(Box::new(DummySceneConfig));

    // Activate TET.
    activate_scene(&mut app, "TET");
    assert_eq!(count_entities::<SceneRoot>(&mut app), 1);
    assert!(count_entities::<Star>(&mut app) > 0, "TET has stars");

    // Switch to Dummy — should despawn TET and spawn Dummy root.
    activate_scene(&mut app, "Dummy");

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(
        root_count, 1,
        "should have exactly one SceneRoot after switch"
    );

    // TET-specific entities should be gone.
    let star_count = count_entities::<Star>(&mut app);
    assert_eq!(star_count, 0, "Stars from TET should be despawned");

    let monolith_count = count_entities::<TetMonolith>(&mut app);
    assert_eq!(
        monolith_count, 0,
        "TetMonolith from TET should be despawned"
    );

    // Verify manager state.
    let mgr = app.world().resource::<SceneManager>();
    assert_eq!(mgr.current_name(), Some("Dummy"));
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-05  Activate unknown returns false
// ---------------------------------------------------------------------------

#[test]
fn t_scene_mgr_05_activate_unknown_returns_false() {
    let mut app = build_scene_mgr_app();

    let activated = activate_scene(&mut app, "NonExistent");
    assert!(!activated, "activate should return false for unknown scene");

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(root_count, 0, "no SceneRoot should exist for unknown scene");
}

// ---------------------------------------------------------------------------
// T-SCENE-MGR-06  Full pipeline: activate → SceneAudioState → track set
// ---------------------------------------------------------------------------

/// Verifies the full scene-audio pipeline: activating a scene through
/// [`SceneManager`] with a [`SceneAudioState`] results in the correct
/// ambient track being requested, and deactivating clears it.
///
/// This is the I-4 integration test from code review.
#[test]
fn t_scene_mgr_06_activate_sets_audio_state() {
    let mut app = build_scene_mgr_app();

    // Also register the dummy scene for multi-scene verification.
    app.world_mut()
        .resource_mut::<SceneManager>()
        .register(Box::new(DummySceneConfig));

    // Insert SceneAudioState so activate can wire audio.
    app.insert_resource(SceneAudioState::default());

    // --- Activate TET with audio state ---
    {
        let mut result = false;
        app.world_mut()
            .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
                world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                    world.resource_scope(
                        |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                            world.resource_scope(
                                |world: &mut World, mut audio_state: Mut<SceneAudioState>| {
                                    let mut commands = world.commands();
                                    result = mgr.activate(
                                        "TET",
                                        &mut commands,
                                        &mut meshes,
                                        &mut materials,
                                        None,
                                        Some(&mut audio_state),
                                    );
                                },
                            );
                        },
                    );
                });
            });
        app.world_mut().flush();
        assert!(result, "TET activation should succeed");
    }

    // Verify SceneAudioState has the TET ambient track.
    {
        let state = app.world().resource::<SceneAudioState>();
        assert_eq!(
            state.desired_track(),
            Some("audio/tet_ambient.ogg"),
            "desired track should be TET ambient after activation"
        );
        assert!(
            state.has_pending_change(),
            "should have a pending change (bridge hasn't run yet)"
        );
    }

    // --- Deactivate with audio state ---
    {
        app.world_mut()
            .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
                world.resource_scope(|world: &mut World, mut audio_state: Mut<SceneAudioState>| {
                    let mut commands = world.commands();
                    mgr.deactivate(&mut commands, Some(&mut audio_state));
                });
            });
        app.world_mut().flush();
    }

    // Verify SceneAudioState is cleared.
    {
        let state = app.world().resource::<SceneAudioState>();
        assert_eq!(
            state.desired_track(),
            None,
            "desired track should be None after deactivation"
        );
    }

    // --- Activate Dummy and verify different track ---
    {
        app.world_mut()
            .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
                world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                    world.resource_scope(
                        |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                            world.resource_scope(
                                |world: &mut World, mut audio_state: Mut<SceneAudioState>| {
                                    let mut commands = world.commands();
                                    mgr.activate(
                                        "Dummy",
                                        &mut commands,
                                        &mut meshes,
                                        &mut materials,
                                        None,
                                        Some(&mut audio_state),
                                    );
                                },
                            );
                        },
                    );
                });
            });
        app.world_mut().flush();
    }

    // Verify SceneAudioState has Dummy's track.
    {
        let state = app.world().resource::<SceneAudioState>();
        assert_eq!(
            state.desired_track(),
            Some("audio/dummy.ogg"),
            "desired track should be Dummy ambient after switching"
        );
    }
}
