//! Procedural vegetation placement.
//!
//! Generates vegetation instance positions based on density,
//! biome filters, and randomization parameters.

use crate::biome::{BiomeFilter, BiomeType};
use crate::error::{Result, VegetationError};

/// Configuration for vegetation placement.
#[derive(Debug, Clone)]
pub struct VegetationConfig {
    /// Density of vegetation (0.0 - 1.0, where 1.0 is maximum density).
    pub density: f32,
    /// Area width for placement.
    pub area_width: f32,
    /// Area height for placement.
    pub area_height: f32,
    /// Random seed for deterministic placement.
    pub seed: u64,
    /// Minimum distance between instances.
    pub min_distance: f32,
    /// Maximum random offset from grid positions.
    pub position_jitter: f32,
    /// Scale variation (min_scale, max_scale).
    pub scale_range: (f32, f32),
    /// Rotation variation (degrees).
    pub rotation_range: (f32, f32),
}

impl Default for VegetationConfig {
    fn default() -> Self {
        Self {
            density: 0.5,
            area_width: 100.0,
            area_height: 100.0,
            seed: 42,
            min_distance: 2.0,
            position_jitter: 1.0,
            scale_range: (0.8, 1.2),
            rotation_range: (0.0, 360.0),
        }
    }
}

/// A single vegetation instance.
#[derive(Debug, Clone)]
pub struct VegetationInstance {
    /// World position (x, y, z).
    pub position: [f32; 3],
    /// Rotation in degrees (yaw).
    pub rotation: f32,
    /// Uniform scale factor.
    pub scale: f32,
    /// Biome type at this location.
    pub biome: BiomeType,
}

/// Result of vegetation placement.
#[derive(Debug, Clone)]
pub struct PlacementResult {
    /// All placed vegetation instances.
    pub instances: Vec<VegetationInstance>,
    /// Number of rejected positions (failed filter).
    pub rejected_count: usize,
}

/// Procedural vegetation placer.
pub struct VegetationPlacer {
    config: VegetationConfig,
    biome_filter: Option<BiomeFilter>,
}

impl VegetationPlacer {
    /// Create a new vegetation placer.
    pub fn new(config: VegetationConfig) -> Self {
        Self {
            config,
            biome_filter: None,
        }
    }

    /// Set the biome filter.
    pub fn with_biome_filter(mut self, filter: BiomeFilter) -> Self {
        self.biome_filter = Some(filter);
        self
    }

    /// Generate vegetation instances using Poisson disk sampling.
    ///
    /// This produces natural-looking distributions without regular grid patterns.
    pub fn place_poisson(&self, height_fn: impl Fn(f32, f32) -> f32) -> Result<PlacementResult> {
        if self.config.density <= 0.0 || self.config.density > 1.0 {
            return Err(VegetationError::InvalidDensity(self.config.density));
        }
        if self.config.area_width <= 0.0 || self.config.area_height <= 0.0 {
            return Err(VegetationError::InvalidArea {
                width: self.config.area_width,
                height: self.config.area_height,
            });
        }

        let mut instances = Vec::new();
        let mut rejected = 0;

        // Calculate cell size based on density and min_distance
        let cell_size = self.config.min_distance / self.config.density.sqrt();
        let grid_width = (self.config.area_width / cell_size).ceil() as usize;
        let grid_height = (self.config.area_height / cell_size).ceil() as usize;

        // Simple grid-based Poisson approximation
        let mut rng = seeded_rng(self.config.seed);

        for gy in 0..grid_height {
            for gx in 0..grid_width {
                // Random chance based on density
                if random_f32(&mut rng) > self.config.density {
                    continue;
                }

                // Calculate position with jitter
                let base_x = gx as f32 * cell_size + cell_size / 2.0;
                let base_z = gy as f32 * cell_size + cell_size / 2.0;

                let jitter_x = (random_f32(&mut rng) - 0.5) * 2.0 * self.config.position_jitter;
                let jitter_z = (random_f32(&mut rng) - 0.5) * 2.0 * self.config.position_jitter;

                let x = (base_x + jitter_x).clamp(0.0, self.config.area_width);
                let z = (base_z + jitter_z).clamp(0.0, self.config.area_height);

                // Get height at position
                let height = height_fn(x, z);

                // Determine biome (simplified - uses height-based classification)
                let biome = classify_biome(height);

                // Check biome filter
                if let Some(ref filter) = self.biome_filter {
                    let slope = 0.0; // Simplified - would need heightmap derivatives
                    if !filter.is_valid(height, slope, biome) {
                        rejected += 1;
                        continue;
                    }
                }

                // Generate rotation and scale
                let rotation = lerp(
                    self.config.rotation_range.0,
                    self.config.rotation_range.1,
                    random_f32(&mut rng),
                );
                let scale = lerp(
                    self.config.scale_range.0,
                    self.config.scale_range.1,
                    random_f32(&mut rng),
                );

                instances.push(VegetationInstance {
                    position: [x, height * 100.0, z], // Scale height for world coordinates
                    rotation,
                    scale,
                    biome,
                });
            }
        }

        Ok(PlacementResult {
            instances,
            rejected_count: rejected,
        })
    }

