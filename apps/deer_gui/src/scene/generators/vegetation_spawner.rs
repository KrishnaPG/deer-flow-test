use crate::scene::generators::vegetation::VegetationInstance;
use bevy::prelude::*;

pub fn spawn_vegetation_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &VegetationInstance), Added<VegetationInstance>>,
) {
    for (entity, instance) in query.iter() {
        let asset_path = format!("apps/deer_gui/assets/{}", instance.model_path);
        if std::path::Path::new(&asset_path).exists() {
            let scene_handle: Handle<Scene> =
                asset_server.load(format!("{}#Scene0", instance.model_path));
            commands.entity(entity).insert(SceneRoot(scene_handle));
        } else {
            bevy::log::warn!("Vegetation model not found: {}", asset_path);
        }
    }
}
