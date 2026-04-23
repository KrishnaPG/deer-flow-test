//! Integration tests for the terrain loading pipeline.
//!
//! These tests verify the complete workflow from heightmap loading to mesh generation,
//! including error handling and edge cases.

use deer_terrain::*;

// ============================================================================
// Heightmap Loading Integration Tests
// ============================================================================

#[test]
fn test_heightmap_full_pipeline_8bit() {
    // Create a 4x4 heightmap with varying heights
    let data = vec![
        0u8, 64, 128, 192, // Row 0: low to high
        64, 128, 192, 255, // Row 1: mid progression
        128, 192, 255, 192, // Row 2: high center
        192, 255, 192, 128, // Row 3: descending
    ];

    let heightmap = Heightmap::from_grayscale(&data, 4, 4).expect("Failed to load heightmap");

    // Verify dimensions
    assert_eq!(heightmap.width, 4);
    assert_eq!(heightmap.height, 4);

    // Verify height values are normalized
    assert!((heightmap.get(0, 0).unwrap() - 0.0).abs() < 1e-6);
    assert!((heightmap.get(3, 0).unwrap() - 192.0 / 255.0).abs() < 1e-6);
    assert!((heightmap.get(1, 3).unwrap() - 1.0).abs() < 1e-6);
}

#[test]
fn test_heightmap_full_pipeline_16bit() {
    // Create a 2x2 16-bit heightmap
    let data = vec![
        0u8, 0u8, // 0 in big-endian
        128u8, 0u8, // 32768 in big-endian
        255u8, 255u8, // 65535 in big-endian
        0u8, 128u8, // 32768 in big-endian (little-endian representation)
    ];

    let heightmap =
        Heightmap::from_grayscale_16bit(&data, 2, 2).expect("Failed to load 16-bit heightmap");

    assert_eq!(heightmap.width, 2);
    assert_eq!(heightmap.height, 2);
}

#[test]
fn test_heightmap_interpolation_pipeline() {
    // Create a simple heightmap with known values for interpolation testing
    let data = vec![
        0u8, 255u8, // Row 0: left=0.0, right=1.0
        255u8, 0u8, // Row 1: left=1.0, right=0.0
    ];

    let heightmap = Heightmap::from_grayscale(&data, 2, 2).expect("Failed to load heightmap");

    // Test bilinear interpolation at various points
    // Center should be exactly 0.5
    let center = heightmap.get_interpolated(0.5, 0.5);
    assert!(
        (center - 0.5).abs() < 0.01,
        "Center interpolation failed: {}",
        center
    );

    // Corner interpolation should match exact values
    let corner_00 = heightmap.get_interpolated(0.0, 0.0);
    assert!((corner_00 - 0.0).abs() < 0.01);

    let corner_10 = heightmap.get_interpolated(1.0, 0.0);
    assert!((corner_10 - 1.0).abs() < 0.01);

    // Edge interpolation
    let edge_mid = heightmap.get_interpolated(0.5, 0.0);
    assert!((edge_mid - 0.5).abs() < 0.01);
}

// ============================================================================
// Mesh Generation Integration Tests
// ============================================================================

#[test]
fn test_mesh_generation_full_pipeline() {
    // Create a heightmap
    let data = vec![128u8; 16]; // 4x4 uniform heightmap
    let heightmap = Heightmap::from_grayscale(&data, 4, 4).expect("Failed to load heightmap");

    // Generate mesh with specific config
    let config = MeshGenConfig {
        world_size: (100.0, 100.0),
        height_scale: 50.0,
        height_offset: 0.0,
        resolution: 8,
        invert: false,
    };

    let mesh = generate_terrain_mesh(&heightmap, &config).expect("Failed to generate mesh");

    // Verify mesh properties
    let expected_vertices = (8 + 1) * (8 + 1);
    assert_eq!(mesh.vertex_count(), expected_vertices);

    let expected_triangles = 8 * 8 * 2;
    assert_eq!(mesh.triangle_count(), expected_triangles);

    // Verify all positions are within expected bounds
    for pos in &mesh.positions {
        assert!(pos[0] >= -50.0 && pos[0] <= 50.0, "X out of bounds");
        assert!(pos[2] >= -50.0 && pos[2] <= 50.0, "Z out of bounds");
    }

    // Verify all normals are normalized
    for normal in &mesh.normals {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 0.01,
            "Normal not normalized: {:?}, length: {}",
            normal,
            len
        );
    }
}