    /// Generate vegetation instances on a regular grid with randomization.
    pub fn place_grid(&self, height_fn: impl Fn(f32, f32) -> f32) -> Result<PlacementResult> {
        if self.config.density <= 0.0 || self.config.density > 1.0 {
            return Err(VegetationError::InvalidDensity(self.config.density));
        }

        let spacing = self.config.min_distance / self.config.density;
        let grid_width = (self.config.area_width / spacing) as i32;
        let grid_height = (self.config.area_height / spacing) as i32;

        let mut instances = Vec::new();
        let mut rejected = 0;
        let mut rng = seeded_rng(self.config.seed);

        for gz in 0..grid_height {
            for gx in 0..grid_width {
                let base_x = gx as f32 * spacing + spacing / 2.0;
                let base_z = gz as f32 * spacing + spacing / 2.0;

                // Add jitter
                let jitter_x = (random_f32(&mut rng) - 0.5) * 2.0 * self.config.position_jitter;
                let jitter_z = (random_f32(&mut rng) - 0.5) * 2.0 * self.config.position_jitter;

                let x = (base_x + jitter_x).clamp(0.0, self.config.area_width);
                let z = (base_z + jitter_z).clamp(0.0, self.config.area_height);

                let height = height_fn(x, z);
                let biome = classify_biome(height);

                if let Some(ref filter) = self.biome_filter {
                    if !filter.is_valid(height, 0.0, biome) {
                        rejected += 1;
                        continue;
                    }
                }

                let rotation = lerp(
                    self.config.rotation_range.0,
                    self.config.rotation_range.1,
                    random_f32(&mut rng),
                );
                let scale = lerp(
                    self.config.scale_range.0,
                    self.config.scale_range.1,
                    random_f32(&mut rng),
                );

                instances.push(VegetationInstance {
                    position: [x, height * 100.0, z],
                    rotation,
                    scale,
                    biome,
                });
            }
        }

        Ok(PlacementResult {
            instances,
            rejected_count: rejected,
        })
    }
}

/// Simple seeded RNG (Xorshift64).
struct Rng {
    state: u64,
}

fn seeded_rng(seed: u64) -> Rng {
    Rng {
        state: seed.wrapping_add(1),
    }
}

fn random_u64(rng: &mut Rng) -> u64 {
    let mut x = rng.state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    rng.state = x;
    x
}

fn random_f32(rng: &mut Rng) -> f32 {
    (random_u64(rng) % 1000000) as f32 / 1000000.0
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Classify biome based on height (simplified).
fn classify_biome(height: f32) -> BiomeType {
    if height < 0.2 {
        BiomeType::Wetland
    } else if height < 0.4 {
        BiomeType::Meadow
    } else if height < 0.7 {
        BiomeType::Forest
    } else if height < 0.85 {
        BiomeType::Rocky
    } else {
        BiomeType::Snow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_config_default() {
        let config = VegetationConfig::default();
        assert_eq!(config.density, 0.5);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn place_poisson_basic() {
        let config = VegetationConfig {
            density: 0.3,
            area_width: 50.0,
            area_height: 50.0,
            seed: 123,
            ..Default::default()
        };

        let placer = VegetationPlacer::new(config);
        let result = placer.place_poisson(|_, _| 0.5).unwrap();

        assert!(result.instances.len() > 0);
        assert!(result.instances.len() < 100); // Should be reasonable for density 0.3
    }

    #[test]
    fn place_grid_basic() {
        let config = VegetationConfig {
            density: 0.5,
            area_width: 30.0,
            area_height: 30.0,
            seed: 456,
            ..Default::default()
        };

        let placer = VegetationPlacer::new(config);
        let result = placer.place_grid(|_, _| 0.5).unwrap();

        assert!(result.instances.len() > 0);
    }

    #[test]
    fn invalid_density_returns_error() {
        let config = VegetationConfig {
            density: 1.5, // Invalid
            ..Default::default()
        };

        let placer = VegetationPlacer::new(config);
        assert!(placer.place_poisson(|_, _| 0.5).is_err());
    }

    #[test]
    fn biome_filter_works() {
        let config = VegetationConfig {
            density: 0.5,
            area_width: 50.0,
            area_height: 50.0,
            seed: 789,
            ..Default::default()
        };

        let filter = BiomeFilter::new()
            .with_biomes(vec![BiomeType::Meadow])
            .with_height_range(0.3, 0.5);

        let placer = VegetationPlacer::new(config).with_biome_filter(filter);
        let result = placer.place_poisson(|_, _| 0.4).unwrap(); // Height 0.4 is in meadow range

        // All instances should be in meadow biome
        for instance in &result.instances {
            assert_eq!(instance.biome, BiomeType::Meadow);
        }
    }
}
