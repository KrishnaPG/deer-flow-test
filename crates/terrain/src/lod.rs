//! Terrain LOD (Level of Detail) calculations.
//!
//! Provides LOD level calculations and distance thresholds
//! without Bevy dependencies.

/// LOD level for terrain chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainLod {
    /// LOD 0 - highest detail (close to camera).
    High,
    /// LOD 1 - medium detail.
    Medium,
    /// LOD 2 - low detail.
    Low,
    /// LOD 3 - lowest detail (far from camera).
    VeryLow,
    /// Chunk is not visible (culled).
    Culled,
}

impl TerrainLod {
    /// Get the numeric LOD level (0 = highest detail).
    pub fn level(&self) -> u32 {
        match self {
            Self::High => 0,
            Self::Medium => 1,
            Self::Low => 2,
            Self::VeryLow => 3,
            Self::Culled => 4,
        }
    }

    /// Get the mesh resolution multiplier for this LOD level.
    ///
    /// Returns a factor to multiply the base resolution by.
    pub fn resolution_factor(&self) -> f32 {
        match self {
            Self::High => 1.0,
            Self::Medium => 0.5,
            Self::Low => 0.25,
            Self::VeryLow => 0.125,
            Self::Culled => 0.0,
        }
    }

    /// Determine LOD level from distance.
    pub fn from_distance(distance: f32, config: &LodConfig) -> Self {
        if distance > config.cull_distance {
            Self::Culled
        } else if distance > config.very_low_distance {
            Self::VeryLow
        } else if distance > config.low_distance {
            Self::Low
        } else if distance > config.medium_distance {
            Self::Medium
        } else {
            Self::High
        }
    }
}

/// Configuration for LOD distance thresholds.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Distance to switch from High to Medium.
    pub medium_distance: f32,
    /// Distance to switch from Medium to Low.
    pub low_distance: f32,
    /// Distance to switch from Low to VeryLow.
    pub very_low_distance: f32,
    /// Distance beyond which chunks are culled.
    pub cull_distance: f32,
    /// Distance hysteresis to prevent LOD popping (in meters).
    pub hysteresis: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            medium_distance: 100.0,
            low_distance: 250.0,
            very_low_distance: 500.0,
            cull_distance: 1000.0,
            hysteresis: 10.0,
        }
    }
}

/// Calculate LOD level for a chunk at given distance.
pub fn calculate_lod(
    distance: f32,
    config: &LodConfig,
    current_lod: Option<TerrainLod>,
) -> TerrainLod {
    // Add hysteresis if currently at a higher LOD
    let adjusted_distance = if let Some(current) = current_lod {
        if current != TerrainLod::High {
            distance + config.hysteresis
        } else {
            distance
        }
    } else {
        distance
    };

    TerrainLod::from_distance(adjusted_distance, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lod_level_values() {
        assert_eq!(TerrainLod::High.level(), 0);
        assert_eq!(TerrainLod::Medium.level(), 1);
        assert_eq!(TerrainLod::Low.level(), 2);
        assert_eq!(TerrainLod::VeryLow.level(), 3);
        assert_eq!(TerrainLod::Culled.level(), 4);
    }

    #[test]
    fn lod_resolution_factors() {
        assert_eq!(TerrainLod::High.resolution_factor(), 1.0);
        assert_eq!(TerrainLod::Medium.resolution_factor(), 0.5);
        assert_eq!(TerrainLod::Low.resolution_factor(), 0.25);
        assert_eq!(TerrainLod::VeryLow.resolution_factor(), 0.125);
        assert_eq!(TerrainLod::Culled.resolution_factor(), 0.0);
    }

    #[test]
    fn lod_from_distance() {
        let config = LodConfig::default();

        assert_eq!(TerrainLod::from_distance(50.0, &config), TerrainLod::High);
        assert_eq!(
            TerrainLod::from_distance(150.0, &config),
            TerrainLod::Medium
        );
        assert_eq!(TerrainLod::from_distance(300.0, &config), TerrainLod::Low);
        assert_eq!(
            TerrainLod::from_distance(600.0, &config),
            TerrainLod::VeryLow
        );
        assert_eq!(
            TerrainLod::from_distance(1500.0, &config),
            TerrainLod::Culled
        );
    }

    #[test]
    fn calculate_lod_with_hysteresis() {
        let config = LodConfig::default();

        // At 95m (below medium threshold of 100), should be High
        let lod = calculate_lod(95.0, &config, Some(TerrainLod::High));
        assert_eq!(lod, TerrainLod::High);

        // At 105m (just above medium threshold of 100), should be Medium
        // (hysteresis only applies when current LOD is High and we're checking if we should go lower)
        let lod = calculate_lod(105.0, &config, Some(TerrainLod::High));
        assert_eq!(lod, TerrainLod::Medium);

        // At 115m (above medium + hysteresis), should be Medium
        let lod = calculate_lod(115.0, &config, Some(TerrainLod::High));
        assert_eq!(lod, TerrainLod::Medium);
    }
}
