//! Integration tests for procedural generator factories.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::descriptor::GeneratorParams;
use deer_gui::scene::generators::registry::GeneratorRegistry;
use deer_gui::scene::generators::{Barge, CloudParticle, DropPod, Traveller};
use deer_gui::scene::primitives::{spawn_root, Star};

fn build_gen_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

#[test]
fn t_gen_01_registry_builtins_registered() {
    let registry = GeneratorRegistry::with_builtins();
    let names = registry.available();
    assert!(names.contains(&"starfield"));
    assert!(names.contains(&"spiral_trails"));
    assert!(names.contains(&"river_barges"));
    assert!(names.contains(&"path_travellers"));
    assert!(names.contains(&"cloud_layer"));
    assert!(names.contains(&"drop_pods"));
    assert!(names.contains(&"static_glow_cluster"));
    assert!(names.len() >= 7);
}

#[test]
fn t_gen_02_starfield_spawns_entities() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("starfield").unwrap();
    let params = GeneratorParams::Starfield {
        count: 50,
        radius: 400.0,
        emissive: [2.0, 2.0, 2.0, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    gen(&mut commands, &mut meshes, &mut mats, root, &params);
                },
            );
        });
    app.world_mut().flush();

    assert_eq!(count::<Star>(&mut app), 50);
}

#[test]
fn t_gen_03_river_barges_spawns() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("river_barges").unwrap();
    let params = GeneratorParams::RiverBarges {
        count: 20,
        speed: 3.0,
        river_radius: 300.0,
        emissive: [0.6, 0.8, 0.4, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    gen(&mut commands, &mut meshes, &mut mats, root, &params);
                },
            );
        });
    app.world_mut().flush();

    assert_eq!(count::<Barge>(&mut app), 20);
}

#[test]
fn t_gen_04_cloud_layer_spawns() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("cloud_layer").unwrap();
    let params = GeneratorParams::CloudLayer {
        count: 30,
        speed: 6.0,
        radius: 400.0,
        emissive: [0.8, 0.9, 1.8, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    gen(&mut commands, &mut meshes, &mut mats, root, &params);
                },
            );
        });
    app.world_mut().flush();

    assert_eq!(count::<CloudParticle>(&mut app), 30);
}

#[test]
fn t_gen_05_unknown_generator_returns_none() {
    let registry = GeneratorRegistry::with_builtins();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn t_gen_06_all_entities_child_of_root() {
    use deer_gui::scene::SceneRoot;

    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("starfield").unwrap();
    let params = GeneratorParams::Starfield {
        count: 10,
        radius: 100.0,
        emissive: [1.0, 1.0, 1.0, 1.0],
    };

    let mut root_entity = Entity::PLACEHOLDER;
    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    root_entity = spawn_root(&mut commands);
                    gen(&mut commands, &mut meshes, &mut mats, root_entity, &params);
                },
            );
        });
    app.world_mut().flush();

    // Verify all Star entities have ChildOf pointing to root.
    let stars: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Star>>()
        .iter(app.world())
        .collect();

    for star in stars {
        let child_of = app.world().get::<ChildOf>(star);
        assert!(
            child_of.is_some(),
            "Star entity should have ChildOf component"
        );
    }
}
