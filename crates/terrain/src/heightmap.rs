//! Heightmap loading and processing.
//!
//! Supports loading heightmaps from grayscale image data
//! and converting them to normalized height values.

use crate::error::{Result, TerrainError};

/// A loaded heightmap with normalized height values.
#[derive(Debug, Clone)]
pub struct Heightmap {
    /// Width of the heightmap (in vertices).
    pub width: u32,
    /// Height of the heightmap (in vertices).
    pub height: u32,
    /// Normalized height values (0.0 - 1.0).
    pub values: Vec<f32>,
}

/// Configuration for heightmap loading.
#[derive(Debug, Clone)]
pub struct HeightmapConfig {
    /// World-space size of the terrain (x, z) in meters.
    pub world_size: (f32, f32),
    /// Maximum height in meters.
    pub height_scale: f32,
    /// Height offset (base terrain level).
    pub height_offset: f32,
    /// Whether to invert the heightmap.
    pub invert: bool,
}

impl Default for HeightmapConfig {
    fn default() -> Self {
        Self {
            world_size: (1000.0, 1000.0),
            height_scale: 100.0,
            height_offset: 0.0,
            invert: false,
        }
    }
}

impl Heightmap {
    /// Load a heightmap from raw grayscale bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw grayscale image bytes (u8 per pixel)
    /// * `width` - Image width
    /// * `height` - Image height
    ///
    /// # Returns
    ///
    /// A normalized heightmap with values in range [0.0, 1.0]
    pub fn from_grayscale(data: &[u8], width: u32, height: u32) -> Result<Self> {
        if width < 2 || height < 2 {
            return Err(TerrainError::InvalidHeightmapSize { width, height });
        }

        let expected_len = (width * height) as usize;
        if data.len() != expected_len {
            return Err(TerrainError::HeightmapLoadError(format!(
                "Expected {} bytes, got {}",
                expected_len,
                data.len()
            )));
        }

        let values: Vec<f32> = data.iter().map(|&v| v as f32 / 255.0).collect();

        Ok(Self {
            width,
            height,
            values,
        })
    }

    /// Load a heightmap from 16-bit grayscale data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw 16-bit grayscale image bytes (big-endian u16 per pixel)
    /// * `width` - Image width
    /// * `height` - Image height
    ///
    /// # Returns
    ///
    /// A normalized heightmap with values in range [0.0, 1.0]
    pub fn from_grayscale_16bit(data: &[u8], width: u32, height: u32) -> Result<Self> {
        if width < 2 || height < 2 {
            return Err(TerrainError::InvalidHeightmapSize { width, height });
        }

        let expected_len = (width * height * 2) as usize;
        if data.len() != expected_len {
            return Err(TerrainError::HeightmapLoadError(format!(
                "Expected {} bytes, got {}",
                expected_len,
                data.len()
            )));
        }

        let values: Vec<f32> = data
            .chunks_exact(2)
            .map(|chunk| {
                let val = u16::from_be_bytes([chunk[0], chunk[1]]);
                val as f32 / 65535.0
            })
            .collect();

        Ok(Self {
            width,
            height,
            values,
        })
    }

    /// Get height at a specific grid position.
    pub fn get(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        Some(self.values[idx])
    }

    /// Get interpolated height at floating-point coordinates.
    ///
    /// Uses bilinear interpolation for smooth height sampling.
    pub fn get_interpolated(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let h00 = self.get_clamped(x0, y0);
        let h10 = self.get_clamped(x1, y0);
        let h01 = self.get_clamped(x0, y1);
        let h11 = self.get_clamped(x1, y1);

        // Bilinear interpolation
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;
        h0 * (1.0 - fy) + h1 * fy
    }

    /// Get height at coordinates, clamping to edges.
    fn get_clamped(&self, x: i32, y: i32) -> f32 {
        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        self.get(x, y).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heightmap_from_grayscale() {
        // Data is row-major: row0 = [0, 128], row1 = [255, 64]
        let data = vec![0u8, 128, 255, 64];
        let heightmap = Heightmap::from_grayscale(&data, 2, 2).unwrap();

        assert_eq!(heightmap.width, 2);
        assert_eq!(heightmap.height, 2);
        // (0,0) = first element
        assert!((heightmap.get(0, 0).unwrap() - 0.0).abs() < 1e-6);
        // (1,0) = second element
        assert!((heightmap.get(1, 0).unwrap() - 128.0 / 255.0).abs() < 1e-6);
        // (0,1) = third element (first of second row)
        assert!((heightmap.get(0, 1).unwrap() - 255.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn heightmap_interpolation() {
        let mut data = vec![0u8; 4];
        data[0] = 0;
        data[1] = 255;
        data[2] = 255;
        data[3] = 0;

        let heightmap = Heightmap::from_grayscale(&data, 2, 2).unwrap();

        // Center should be approximately 0.5
        let h = heightmap.get_interpolated(0.5, 0.5);
        assert!((h - 0.5).abs() < 0.01);
    }

    #[test]
    fn heightmap_invalid_size() {
        let data = vec![0u8; 10];
        assert!(Heightmap::from_grayscale(&data, 1, 1).is_err());
    }

    #[test]
    fn heightmap_out_of_bounds() {
        let data = vec![128u8; 4];
        let heightmap = Heightmap::from_grayscale(&data, 2, 2).unwrap();
        assert!(heightmap.get(2, 0).is_none());
        assert!(heightmap.get(0, 2).is_none());
    }
}