#[test]
fn test_mesh_generation_inverted() {
    // Use a heightmap with gradient: low at one end, high at other
    // This way inversion will clearly swap the heights
    let data = vec![0u8, 64u8, 128u8, 192u8]; // 2x2 gradient
    let heightmap = Heightmap::from_grayscale(&data, 2, 2).expect("Failed to load heightmap");

    let config_normal = MeshGenConfig {
        world_size: (100.0, 100.0),
        height_scale: 10.0,
        resolution: 4,
        invert: false,
        ..Default::default()
    };

    let config_inverted = MeshGenConfig {
        world_size: (100.0, 100.0),
        height_scale: 10.0,
        resolution: 4,
        invert: true,
        ..Default::default()
    };

    let mesh_normal = generate_terrain_mesh(&heightmap, &config_normal).unwrap();
    let mesh_inverted = generate_terrain_mesh(&heightmap, &config_inverted).unwrap();

    // Get heights at corner (0,0) for both meshes
    // In normal, (0,0) height 0 should give Y=0
    // In inverted, (0,0) height 0 should give Y=10
    let corner_normal = mesh_normal.positions[0][1];
    let corner_inverted = mesh_inverted.positions[0][1];

    // For normal: 0.0 * 10 = 0.0
    // For inverted: (1.0 - 0.0) * 10 = 10.0
    assert!(
        (corner_normal - 0.0).abs() < 0.01,
        "Normal corner should be ~0"
    );
    assert!(
        (corner_inverted - 10.0).abs() < 0.01,
        "Inverted corner should be ~10"
    );

    // Verify that inverting produces inverse relationship
    // For any vertex, normal_height + inverted_height = height_scale
    for (pos_normal, pos_inverted) in mesh_normal
        .positions
        .iter()
        .zip(mesh_inverted.positions.iter())
    {
        let sum = pos_normal[1] + pos_inverted[1];
        assert!(
            (sum - 10.0).abs() < 0.01,
            "Inverted heights should sum to scale: {} + {} = {}",
            pos_normal[1],
            pos_inverted[1],
            sum
        );
    }
}

#[test]
fn test_mesh_generation_with_height_offset() {
    let data = vec![128u8; 4]; // Uniform heightmap
    let heightmap = Heightmap::from_grayscale(&data, 2, 2).unwrap();

    let config = MeshGenConfig {
        world_size: (100.0, 100.0),
        height_scale: 10.0,
        height_offset: 100.0, // Base elevation
        resolution: 4,
        ..Default::default()
    };

    let mesh = generate_terrain_mesh(&heightmap, &config).unwrap();

    // All vertices should have Y >= 100.0 (offset)
    for pos in &mesh.positions {
        assert!(pos[1] >= 100.0, "Height below offset: {}", pos[1]);
    }
}

// ============================================================================
// Splat Mask Integration Tests
// ============================================================================

