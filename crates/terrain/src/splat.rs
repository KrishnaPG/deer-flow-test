//! Splat mask generation and utilities for terrain texture blending.
//!
//! Splat masks are RGBA textures where each channel controls the blend weight
//! of a terrain layer. This module provides:
//!
//! - Procedural splat mask generation (height-based, slope-based)
//! - Splat texture creation utilities
//! - RON-compatible configuration

use serde::{Deserialize, Serialize};

use crate::error::TerrainError;

/// Maximum number of terrain layers supported by splat mask (RGBA = 4 layers).
pub const MAX_TERRAIN_LAYERS: usize = 4;

/// Generation mode for splat masks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplatGenerationMode {
    /// Height-based blending: lower elevations use layer 0, higher use layer 3.
    HeightBased,
    /// Slope-based blending: flat areas use one layer, steep areas use another.
    SlopeBased,
    /// Combined height and slope blending.
    HeightAndSlope,
    /// Uniform blend (equal weights for all layers).
    Uniform,
}

impl Default for SplatGenerationMode {
    fn default() -> Self {
        Self::HeightBased
    }
}

/// Configuration for procedural splat mask generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplatConfig {
    /// Generation mode.
    #[serde(default)]
    pub mode: SplatGenerationMode,

    /// Height thresholds for layer transitions (min_height, max_height per layer).
    /// If fewer than 4 entries, remaining layers get default thresholds.
    #[serde(default)]
    pub height_thresholds: Vec<(f32, f32)>,

    /// Slope thresholds for layer transitions (0-90 degrees).
    #[serde(default)]
    pub slope_thresholds: Vec<(f32, f32)>,

    /// Blend sharpness (0 = soft blend, 1 = sharp transitions).
    #[serde(default = "default_sharpness")]
    pub blend_sharpness: f32,

    /// Resolution of the generated splat texture.
    #[serde(default = "default_resolution")]
    pub resolution: u32,
}

fn default_sharpness() -> f32 {
    0.5
}

fn default_resolution() -> u32 {
    256
}

impl Default for SplatConfig {
    fn default() -> Self {
        Self {
            mode: SplatGenerationMode::default(),
            height_thresholds: vec![
                (0.0, 0.25), // Layer 0: low ground (grass)
                (0.2, 0.5),  // Layer 1: mid ground (dirt)
                (0.4, 0.75), // Layer 2: high ground (rock)
                (0.7, 1.0),  // Layer 3: peaks (snow)
            ],
            slope_thresholds: vec![
                (0.0, 15.0),  // Layer 0: flat areas
                (10.0, 30.0), // Layer 1: gentle slopes
                (25.0, 60.0), // Layer 2: steep slopes
                (50.0, 90.0), // Layer 3: cliffs
            ],
            blend_sharpness: 0.5,
            resolution: 256,
        }
    }
}

/// Splat mask data ready for GPU upload.
#[derive(Debug, Clone)]
pub struct SplatMask {
    /// Raw RGBA pixel data (width * height * 4 bytes).
    pub data: Vec<u8>,
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
}

