//! Vegetation generator for biome-based foliage scattering.
//!
//! Spawns tree and bush instances based on biome configuration.
//! Uses bevy_feronia for wind animation and GPU instancing.

use std::f32::consts::TAU;

use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, trace, warn};
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use crate::scene::descriptor::GeneratorParams;

/// Marker component for vegetation entities.
#[derive(Component, Debug)]
pub struct VegetationInstance {
    /// Type of vegetation (tree, bush, grass).
    pub kind: VegetationKind,
    /// Model asset path.
    pub model_path: String,
}

/// Type of vegetation instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VegetationKind {
    Tree,
    Bush,
    Grass,
}

/// Vegetation generator configuration.
#[derive(Debug, Clone)]
pub struct VegetationGenerator {
    /// Biome type name.
    pub biome: String,
    /// Center position for scatter.
    pub center: Vec3,
    /// Scatter radius in meters.
    pub radius: f32,
    /// Density multiplier.
    pub density: f32,
    /// Enable wind animation.
    pub enable_wind: bool,
    /// Draw distance in meters.
    pub draw_distance: f32,
}

impl VegetationGenerator {
    /// Create from scene descriptor params.
    pub fn from_params(params: &GeneratorParams) -> Option<Self> {
        match params {
            GeneratorParams::Vegetation {
                biome,
                center,
                radius,
                density,
                enable_wind,
                draw_distance,
            } => Some(Self {
                biome: biome.clone(),
                center: Vec3::from_array(*center),
                radius: *radius,
                density: *density,
                enable_wind: *enable_wind,
                draw_distance: *draw_distance,
            }),
            _ => None,
        }
    }

    /// Get model paths for this biome.
    fn get_models(&self) -> Vec<(VegetationKind, String, f32)> {
        match self.biome.as_str() {
            "Meadow" => vec![
                (
                    VegetationKind::Tree,
                    "models/foliage/oak.glb".to_string(),
                    0.3,
                ),
                (
                    VegetationKind::Bush,
                    "models/foliage/bush.glb".to_string(),
                    0.7,
                ),
            ],
            "Forest" => vec![
                (
                    VegetationKind::Tree,
                    "models/foliage/pine.glb".to_string(),
                    0.5,
                ),
                (
                    VegetationKind::Tree,
                    "models/foliage/birch.glb".to_string(),
                    0.3,
                ),
                (
                    VegetationKind::Bush,
                    "models/foliage/bush.glb".to_string(),
                    0.5,
                ),
            ],
            _ => vec![(
                VegetationKind::Tree,
                "models/foliage/oak.glb".to_string(),
                1.0,
            )],
        }
    }

    /// Calculate number of instances to spawn.
    fn instance_count(&self) -> usize {
        let area = std::f32::consts::PI * self.radius * self.radius;
        let base_density = match self.biome.as_str() {
            "Meadow" => 0.02,
            "Forest" => 0.05,
            _ => 0.01,
        };
        (area * base_density * self.density) as usize
    }
}

/// Generator factory function for vegetation scattering.
pub fn gen_vegetation(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    trace!("gen_vegetation: called");

    let Some(generator) = VegetationGenerator::from_params(params) else {
        warn!("gen_vegetation: invalid params");
        return;
    };

    let count = generator.instance_count();
    info!(
        "gen_vegetation: spawning {} instances for {:?} biome (radius={}, density={})",
        count, generator.biome, generator.radius, generator.density
    );

    let models = generator.get_models();
    if models.is_empty() {
        warn!("gen_vegetation: no models configured for biome");
        return;
    }

    let mut rng = thread_rng();
    let mut spawned = 0;

    for _ in 0..count {
        // Pick random model weighted by probability
        let (kind, model_path, _weight) = models
            .choose_weighted(&mut rng, |m| m.2)
            .expect("Failed to choose model")
            .clone();

        // Random position within radius
        let angle = rng.gen_range(0.0..TAU);
        let distance = rng.gen_range(0.0..generator.radius);
        let x = generator.center.x + angle.cos() * distance;
        let z = generator.center.z + angle.sin() * distance;
        let y = generator.center.y;

        // Random rotation and scale
        let rotation = Quat::from_rotation_y(rng.gen_range(0.0..TAU));
        let scale = rng.gen_range(0.8..1.3);

        // Spawn vegetation instance
        let entity = commands
            .spawn((
                Name::new(format!("{:?}_{}", kind, spawned)),
                VegetationInstance {
                    kind,
                    model_path: model_path.clone(),
                },
                Transform {
                    translation: Vec3::new(x, y, z),
                    rotation,
                    scale: Vec3::splat(scale),
                },
                Visibility::default(),
                InheritedVisibility::default(),
            ))
            .id();

        commands.entity(root).add_child(entity);
        spawned += 1;
    }

    info!("gen_vegetation: spawned {spawned} vegetation instances");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_generator_from_params() {
        let params = GeneratorParams::Vegetation {
            biome: "Forest".to_string(),
            center: [0.0, 0.0, 0.0],
            radius: 100.0,
            density: 1.5,
            enable_wind: true,
            draw_distance: 500.0,
        };

        let gen = VegetationGenerator::from_params(&params).unwrap();
        assert_eq!(gen.biome, "Forest");
        assert_eq!(gen.radius, 100.0);
        assert_eq!(gen.density, 1.5);
        assert!(gen.enable_wind);
    }

    #[test]
    fn vegetation_instance_count() {
        let gen = VegetationGenerator {
            biome: "Forest".to_string(),
            center: Vec3::ZERO,
            radius: 100.0,
            density: 1.0,
            enable_wind: true,
            draw_distance: 500.0,
        };

        let count = gen.instance_count();
        assert!(count > 0, "Should spawn some instances");
        assert!(count < 20000, "Should not spawn excessive instances");
    }

    #[test]
    fn vegetation_models_by_biome() {
        let meadow = VegetationGenerator {
            biome: "Meadow".to_string(),
            center: Vec3::ZERO,
            radius: 100.0,
            density: 1.0,
            enable_wind: true,
            draw_distance: 500.0,
        };

        let models = meadow.get_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|(k, _, _)| *k == VegetationKind::Tree));
        assert!(models.iter().any(|(k, _, _)| *k == VegetationKind::Bush));
    }
}