#[test]
fn test_splat_generation_height_based() {
    // Create height data with clear gradients: 0.0 at (0,0) to 1.0 at (3,3)
    let width = 4;
    let height = 4;
    let height_data: Vec<f32> = (0..(width * height))
        .map(|i| (i as f32) / ((width * height - 1) as f32))
        .collect();

    let config = SplatConfig::default();
    let mask = generate_splat_from_height(&height_data, width, height, &config)
        .expect("Failed to generate splat");

    // Verify mask dimensions
    assert_eq!(mask.width, width);
    assert_eq!(mask.height, height);

    // Verify weights are normalized (sum to ~255) for each pixel
    for y in 0..height {
        for x in 0..width {
            let sum: u16 = (0..4)
                .map(|l| mask.get_weight(x, y, l).unwrap() as u16)
                .sum();
            assert!(
                (sum as i32 - 255).abs() <= 5,
                "Weights not normalized at ({}, {}): sum={}",
                x,
                y,
                sum
            );
        }
    }

    // Verify that layer 0 (grass, low height) has higher weight at low heights
    // and layer 3 (snow, high height) has higher weight at high heights
    let layer0_at_low = mask.get_weight(0, 0, 0).unwrap(); // Layer 0 at height ~0
    let layer0_at_high = mask.get_weight(3, 3, 0).unwrap(); // Layer 0 at height ~1
    let layer3_at_low = mask.get_weight(0, 0, 3).unwrap(); // Layer 3 at height ~0
    let layer3_at_high = mask.get_weight(3, 3, 3).unwrap(); // Layer 3 at height ~1

    // Layer 0 should decrease with height
    assert!(
        layer0_at_low > layer0_at_high,
        "Layer 0 should have higher weight at low heights: {} > {}",
        layer0_at_low,
        layer0_at_high
    );

    // Layer 3 should increase with height
    assert!(
        layer3_at_high > layer3_at_low,
        "Layer 3 should have higher weight at high heights: {} > {}",
        layer3_at_high,
        layer3_at_low
    );

    // At low height, layer 0 should be the highest weight among all layers
    let low_height_weights = [
        mask.get_weight(0, 0, 0).unwrap(),
        mask.get_weight(0, 0, 1).unwrap(),
        mask.get_weight(0, 0, 2).unwrap(),
        mask.get_weight(0, 0, 3).unwrap(),
    ];
    let max_low_idx = low_height_weights
        .iter()
        .enumerate()
        .max_by_key(|(_, &w)| w)
        .map(|(i, _)| i)
        .unwrap();
    assert_eq!(
        max_low_idx, 0,
        "Layer 0 should have highest weight at low height, got {:?}",
        low_height_weights
    );

    // At high height, layer 3 should be the highest weight among all layers
    let high_height_weights = [
        mask.get_weight(3, 3, 0).unwrap(),
        mask.get_weight(3, 3, 1).unwrap(),
        mask.get_weight(3, 3, 2).unwrap(),
        mask.get_weight(3, 3, 3).unwrap(),
    ];
    let max_high_idx = high_height_weights
        .iter()
        .enumerate()
        .max_by_key(|(_, &w)| w)
        .map(|(i, _)| i)
        .unwrap();
    assert_eq!(
        max_high_idx, 3,
        "Layer 3 should have highest weight at high height, got {:?}",
        high_height_weights
    );

    // Verify the gradient: as we move from low to high, layer 0 decreases
    // Sample at multiple points along the gradient
    let mut prev_layer0 = layer0_at_low;

    for i in 1..4 {
        let layer0 = mask.get_weight(i as u32, i as u32, 0).unwrap();

        // Layer 0 should generally decrease (or stay similar) as height increases
        // Allow for some non-linearity due to sigmoid function
        assert!(
            (layer0 as i16) <= (prev_layer0 as i16) + 50,
            "Layer 0 should decrease or stay similar as height increases"
        );

        prev_layer0 = layer0;
    }
}