impl SplatMask {
    /// Create a new empty splat mask.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            data: vec![0u8; size],
            width,
            height,
        }
    }

    /// Create a uniform splat mask (equal weights for all layers).
    pub fn uniform(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            data: vec![64u8; size], // 64 = 25% per layer
            width,
            height,
        }
    }

    /// Set the weight for a specific layer at (x, y).
    pub fn set_weight(
        &mut self,
        x: u32,
        y: u32,
        layer: usize,
        weight: u8,
    ) -> Result<(), TerrainError> {
        if layer >= MAX_TERRAIN_LAYERS {
            return Err(TerrainError::InvalidConfig(format!(
                "Layer {} exceeds max {}",
                layer, MAX_TERRAIN_LAYERS
            )));
        }
        if x >= self.width || y >= self.height {
            return Err(TerrainError::InvalidConfig(format!(
                "Position ({}, {}) out of bounds ({}, {})",
                x, y, self.width, self.height
            )));
        }

        let idx = ((y * self.width + x) * 4 + layer as u32) as usize;
        self.data[idx] = weight;
        Ok(())
    }

    /// Get the weight for a specific layer at (x, y).
    pub fn get_weight(&self, x: u32, y: u32, layer: usize) -> Option<u8> {
        if layer >= MAX_TERRAIN_LAYERS || x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4 + layer as u32) as usize;
        Some(self.data[idx])
    }

    /// Normalize weights so each pixel's channels sum to 255.
    pub fn normalize(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let base = ((y * self.width + x) * 4) as usize;
                let r = self.data[base] as u16;
                let g = self.data[base + 1] as u16;
                let b = self.data[base + 2] as u16;
                let a = self.data[base + 3] as u16;
                let sum = r + g + b + a;

                if sum > 0 {
                    let scale = 255.0 / sum as f32;
                    self.data[base] = (r as f32 * scale).min(255.0) as u8;
                    self.data[base + 1] = (g as f32 * scale).min(255.0) as u8;
                    self.data[base + 2] = (b as f32 * scale).min(255.0) as u8;
                    self.data[base + 3] = (a as f32 * scale).min(255.0) as u8;
                } else {
                    // Default to layer 0
                    self.data[base] = 255;
                    self.data[base + 1] = 0;
                    self.data[base + 2] = 0;
                    self.data[base + 3] = 0;
                }
            }
        }
    }
}

/// Generate a splat mask from height data.
///
/// # Arguments
/// * `height_data` - Height values normalized to 0.0-1.0
/// * `width` - Heightmap width
/// * `height` - Heightmap height
/// * `config` - Splat generation configuration
///
/// # Returns
/// SplatMask with RGBA channels representing blend weights for 4 terrain layers.
pub fn generate_splat_from_height(
    height_data: &[f32],
    width: u32,
    height: u32,
    config: &SplatConfig,
) -> Result<SplatMask, TerrainError> {
    if height_data.len() != (width * height) as usize {
        return Err(TerrainError::InvalidConfig(format!(
            "Height data size {} doesn't match {}x{}",
            height_data.len(),
            width,
            height
        )));
    }

    let mut mask = SplatMask::new(width, height);
    let sharpness = (config.blend_sharpness * 10.0).exp();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let h = height_data[idx];

            // Calculate weights for each layer based on height thresholds
            let mut weights = [0.0f32; MAX_TERRAIN_LAYERS];

            for (layer, &(min_h, max_h)) in config
                .height_thresholds
                .iter()
                .enumerate()
                .take(MAX_TERRAIN_LAYERS)
            {
                let mid = (min_h + max_h) / 2.0;
                let range = (max_h - min_h) / 2.0;

                if range > 0.0 {
                    // Smooth blend using sigmoid-like function
                    let t = (h - mid) / range;
                    weights[layer] = 1.0 / (1.0 + (-t * sharpness).exp());
                } else {
                    weights[layer] = if (h - min_h).abs() < 0.01 { 1.0 } else { 0.0 };
                }
            }

            // Normalize weights
            let sum: f32 = weights.iter().sum();
            if sum > 0.0 {
                for (layer, w) in weights.iter_mut().enumerate() {
                    *w /= sum;
                    mask.set_weight(x, y, layer, (*w * 255.0) as u8)?;
                }
            } else {
                mask.set_weight(x, y, 0, 255)?;
            }
        }
    }

    Ok(mask)
}

