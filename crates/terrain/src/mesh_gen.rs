//! Terrain mesh generation.
//!
//! Generates mesh data (positions, normals, UVs, indices) for terrain
//! that can be used with any 3D rendering system.

use crate::error::Result;
use crate::heightmap::Heightmap;

/// Configuration for mesh generation.
#[derive(Debug, Clone)]
pub struct MeshGenConfig {
    /// World-space size (width, depth) in meters.
    pub world_size: (f32, f32),
    /// Maximum height in meters.
    pub height_scale: f32,
    /// Base height offset in meters.
    pub height_offset: f32,
    /// Mesh resolution (vertices per side).
    pub resolution: u32,
    /// Whether to invert the heightmap.
    pub invert: bool,
}

impl Default for MeshGenConfig {
    fn default() -> Self {
        Self {
            world_size: (1000.0, 1000.0),
            height_scale: 100.0,
            height_offset: 0.0,
            resolution: 64,
            invert: false,
        }
    }
}

/// Generated terrain mesh data.
#[derive(Debug, Clone)]
pub struct TerrainMeshData {
    /// Vertex positions (x, y, z).
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals (x, y, z).
    pub normals: Vec<[f32; 3]>,
    /// Vertex UV coordinates (u, v).
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl TerrainMeshData {
    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Generate terrain mesh data from a heightmap.
///
/// # Arguments
///
/// * `heightmap` - The source heightmap
/// * `config` - Mesh generation configuration
///
/// # Returns
///
/// Mesh data ready for upload to GPU
pub fn generate_terrain_mesh(
    heightmap: &Heightmap,
    config: &MeshGenConfig,
) -> Result<TerrainMeshData> {
    let resolution = config.resolution.max(2);
    let (world_width, world_depth) = config.world_size;

    let vertex_count = ((resolution + 1) * (resolution + 1)) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);
    let mut indices: Vec<u32> = Vec::with_capacity((resolution * resolution * 6) as usize);

    // Generate vertices
    for z in 0..=resolution {
        for x in 0..=resolution {
            // UV coordinates (0-1)
            let u = x as f32 / resolution as f32;
            let v = z as f32 / resolution as f32;

            // Sample heightmap
            let hx = u * (heightmap.width - 1) as f32;
            let hz = v * (heightmap.height - 1) as f32;
            let height = heightmap.get_interpolated(hx, hz);

            // Apply height configuration
            let world_height = if config.invert {
                (1.0 - height) * config.height_scale + config.height_offset
            } else {
                height * config.height_scale + config.height_offset
            };

            // World position (centered at origin)
            let world_x = u * world_width - world_width / 2.0;
            let world_z = v * world_depth - world_depth / 2.0;

            positions.push([world_x, world_height, world_z]);
            uvs.push([u, v]);
            normals.push([0.0, 1.0, 0.0]); // Will compute later
        }
    }

    // Generate indices (two triangles per quad)
    for z in 0..resolution {
        for x in 0..resolution {
            let i0 = (z * (resolution + 1) + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + (resolution + 1) as u32;
            let i3 = i2 + 1;

            // First triangle
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            // Second triangle
            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    // Compute proper normals
    compute_normals(&mut positions, &mut normals, &indices);

    Ok(TerrainMeshData {
        positions,
        normals,
        uvs,
        indices,
    })
}

/// Compute vertex normals from positions and indices.
fn compute_normals(positions: &[[f32; 3]], normals: &mut [[f32; 3]], indices: &[u32]) {
    use bevy::math::Vec3;

    // Zero out normals
    for normal in normals.iter_mut() {
        *normal = [0.0, 0.0, 0.0];
    }

    // Accumulate face normals
    for chunk in indices.chunks_exact(3) {
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        let v0 = Vec3::from(positions[i0]);
        let v1 = Vec3::from(positions[i1]);
        let v2 = Vec3::from(positions[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();

        // Add to vertex normals
        for &idx in &[i0, i1, i2] {
            let n = Vec3::from(normals[idx]);
            let sum = n + face_normal;
            normals[idx] = [sum.x, sum.y, sum.z];
        }
    }

    // Normalize all normals
    for normal in normals.iter_mut() {
        let n = Vec3::from(*normal);
        let len = n.length();
        if len > 0.0 {
            let normalized = n / len;
            *normal = [normalized.x, normalized.y, normalized.z];
        }
    }
}

/// Create a simple flat terrain mesh (for testing).
pub fn create_flat_mesh(width: f32, depth: f32, resolution: u32) -> TerrainMeshData {
    let config = MeshGenConfig {
        world_size: (width, depth),
        height_scale: 0.0,
        height_offset: 0.0,
        resolution,
        invert: false,
    };

    // Create a minimal 2x2 heightmap with zero height
    let heightmap = Heightmap::from_grayscale(&[128u8; 4], 2, 2).unwrap();

    generate_terrain_mesh(&heightmap, &config).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_gen_config_default() {
        let config = MeshGenConfig::default();
        assert_eq!(config.world_size, (1000.0, 1000.0));
        assert_eq!(config.resolution, 64);
    }

    #[test]
    fn generate_simple_mesh() {
        // Create a 2x2 heightmap with uniform height
        let data = vec![128u8; 4];
        let heightmap = Heightmap::from_grayscale(&data, 2, 2).unwrap();

        let config = MeshGenConfig {
            world_size: (100.0, 100.0),
            height_scale: 50.0,
            resolution: 4,
            ..Default::default()
        };

        let mesh = generate_terrain_mesh(&heightmap, &config).unwrap();

        // (4+1) * (4+1) = 25 vertices
        assert_eq!(mesh.vertex_count(), 25);
        // 4 * 4 * 2 triangles = 32 triangles = 96 indices
        assert_eq!(mesh.triangle_count(), 32);
    }

    #[test]
    fn flat_mesh_generation() {
        let mesh = create_flat_mesh(100.0, 100.0, 8);

        // (8+1) * (8+1) = 81 vertices
        assert_eq!(mesh.vertex_count(), 81);

        // All heights should be 0
        for pos in &mesh.positions {
            assert!((pos[1] - 0.0).abs() < 1e-6);
        }

        // All normals should point up (0, 1, 0)
        for normal in &mesh.normals {
            assert!((normal[0] - 0.0).abs() < 1e-6);
            assert!((normal[1] - 1.0).abs() < 1e-6);
            assert!((normal[2] - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn mesh_data_counts() {
        let mesh = create_flat_mesh(50.0, 50.0, 16);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len(), mesh.uvs.len());
    }
}