#[test]
fn test_splat_generation_with_slope() {
    let width = 4;
    let height = 4;
    let height_data: Vec<f32> = (0..(width * height))
        .map(|i| (i as f32) / ((width * height - 1) as f32))
        .collect();

    // Create slope data: flat at edges, steep in middle
    let slope_data: Vec<f32> = (0..(width * height))
        .map(|i| {
            let x = (i % width) as f32;
            let y = (i / width) as f32;
            let center_dist = ((x - 1.5).powi(2) + (y - 1.5).powi(2)).sqrt();
            center_dist * 30.0 // 0-60 degrees
        })
        .collect();

    let config = SplatConfig::default();
    let mask =
        generate_splat_from_height_and_slope(&height_data, &slope_data, width, height, &config)
            .expect("Failed to generate height+slope splat");

    // Verify mask was generated
    assert_eq!(mask.width, width);
    assert_eq!(mask.height, height);
}

#[test]
fn test_splat_uniform_generation() {
    let mask = SplatMask::uniform(8, 8);

    assert_eq!(mask.width, 8);
    assert_eq!(mask.height, 8);

    // All pixels should have equal weights
    for y in 0..8 {
        for x in 0..8 {
            for layer in 0..4 {
                assert_eq!(
                    mask.get_weight(x, y, layer),
                    Some(64),
                    "Uniform weight check failed at ({}, {}), layer {}",
                    x,
                    y,
                    layer
                );
            }
        }
    }
}

// ============================================================================
// LOD Integration Tests
// ============================================================================

#[test]
fn test_lod_level_determination() {
    let config = LodConfig::default();

    // Test all LOD levels at various distances
    // Note: LOD uses > comparisons, so at exactly the threshold, higher LOD is returned
    let test_cases = vec![
        (0.0, TerrainLod::High),
        (50.0, TerrainLod::High),
        (99.9, TerrainLod::High),
        (100.1, TerrainLod::Medium), // Just above threshold
        (150.0, TerrainLod::Medium),
        (249.9, TerrainLod::Medium),
        (250.1, TerrainLod::Low),
        (350.0, TerrainLod::Low),
        (499.9, TerrainLod::Low),
        (500.1, TerrainLod::VeryLow),
        (750.0, TerrainLod::VeryLow),
        (999.9, TerrainLod::VeryLow),
        (1000.1, TerrainLod::Culled),
        (2000.0, TerrainLod::Culled),
    ];

    for (distance, expected_lod) in test_cases {
        let lod = TerrainLod::from_distance(distance, &config);
        assert_eq!(lod, expected_lod, "LOD mismatch at distance {}", distance);
    }
}

#[test]
fn test_lod_hysteresis() {
    let config = LodConfig {
        hysteresis: 20.0,
        ..Default::default()
    };

    // Start at High LOD at distance 90
    let lod = calculate_lod(90.0, &config, Some(TerrainLod::High));
    assert_eq!(lod, TerrainLod::High);

    // Move to distance 110 - above 100 threshold, should switch to Medium
    // (hysteresis only applies when going from non-High to High, not from High to lower)
    let lod = calculate_lod(110.0, &config, Some(TerrainLod::High));
    assert_eq!(lod, TerrainLod::Medium);

    // Coming from Medium at distance 90 - below threshold but hysteresis applies
    // Effective distance = 90 + 20 = 110, still above 100, so stays Medium
    // (This is the anti-flicker behavior: once at lower LOD, need to go much closer to upgrade)
    let lod = calculate_lod(90.0, &config, Some(TerrainLod::Medium));
    assert_eq!(lod, TerrainLod::Medium); // Hysteresis keeps it at Medium

    // Need to go below 80 (100 - 20) to switch back to High when coming from Medium
    let lod = calculate_lod(79.0, &config, Some(TerrainLod::Medium));
    assert_eq!(lod, TerrainLod::High);

    // Direct calculation without current LOD
    let lod = TerrainLod::from_distance(90.0, &config);
    assert_eq!(lod, TerrainLod::High); // Direct: 90 < 100, so High
}

// ============================================================================
// Error Handling Integration Tests
// ============================================================================