/// Generate a splat mask from height and slope data.
///
/// Combines height-based and slope-based blending for more realistic terrain.
pub fn generate_splat_from_height_and_slope(
    height_data: &[f32],
    slope_data: &[f32],
    width: u32,
    height: u32,
    config: &SplatConfig,
) -> Result<SplatMask, TerrainError> {
    if slope_data.len() != height_data.len() {
        return Err(TerrainError::InvalidConfig(
            "Height and slope data must have same length".to_string(),
        ));
    }

    let mut mask = SplatMask::new(width, height);
    let sharpness = (config.blend_sharpness * 10.0).exp();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let h = height_data[idx];
            let s = slope_data[idx];

            let mut weights = [0.0f32; MAX_TERRAIN_LAYERS];

            // Height contribution (layers 0, 1 for ground; 2, 3 for high areas)
            for (layer, &(min_h, max_h)) in config.height_thresholds.iter().enumerate().take(2) {
                let mid = (min_h + max_h) / 2.0;
                let range = (max_h - min_h) / 2.0;
                if range > 0.0 {
                    let t = (h - mid) / range;
                    weights[layer] += 0.6 * (1.0 / (1.0 + (-t * sharpness).exp()));
                }
            }

            // Slope contribution (layers 2, 3 for steep areas)
            for (layer_offset, &(min_s, max_s)) in
                config.slope_thresholds.iter().enumerate().skip(2).take(2)
            {
                let layer = layer_offset;
                let mid = (min_s + max_s) / 2.0;
                let range = (max_s - min_s) / 2.0;
                if range > 0.0 {
                    let t = (s - mid) / range;
                    weights[layer] += 0.4 * (1.0 / (1.0 + (-t * sharpness).exp()));
                }
            }

            // Normalize
            let sum: f32 = weights.iter().sum();
            if sum > 0.0 {
                for (layer, w) in weights.iter_mut().enumerate() {
                    *w /= sum;
                    mask.set_weight(x, y, layer, (*w * 255.0) as u8)?;
                }
            } else {
                mask.set_weight(x, y, 0, 255)?;
            }
        }
    }

    Ok(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splat_mask_creation() {
        let mask = SplatMask::new(4, 4);
        assert_eq!(mask.width, 4);
        assert_eq!(mask.height, 4);
        assert_eq!(mask.data.len(), 4 * 4 * 4);
    }

    #[test]
    fn splat_mask_uniform() {
        let mask = SplatMask::uniform(2, 2);
        assert_eq!(mask.get_weight(0, 0, 0), Some(64));
        assert_eq!(mask.get_weight(0, 0, 1), Some(64));
    }

    #[test]
    fn splat_config_default() {
        let config = SplatConfig::default();
        assert_eq!(config.mode, SplatGenerationMode::HeightBased);
        assert_eq!(config.height_thresholds.len(), 4);
        assert_eq!(config.resolution, 256);
    }

    #[test]
    fn generate_height_splat() {
        let height_data = vec![
            0.0, 0.25, 0.5, 0.75, 0.0, 0.25, 0.5, 0.75, 0.0, 0.25, 0.5, 0.75, 0.0, 0.25, 0.5, 0.75,
        ];
        let config = SplatConfig::default();
        let mask = generate_splat_from_height(&height_data, 4, 4, &config);
        assert!(mask.is_ok());

        let mask = mask.unwrap();
        // Low height should have more weight in layer 0
        assert!(mask.get_weight(0, 0, 0).unwrap() > mask.get_weight(0, 0, 3).unwrap());
        // High height should have more weight in layer 3
        assert!(mask.get_weight(3, 3, 3).unwrap() > mask.get_weight(3, 3, 0).unwrap());
    }

    #[test]
    fn normalize_weights() {
        let mut mask = SplatMask::new(1, 1);
        mask.set_weight(0, 0, 0, 100).unwrap();
        mask.set_weight(0, 0, 1, 100).unwrap();
        mask.set_weight(0, 0, 2, 0).unwrap();
        mask.set_weight(0, 0, 3, 0).unwrap();

        mask.normalize();

        let sum = mask.get_weight(0, 0, 0).unwrap() as u16
            + mask.get_weight(0, 0, 1).unwrap() as u16
            + mask.get_weight(0, 0, 2).unwrap() as u16
            + mask.get_weight(0, 0, 3).unwrap() as u16;
        assert_eq!(sum, 255);
    }
}
