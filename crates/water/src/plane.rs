//! Water plane generation.
//!
//! Generates water plane meshes with configurable dimensions and resolution.

use crate::error::{Result, WaterError};
use crate::waves::{WaveConfig, WaveParams};

/// Configuration for water plane.
#[derive(Debug, Clone)]
pub struct WaterConfig {
    /// Width of the water plane in world units.
    pub width: f32,
    /// Height (depth) of the water plane in world units.
    pub height: f32,
    /// Water level (y position) in world units.
    pub level: f32,
    /// Mesh resolution (subdivisions per side).
    pub resolution: u32,
    /// Wave configuration.
    pub waves: WaveConfig,
    /// Flow direction (x, z) normalized.
    pub flow_direction: (f32, f32),
    /// Flow speed in units per second.
    pub flow_speed: f32,
}

impl Default for WaterConfig {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 1000.0,
            level: 0.0,
            resolution: 64,
            waves: WaveConfig::default(),
            flow_direction: (1.0, 0.0),
            flow_speed: 1.0,
        }
    }
}

/// Generated water mesh data.
#[derive(Debug, Clone)]
pub struct WaterMeshData {
    /// Vertex positions (x, y, z).
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals (x, y, z).
    pub normals: Vec<[f32; 3]>,
    /// Vertex UV coordinates (u, v).
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl WaterMeshData {
    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Water plane generator.
pub struct WaterPlane {
    config: WaterConfig,
}

impl WaterPlane {
    /// Create a new water plane generator.
    pub fn new(config: WaterConfig) -> Self {
        Self { config }
    }

    /// Generate water mesh data at a specific time.
    ///
    /// The time parameter is used for wave animation.
    pub fn generate_mesh(&self, time: f32) -> Result<WaterMeshData> {
        if self.config.width <= 0.0 || self.config.height <= 0.0 {
            return Err(WaterError::InvalidDimensions {
                width: self.config.width,
                height: self.config.height,
            });
        }

        let resolution = self.config.resolution.max(2);
        let half_width = self.config.width / 2.0;
        let half_height = self.config.height / 2.0;

        let vertex_count = ((resolution + 1) * (resolution + 1)) as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);
        let mut indices: Vec<u32> = Vec::with_capacity((resolution * resolution * 6) as usize);

        // Generate vertices with wave displacement
        for z in 0..=resolution {
            for x in 0..=resolution {
                let u = x as f32 / resolution as f32;
                let v = z as f32 / resolution as f32;

                // Base position (centered)
                let px = u * self.config.width - half_width;
                let pz = v * self.config.height - half_height;

                // Calculate wave displacement
                let (displacement, normal) = self.config.waves.calculate(px, pz, time);
                let py = self.config.level + displacement;

                positions.push([px, py, pz]);
                normals.push([normal[0], normal[1], normal[2]]);
                uvs.push([u, v]);
            }
        }

        // Generate indices
        for z in 0..resolution {
            for x in 0..resolution {
                let i0 = (z * (resolution + 1) + x) as u32;
                let i1 = i0 + 1;
                let i2 = i0 + (resolution + 1) as u32;
                let i3 = i2 + 1;

                // Two triangles per quad
                indices.push(i0);
                indices.push(i2);
                indices.push(i1);

                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }

        Ok(WaterMeshData {
            positions,
            normals,
            uvs,
            indices,
        })
    }

    /// Generate a static (flat) water mesh without wave animation.
    pub fn generate_static_mesh(&self) -> Result<WaterMeshData> {
        self.generate_mesh(0.0)
    }
}

/// Create a simple flat water plane mesh.
pub fn create_flat_water_plane(
    width: f32,
    height: f32,
    level: f32,
    resolution: u32,
) -> WaterMeshData {
    let config = WaterConfig {
        width,
        height,
        level,
        resolution,
        waves: WaveConfig::none(),
        ..Default::default()
    };

    let plane = WaterPlane::new(config);
    plane.generate_static_mesh().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_config_default() {
        let config = WaterConfig::default();
        assert_eq!(config.width, 1000.0);
        assert_eq!(config.height, 1000.0);
        assert_eq!(config.level, 0.0);
        assert_eq!(config.resolution, 64);
    }

    #[test]
    fn generate_flat_mesh() {
        let config = WaterConfig {
            width: 100.0,
            height: 100.0,
            level: 10.0,
            resolution: 16,
            waves: WaveConfig::none(),
            ..Default::default()
        };

        let plane = WaterPlane::new(config);
        let mesh = plane.generate_static_mesh().unwrap();

        // (16+1) * (16+1) = 289 vertices
        assert_eq!(mesh.vertex_count(), 289);
        assert!(mesh.triangle_count() > 0);

        // All y positions should be at water level (10.0)
        for pos in &mesh.positions {
            assert!((pos[1] - 10.0).abs() < 1e-6);
        }
    }

    #[test]
    fn generate_animated_mesh() {
        let config = WaterConfig {
            width: 50.0,
            height: 50.0,
            level: 5.0,
            resolution: 8,
            ..Default::default()
        };

        let plane = WaterPlane::new(config);

        // Generate at different times
        let mesh_t0 = plane.generate_mesh(0.0).unwrap();
        let mesh_t1 = plane.generate_mesh(1.0).unwrap();

        // Positions should be different due to waves
        let mut positions_changed = false;
        for (p0, p1) in mesh_t0.positions.iter().zip(mesh_t1.positions.iter()) {
            if (p0[1] - p1[1]).abs() > 1e-6 {
                positions_changed = true;
                break;
            }
        }
        assert!(
            positions_changed,
            "Wave animation should change vertex positions"
        );
    }

    #[test]
    fn flat_water_plane_helper() {
        let mesh = create_flat_water_plane(200.0, 200.0, 5.0, 32);

        // (32+1) * (32+1) = 1089 vertices
        assert_eq!(mesh.vertex_count(), 1089);

        // All normals should point up
        for normal in &mesh.normals {
            assert!((normal[0] - 0.0).abs() < 1e-6);
            assert!((normal[1] - 1.0).abs() < 1e-6);
            assert!((normal[2] - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn invalid_dimensions_error() {
        let config = WaterConfig {
            width: -10.0,
            height: 100.0,
            ..Default::default()
        };

        let plane = WaterPlane::new(config);
        assert!(plane.generate_static_mesh().is_err());
    }
}