#[test]
fn test_heightmap_error_handling() {
    // Test invalid dimensions
    let result = Heightmap::from_grayscale(&[0u8; 10], 1, 1);
    assert!(result.is_err());

    // Test size mismatch
    let result = Heightmap::from_grayscale(&[0u8; 10], 4, 4);
    assert!(result.is_err());

    // Test 16-bit size mismatch
    let result = Heightmap::from_grayscale_16bit(&[0u8; 10], 4, 4);
    assert!(result.is_err());
}

#[test]
fn test_splat_error_handling() {
    let height_data = vec![0.5; 4];
    let config = SplatConfig::default();

    // Test size mismatch
    let result = generate_splat_from_height(&height_data, 3, 3, &config);
    assert!(result.is_err());

    // Test invalid layer access
    let mut mask = SplatMask::new(2, 2);
    let result = mask.set_weight(0, 0, 4, 128); // Layer 4 doesn't exist
    assert!(result.is_err());

    // Test out of bounds access
    let result = mask.get_weight(5, 5, 0);
    assert!(result.is_none());
}

// ============================================================================
// Full Pipeline Integration Tests
// ============================================================================

#[test]
fn test_complete_terrain_loading_pipeline() {
    // Simulate loading a heightmap from a file
    let width = 64;
    let height = 64;
    let heightmap_data: Vec<u8> = (0..(width * height))
        .map(|i| {
            let x = (i % width) as f32;
            let y = (i / width) as f32;
            // Create a simple mountain in the center
            let dx = x - 32.0;
            let dy = y - 32.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let height_val = (1.0 - (dist / 32.0).min(1.0)).max(0.0);
            (height_val * 255.0) as u8
        })
        .collect();

    // Step 1: Load heightmap
    let heightmap = Heightmap::from_grayscale(&heightmap_data, width, height)
        .expect("Failed to load heightmap");

    // Step 2: Generate mesh
    let mesh_config = MeshGenConfig {
        world_size: (1000.0, 1000.0),
        height_scale: 200.0,
        height_offset: 0.0,
        resolution: 32,
        invert: false,
    };

    let mesh = generate_terrain_mesh(&heightmap, &mesh_config).expect("Failed to generate mesh");

    // Step 3: Generate splat mask
    let splat_config = SplatConfig::default();
    let splat_mask = generate_splat_from_height(
        &heightmap.values,
        heightmap.width,
        heightmap.height,
        &splat_config,
    )
    .expect("Failed to generate splat mask");

    // Verify all components
    assert_eq!(mesh.vertex_count(), (32 + 1) * (32 + 1));
    assert_eq!(splat_mask.width, width);
    assert_eq!(splat_mask.height, height);

    // Verify mesh bounds
    for pos in &mesh.positions {
        assert!(pos[0] >= -500.0 && pos[0] <= 500.0);
        assert!(pos[2] >= -500.0 && pos[2] <= 500.0);
        assert!(pos[1] >= 0.0 && pos[1] <= 200.0); // Height range
    }
}

#[test]
fn test_terrain_with_different_resolutions() {
    let data = vec![128u8; 16]; // 4x4 heightmap
    let heightmap = Heightmap::from_grayscale(&data, 4, 4).unwrap();

    // Test different mesh resolutions
    let resolutions = vec![4, 8, 16, 32, 64];

    for resolution in resolutions {
        let config = MeshGenConfig {
            world_size: (100.0, 100.0),
            height_scale: 10.0,
            resolution,
            ..Default::default()
        };

        let mesh = generate_terrain_mesh(&heightmap, &config)
            .unwrap_or_else(|_| panic!("Failed to generate mesh at resolution {}", resolution));

        let expected_vertices = (resolution + 1) * (resolution + 1);
        assert_eq!(
            mesh.vertex_count(),
            expected_vertices as usize,
            "Vertex count mismatch at resolution {}",
            resolution
        );

        // Verify mesh is valid
        assert!(mesh.triangle_count() > 0);
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len(), mesh.uvs.len());
    }
}
