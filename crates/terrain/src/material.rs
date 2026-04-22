//! Terrain material with texture splatting.
//!
//! Supports up to 4 terrain texture layers with splat mask blending.

use bevy::asset::Handle;
use bevy::image::Image;
use bevy::pbr::{Material, MaterialPipeline};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

/// A terrain layer with texture and scaling.
#[derive(Debug, Clone)]
pub struct TerrainLayer {
    /// Albedo/diffuse texture.
    pub albedo: Handle<Image>,
    /// Normal map texture (optional).
    pub normal: Option<Handle<Image>>,
    /// UV scale for tiling.
    pub uv_scale: Vec2,
    /// Channel in the splat mask (0-3).
    pub channel: u8,
}

/// Splat texture containing blending weights for terrain layers.
///
/// RGBA channels correspond to 4 terrain layers.
/// Each channel value (0-255) represents the weight of that layer.
#[derive(Debug, Clone)]
pub struct SplatTexture {
    /// The splat mask image.
    pub image: Handle<Image>,
}

/// Terrain material with texture splatting support.
#[derive(Asset, TypePath, Debug, Clone, AsBindGroup)]
pub struct TerrainMaterial {
    /// Terrain texture layers (up to 4).
    #[texture(0)]
    #[sampler(1)]
    pub albedo_array: Option<Handle<Image>>,

    /// Splat mask for blending layers.
    #[texture(2)]
    #[sampler(3)]
    pub splat_mask: Option<Handle<Image>>,

    /// Normal maps array.
    #[texture(4)]
    #[sampler(5)]
    pub normal_array: Option<Handle<Image>>,

    /// UV scale for terrain tiling.
    #[uniform(6)]
    pub uv_scale: Vec4,

    /// Tint colors for each layer.
    #[uniform(7)]
    pub layer_tints: [Vec4; 4],

    /// Height-based blending factor (0 = splat only, 1 = height blend).
    #[uniform(8)]
    pub height_blend_factor: f32,
}

impl Default for TerrainMaterial {
    fn default() -> Self {
        Self {
            albedo_array: None,
            splat_mask: None,
            normal_array: None,
            uv_scale: Vec4::splat(10.0),
            layer_tints: [
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
            ],
            height_blend_factor: 0.0,
        }
    }
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn specialize(
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipeline<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

/// Builder for creating terrain materials.
#[derive(Debug, Clone, Default)]
pub struct TerrainMaterialBuilder {
    layers: Vec<TerrainLayer>,
    splat_handle: Option<Handle<Image>>,
    height_blend: f32,
}

impl TerrainMaterialBuilder {
    /// Create a new terrain material builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a terrain layer.
    pub fn with_layer(mut self, layer: TerrainLayer) -> Self {
        if self.layers.len() < 4 {
            self.layers.push(layer);
        }
        self
    }

    /// Set the splat mask texture.
    pub fn with_splat_mask(mut self, splat: Handle<Image>) -> Self {
        self.splat_handle = Some(splat);
        self
    }

    /// Set height-based blending factor.
    pub fn with_height_blend(mut self, factor: f32) -> Self {
        self.height_blend = factor.clamp(0.0, 1.0);
        self
    }

    /// Build the terrain material.
    ///
    /// Note: Actual texture array creation requires asset loading,
    /// so this returns a partial material that needs textures filled in.
    pub fn build(self) -> TerrainMaterial {
        let mut uv_scale = Vec4::ONE;
        let layer_tints = [Vec4::ONE; 4];

        for (i, layer) in self.layers.iter().enumerate() {
            if i < 4 {
                uv_scale[i] = layer.uv_scale.length();
            }
        }

        TerrainMaterial {
            albedo_array: None, // Would be created from layer textures
            splat_mask: self.splat_handle,
            normal_array: None,
            uv_scale,
            layer_tints,
            height_blend_factor: self.height_blend,
        }
    }
}

/// Preset terrain layers for common biome types.
pub mod presets {
    use super::*;

    /// Create a grass layer.
    pub fn grass(texture: Handle<Image>) -> TerrainLayer {
        TerrainLayer {
            albedo: texture,
            normal: None,
            uv_scale: Vec2::splat(10.0),
            channel: 0,
        }
    }

    /// Create a dirt layer.
    pub fn dirt(texture: Handle<Image>) -> TerrainLayer {
        TerrainLayer {
            albedo: texture,
            normal: None,
            uv_scale: Vec2::splat(5.0),
            channel: 1,
        }
    }

    /// Create a rock layer.
    pub fn rock(texture: Handle<Image>) -> TerrainLayer {
        TerrainLayer {
            albedo: texture,
            normal: None,
            uv_scale: Vec2::splat(3.0),
            channel: 2,
        }
    }

    /// Create a snow layer.
    pub fn snow(texture: Handle<Image>) -> TerrainLayer {
        TerrainLayer {
            albedo: texture,
            normal: None,
            uv_scale: Vec2::splat(8.0),
            channel: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_material_default() {
        let material = TerrainMaterial::default();
        assert!(material.albedo_array.is_none());
        assert!(material.splat_mask.is_none());
        assert_eq!(material.height_blend_factor, 0.0);
    }

    #[test]
    fn terrain_material_builder() {
        // Note: We can't test with real textures without asset server
        let material = TerrainMaterialBuilder::new().with_height_blend(0.5).build();

        assert_eq!(material.height_blend_factor, 0.5);
    }
}
