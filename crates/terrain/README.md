# deer-terrain

Core terrain algorithms for heightmap processing and mesh data generation. This crate provides the data structures and algorithms for terrain generation; the app layer handles Bevy ECS integration and rendering.

## Features

- **Heightmap Loading**: Load and process heightmaps from 8-bit or 16-bit grayscale image data
- **Mesh Generation**: Generate terrain mesh data (positions, normals, UVs, indices) from heightmaps
- **LOD System**: Level of Detail calculations for performance optimization
- **Splat Masks**: Procedural splat mask generation for texture blending (height-based, slope-based)
- **Modular Design**: Zero Bevy dependencies in core algorithms for maximum reusability

## Quick Start

```rust
use deer_terrain::*;

// Load a heightmap from grayscale bytes
let data = vec![0u8, 128, 255, 64]; // 2x2 heightmap
let heightmap = Heightmap::from_grayscale(&data, 2, 2)?;

// Generate a terrain mesh
let config = MeshGenConfig {
    world_size: (1000.0, 1000.0),
    height_scale: 200.0,
    resolution: 64,
    ..Default::default()
};
let mesh = generate_terrain_mesh(&heightmap, &config)?;

// Generate a splat mask for texture blending
let splat_config = SplatConfig::default();
let splat_mask = generate_splat_from_height(
    &heightmap.values,
    width,
    height,
    &splat_config,
)?;
```

## Modules

### `heightmap`

Heightmap loading and processing.

```rust
use deer_terrain::heightmap::{Heightmap, HeightmapConfig};

// Load from 8-bit grayscale
let heightmap = Heightmap::from_grayscale(&bytes, width, height)?;

// Load from 16-bit grayscale (higher precision)
let heightmap = Heightmap::from_grayscale_16bit(&bytes, width, height)?;

// Sample height at grid position
let height = heightmap.get(x, y); // Option<f32>

// Bilinear interpolation for smooth sampling
let height = heightmap.get_interpolated(x, y); // f32
```

### `mesh_gen`

Terrain mesh generation.

```rust
use deer_terrain::mesh_gen::{generate_terrain_mesh, MeshGenConfig, TerrainMeshData};

let config = MeshGenConfig {
    world_size: (1000.0, 1000.0),    // World-space size in meters
    height_scale: 200.0,             // Maximum height in meters
    height_offset: 0.0,              // Base elevation
    resolution: 64,                  // Vertices per side
    invert: false,                   // Invert heightmap
};

let mesh = generate_terrain_mesh(&heightmap, &config)?;

// Access mesh data
let positions: &[[f32; 3]] = &mesh.positions;  // Vertex positions (x, y, z)
let normals: &[[f32; 3]] = &mesh.normals;      // Vertex normals
let uvs: &[[f32; 2]] = &mesh.uvs;              // Texture coordinates
let indices: &[u32] = &mesh.indices;            // Triangle indices

let vertex_count = mesh.vertex_count();
let triangle_count = mesh.triangle_count();
```

### `lod`

Level of Detail calculations.

```rust
use deer_terrain::lod::{TerrainLod, LodConfig, calculate_lod};

// Configure LOD distance thresholds
let config = LodConfig {
    medium_distance: 100.0,    // Switch to Medium at this distance
    low_distance: 250.0,       // Switch to Low at this distance
    very_low_distance: 500.0,  // Switch to VeryLow at this distance
    cull_distance: 1000.0,     // Cull beyond this distance
    hysteresis: 10.0,          // Anti-flicker buffer
};

// Determine LOD from distance
let lod = TerrainLod::from_distance(distance, &config);

// Calculate LOD with hysteresis (prevents flickering)
let lod = calculate_lod(distance, &config, Some(current_lod));

// Use LOD to determine mesh resolution
let resolution_factor = lod.resolution_factor(); // 1.0, 0.5, 0.25, 0.125, or 0.0
let lod_level = lod.level(); // 0, 1, 2, 3, or 4
```

### `splat`

Splat mask generation for terrain texture blending.

```rust
use deer_terrain::splat::{
    generate_splat_from_height,
    generate_splat_from_height_and_slope,
    SplatConfig,
    SplatGenerationMode,
    SplatMask,
};

// Configure splat generation
let config = SplatConfig {
    mode: SplatGenerationMode::HeightBased,
    height_thresholds: vec![
        (0.0, 0.25),  // Layer 0: grass (low)
        (0.2, 0.5),   // Layer 1: dirt (mid)
        (0.4, 0.75),  // Layer 2: rock (high)
        (0.7, 1.0),   // Layer 3: snow (peaks)
    ],
    blend_sharpness: 0.5,
    resolution: 256,
    ..Default::default()
};

// Generate from height only
let mask = generate_splat_from_height(&height_data, width, height, &config)?;

// Generate from height and slope (more realistic)
let mask = generate_splat_from_height_and_slope(
    &height_data,
    &slope_data,
    width,
    height,
    &config,
)?;

// Access splat data (RGBA texture ready for GPU)
let rgba_data: &[u8] = &mask.data;
let width = mask.width;
let height = mask.height;

// Or create a uniform mask
let mask = SplatMask::uniform(256, 256);
```

## Integration with Bevy

This crate provides the core algorithms. For Bevy integration, use the `deer_gui` application which includes:

- `HeightmapTerrain` generator for scene descriptors
- Terrain material with texture splatting
- LOD-based chunk management
- Automatic heightmap loading from assets

## Reusability

This crate has **no Bevy dependencies** in its core algorithms, making it suitable for:

- Game engines other than Bevy
- Offline terrain generation tools
- Server-side terrain processing
- WebAssembly terrain previews

## Error Handling

All functions return `Result<T, TerrainError>`:

```rust
use deer_terrain::TerrainError;

match generate_terrain_mesh(&heightmap, &config) {
    Ok(mesh) => { /* use mesh */ },
    Err(TerrainError::InvalidHeightmapSize { width, height }) => {
        eprintln!("Invalid heightmap dimensions: {}x{}", width, height);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

## License

This crate is part of the Deer project.
