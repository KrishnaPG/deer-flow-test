//! Integration tests for shared scene primitives.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::common::parallax::ParallaxLayer;
use deer_gui::scene::primitives::{
    entity_scale, fibonacci_sphere_point, spawn_root, spawn_starfield, Star,
};
use deer_gui::scene::tet::setup::TetMonolith;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_primitives_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

fn count_entities<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn t_prim_01_spawn_root_creates_scene_root() {
    let mut app = build_primitives_app();

    // Count entities before
    let before = app
        .world_mut()
        .query_filtered::<Entity, With<deer_gui::scene::SceneRoot>>()
        .iter(app.world())
        .count();

    // Spawn root using commands via world commands pattern
    {
        let mut commands = app.world_mut().commands();
        spawn_root(&mut commands);
    }
    app.world_mut().flush();

    // Count entities after
    let after = app
        .world_mut()
        .query_filtered::<Entity, With<deer_gui::scene::SceneRoot>>()
        .iter(app.world())
        .count();

    assert_eq!(
        after,
        before + 1,
        "spawn_root should create exactly one SceneRoot entity"
    );
}

#[test]
fn t_prim_02_spawn_starfield_count() {
    let mut app = build_primitives_app();
    let count = 50;

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    spawn_starfield(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        root,
                        bevy::color::LinearRgba::new(2.0, 2.0, 2.0, 1.0),
                        count,
                        800.0,
                    );
                },
            );
        });
    app.world_mut().flush();

    let star_count = count_entities::<Star>(&mut app);
    assert_eq!(
        star_count, count,
        "should spawn exactly {count} Star entities"
    );

    // Verify parallax layers.
    let parallax_count = count_entities::<ParallaxLayer>(&mut app);
    assert_eq!(
        parallax_count, count,
        "every star should have ParallaxLayer"
    );
}

#[test]
fn t_prim_03_fibonacci_sphere_within_radius() {
    let radius = 800.0;
    for i in 0..200 {
        let p = fibonacci_sphere_point(i, 200, radius);
        assert!(
            p.length() <= radius + 1e-3,
            "point {i} at distance {} exceeds radius {radius}",
            p.length()
        );
    }
}

#[test]
fn t_prim_04_entity_scale_in_range() {
    for i in 0..200 {
        let s = entity_scale(i);
        assert!(
            s >= 0.29 && s <= 1.01,
            "entity_scale({i}) = {s} out of range [0.3, 1.0]"
        );
    }
}

#[test]
fn t_prim_05_tet_unchanged_after_refactor() {
    use deer_gui::scene::tet::config::TetSceneConfig;
    use deer_gui::scene::SceneManager;
    use deer_gui::scene::SceneRoot;

    let mut app = build_primitives_app();

    let mut mgr = SceneManager::new();
    mgr.register(Box::new(TetSceneConfig));
    app.insert_resource(mgr);

    // Activate TET.
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(
                    |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                        let mut commands = world.commands();
                        mgr.activate(
                            "TET",
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            None,
                            None,
                        );
                    },
                );
            });
        });
    app.world_mut().flush();

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(root_count, 1, "TET should have one SceneRoot");

    let star_count = count_entities::<Star>(&mut app);
    assert!(star_count > 0, "TET should have stars after refactor");

    let monolith_count = count_entities::<TetMonolith>(&mut app);
    assert_eq!(monolith_count, 1, "TET should have one TetMonolith");
}
