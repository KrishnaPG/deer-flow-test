//! Biome classification and filtering for vegetation placement.

use serde::Deserialize;

/// Biome type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum BiomeType {
    /// Grassland / meadow biome.
    Meadow,
    /// Forest / woodland biome.
    Forest,
    /// Rocky / mountain biome.
    Rocky,
    /// Wetland / swamp biome.
    Wetland,
    /// Desert / arid biome.
    Desert,
    /// Snow / alpine biome.
    Snow,
}

impl BiomeType {
    /// Get all biome types.
    pub fn all() -> &'static [BiomeType] {
        &[
            Self::Meadow,
            Self::Forest,
            Self::Rocky,
            Self::Wetland,
            Self::Desert,
            Self::Snow,
        ]
    }
}

/// Filter for biome-based vegetation placement.
#[derive(Debug, Clone)]
pub struct BiomeFilter {
    /// Allowed biome types.
    pub allowed_biomes: Vec<BiomeType>,
    /// Minimum height for vegetation (0.0 - 1.0 normalized).
    pub min_height: f32,
    /// Maximum height for vegetation (0.0 - 1.0 normalized).
    pub max_height: f32,
    /// Minimum slope angle (degrees, 0 = flat).
    pub min_slope: f32,
    /// Maximum slope angle (degrees, 90 = vertical).
    pub max_slope: f32,
}

impl Default for BiomeFilter {
    fn default() -> Self {
        Self {
            allowed_biomes: vec![BiomeType::Meadow, BiomeType::Forest],
            min_height: 0.0,
            max_height: 1.0,
            min_slope: 0.0,
            max_slope: 45.0,
        }
    }
}

impl BiomeFilter {
    /// Create a new biome filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set allowed biomes.
    pub fn with_biomes(mut self, biomes: Vec<BiomeType>) -> Self {
        self.allowed_biomes = biomes;
        self
    }

    /// Set height range.
    pub fn with_height_range(mut self, min: f32, max: f32) -> Self {
        self.min_height = min;
        self.max_height = max;
        self
    }

    /// Set slope range.
    pub fn with_slope_range(mut self, min: f32, max: f32) -> Self {
        self.min_slope = min;
        self.max_slope = max;
        self
    }

    /// Check if a position passes the biome filter.
    ///
    /// # Arguments
    ///
    /// * `height` - Normalized height (0.0 - 1.0)
    /// * `slope` - Slope angle in degrees
    /// * `biome` - Biome type at the position
    pub fn is_valid(&self, height: f32, slope: f32, biome: BiomeType) -> bool {
        if !self.allowed_biomes.contains(&biome) {
            return false;
        }
        if height < self.min_height || height > self.max_height {
            return false;
        }
        if slope < self.min_slope || slope > self.max_slope {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_type_all() {
        assert_eq!(BiomeType::all().len(), 6);
    }

    #[test]
    fn biome_filter_default() {
        let filter = BiomeFilter::default();
        assert!(filter.allowed_biomes.contains(&BiomeType::Meadow));
        assert!(filter.allowed_biomes.contains(&BiomeType::Forest));
        assert_eq!(filter.min_height, 0.0);
        assert_eq!(filter.max_height, 1.0);
    }

    #[test]
    fn biome_filter_is_valid() {
        let filter = BiomeFilter::new()
            .with_height_range(0.2, 0.8)
            .with_slope_range(0.0, 30.0);

        // Valid position
        assert!(filter.is_valid(0.5, 15.0, BiomeType::Forest));

        // Height too low
        assert!(!filter.is_valid(0.1, 15.0, BiomeType::Forest));

        // Height too high
        assert!(!filter.is_valid(0.9, 15.0, BiomeType::Forest));

        // Slope too steep
        assert!(!filter.is_valid(0.5, 45.0, BiomeType::Forest));

        // Wrong biome
        assert!(!filter.is_valid(0.5, 15.0, BiomeType::Desert));
    }

    #[test]
    fn biome_filter_builder() {
        let filter = BiomeFilter::new()
            .with_biomes(vec![BiomeType::Meadow])
            .with_height_range(0.1, 0.9)
            .with_slope_range(5.0, 60.0);

        assert_eq!(filter.allowed_biomes.len(), 1);
        assert_eq!(filter.min_height, 0.1);
        assert_eq!(filter.max_slope, 60.0);
    }
}
