use crate::scene::generators::terrain::TerrainHeightmap;
use crate::scene::generators::vegetation::VegetationInstance;
use bevy::prelude::*;

pub fn spawn_vegetation_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    heightmap: Option<Res<TerrainHeightmap>>,
    mut query: Query<(Entity, &VegetationInstance, &mut Transform), Added<VegetationInstance>>,
) {
    for (entity, instance, mut transform) in query.iter_mut() {
        // Adjust height based on terrain
        if let Some(hm) = &heightmap {
            let y = hm.get_height_at(transform.translation.x, transform.translation.z);
            transform.translation.y = y;
        }

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
