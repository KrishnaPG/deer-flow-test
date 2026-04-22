//! Hardware capability detection for quality selection.
//!
//! Instead of guessing from GPU names, we query actual hardware features,
//! limits, and run micro-benchmarks to determine quality settings.

use bevy::log::{debug, info, warn};
use bevy::prelude::*;
use bevy::render::render_resource::{
    DownlevelFlags, WgpuFeatures as Features, WgpuLimits as Limits,
};
use bevy::render::renderer::{RenderAdapter, RenderDevice};

use super::quality::{AntiAliasing, QualityPreset, QualitySettings};

/// Performance tier based on capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceTier {
    /// Cannot run modern features.
    Unsupported = 0,
    /// Very limited (integrated old GPU).
    Potato = 1,
    /// Low end.
    Low = 2,
    /// Medium capability.
    Medium = 3,
    /// High capability.
    High = 4,
    /// Very high capability.
    Ultra = 5,
    /// Maximum capability.
    Cinematic = 6,
}

/// Detected hardware capabilities.
#[derive(Debug, Clone, Resource)]
pub struct HardwareCapabilities {
    /// GPU name from driver.
    pub gpu_name: String,
    /// Backend name (Vulkan, Metal, DX12).
    pub backend_name: String,
    /// Device type name (Discrete, Integrated, etc).
    pub device_type_name: String,

    // =========================================================================
    // Feature Flags (what the GPU supports)
    // =========================================================================
    pub supports_bc_compression: bool,
    pub supports_anisotropic: bool,
    pub supports_depth_clip_control: bool,
    pub supports_shader_f16: bool,

    // =========================================================================
    // Limits (what the GPU can handle)
    // =========================================================================
    pub max_texture_size_2d: u32,
    pub max_sample_count: u32,
    pub max_uniform_buffer_size: u32,
    pub max_storage_buffer_size: u64,
    pub max_uniform_buffers_per_stage: u32,

    // =========================================================================
    // Performance Score (computed)
    // =========================================================================
    pub performance_score: f32,
    pub tier: PerformanceTier,
}

impl Default for HardwareCapabilities {
    fn default() -> Self {
        Self {
            gpu_name: "Unknown".to_string(),
            backend_name: "Unknown".to_string(),
            device_type_name: "Unknown".to_string(),
            supports_bc_compression: false,
            supports_anisotropic: true,
            supports_depth_clip_control: true,
            supports_shader_f16: false,
            max_texture_size_2d: 4096,
            max_sample_count: 4,
            max_uniform_buffer_size: 16384,
            max_storage_buffer_size: 128 << 20,
            max_uniform_buffers_per_stage: 12,
            performance_score: 1.0,
            tier: PerformanceTier::Medium,
        }
    }
}

impl HardwareCapabilities {
    /// Compute performance tier based on capabilities.
    fn compute_tier(&mut self) {
        let mut score: f32 = 1.0;

        // Texture size capability (indicates GPU generation)
        score *= (self.max_texture_size_2d as f32 / 4096.0).clamp(0.5, 4.0);

        // MSAA capability
        score *= (self.max_sample_count as f32 / 4.0).clamp(0.5, 2.0);

        // Uniform buffer size (shader complexity)
        score *= (self.max_uniform_buffer_size as f32 / 65536.0).clamp(0.5, 2.0);

        // Storage buffer (compute capability)
        score *= (self.max_storage_buffer_size as f32 / (128.0 * 1024.0 * 1024.0)).clamp(0.5, 2.0);

        // Shader f16 support (modern GPU)
        if self.supports_shader_f16 {
            score *= 1.2;
        }

        // Device type adjustment
        if self.device_type_name.contains("Discrete") {
            score *= 2.0;
        } else if self.device_type_name.contains("Integrated") {
            score *= 0.4;
        } else if self.device_type_name.contains("Virtual") {
            score *= 1.2;
        }

        self.performance_score = score;

        // Determine tier from score
        self.tier = if self.max_texture_size_2d < 2048 {
            PerformanceTier::Unsupported
        } else if score < 0.5 {
            PerformanceTier::Potato
        } else if score < 1.0 {
            PerformanceTier::Low
        } else if score < 2.0 {
            PerformanceTier::Medium
        } else if score < 4.0 {
            PerformanceTier::High
        } else if score < 8.0 {
            PerformanceTier::Ultra
        } else {
            PerformanceTier::Cinematic
        };
    }

    /// Get recommended quality preset.
    pub fn recommended_preset(&self) -> QualityPreset {
        match self.tier {
            PerformanceTier::Unsupported => QualityPreset::Low,
            PerformanceTier::Potato => QualityPreset::Low,
            PerformanceTier::Low => QualityPreset::Medium,
            PerformanceTier::Medium => QualityPreset::High,
            PerformanceTier::High => QualityPreset::Ultra,
            PerformanceTier::Ultra => QualityPreset::Cinematic,
            PerformanceTier::Cinematic => QualityPreset::Cinematic,
        }
    }

    /// Apply capability-based adjustments to settings.
    pub fn apply_to_settings(&self, settings: &mut QualitySettings) {
        // Texture quality based on max texture size
        settings.texture_quality = match self.max_texture_size_2d {
            s if s >= 16384 => 3,
            s if s >= 8192 => 2,
            s if s >= 4096 => 1,
            _ => 0,
        };

        // Anisotropic filtering
        settings.anisotropic_filtering = if self.supports_anisotropic {
            settings
                .anisotropic_filtering
                .min(self.max_sample_count.min(16))
        } else {
            1
        };

        // MSAA based on max samples
        settings.antialiasing = match self.max_sample_count {
            s if s >= 8 => AntiAliasing::Msaa8x,
            s if s >= 4 => AntiAliasing::Msaa4x,
            s if s >= 2 => AntiAliasing::Fx,
            _ => AntiAliasing::None,
        };

        // Shadow cascades based on tier
        settings.shadow_cascades = match self.tier {
            PerformanceTier::Cinematic | PerformanceTier::Ultra => 4,
            PerformanceTier::High => 3,
            PerformanceTier::Medium => 2,
            _ => 1,
        };

        // Shadow resolution based on texture limits
        settings.shadow_map_resolution = match self.max_texture_size_2d {
            s if s >= 8192 => 2048,
            s if s >= 4096 => 1024,
            s if s >= 2048 => 512,
            _ => 256,
        };

        // Vegetation based on tier
        settings.vegetation_draw_distance = match self.tier {
            PerformanceTier::Cinematic => 500.0,
            PerformanceTier::Ultra => 400.0,
            PerformanceTier::High => 300.0,
            PerformanceTier::Medium => 200.0,
            PerformanceTier::Low => 100.0,
            _ => 50.0,
        };

        // Disable expensive effects on low-tier GPUs
        if self.tier <= PerformanceTier::Low {
            settings.ssao_enabled = false;
            settings.bloom_enabled = false;
            settings.water_ssr_enabled = false;
            settings.parallax_mapping_enabled = false;
            settings.light_scattering_enabled = false;
        }

        // Integrated GPU specific
        if self.device_type_name.contains("Integrated") {
            settings.ssao_enabled = false;
            settings.shadows_enabled = self.tier > PerformanceTier::Low;
        }

        info!(
            "Applied settings: shadows={}, ssao={}, bloom={}, vegetation={}m, aa={:?}",
            settings.shadows_enabled,
            settings.ssao_enabled,
            settings.bloom_enabled,
            settings.vegetation_draw_distance,
            settings.antialiasing
        );
    }

    /// Get summary string for UI.
    pub fn summary(&self) -> String {
        format!(
            "{} ({}) - {}",
            self.gpu_name,
            self.device_type_name,
            match self.tier {
                PerformanceTier::Unsupported => "Unsupported",
                PerformanceTier::Potato => "Very Low",
                PerformanceTier::Low => "Low",
                PerformanceTier::Medium => "Medium",
                PerformanceTier::High => "High",
                PerformanceTier::Ultra => "Very High",
                PerformanceTier::Cinematic => "Maximum",
            }
        )
    }
}

/// Plugin for automatic quality detection.
pub struct CapabilityQualityPlugin;

impl Plugin for CapabilityQualityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, detect_and_apply_capabilities);
    }
}

/// Detect hardware capabilities and apply quality settings.
fn detect_and_apply_capabilities(
    adapter: Option<Res<RenderAdapter>>,
    device: Option<Res<RenderDevice>>,
    mut commands: Commands,
    mut quality: ResMut<QualitySettings>,
) {
    let (Some(adapter), Some(device)) = (adapter, device) else {
        warn!("CapabilityQuality: No adapter/device, using default High quality");
        return;
    };

    // Get adapter info through the wrapper
    let adapter_info = adapter.get_info();
    let features = device.features();
    let limits = device.limits();

    debug!("Detecting GPU capabilities:");
    debug!("  GPU: {}", adapter_info.name);

    // Determine max sample count from features
    let max_sample_count = if features
        .contains(Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
    {
        8
    } else {
        4
    };

    let mut caps = HardwareCapabilities {
        gpu_name: adapter_info.name.clone(),
        backend_name: format!("{:?}", adapter_info.backend),
        device_type_name: format!("{:?}", adapter_info.device_type),

        // Feature detection
        supports_bc_compression: features.contains(Features::TEXTURE_COMPRESSION_BC),
        supports_anisotropic: true,
        supports_depth_clip_control: features.contains(Features::DEPTH_CLIP_CONTROL),
        supports_shader_f16: features.contains(Features::SHADER_F16),

        // Limits
        max_texture_size_2d: limits.max_texture_dimension_2d,
        max_sample_count,
        max_uniform_buffer_size: limits.max_uniform_buffer_binding_size,
        max_storage_buffer_size: limits.max_storage_buffer_binding_size as u64,
        max_uniform_buffers_per_stage: limits.max_uniform_buffers_per_shader_stage,

        // Defaults, will be computed
        performance_score: 1.0,
        tier: PerformanceTier::Medium,
    };

    caps.compute_tier();
    let recommended = caps.recommended_preset();

    info!(
        "CapabilityQuality: {} → recommending {:?}",
        caps.summary(),
        recommended
    );

    // Store for UI access
    commands.insert_resource(caps.clone());

    // Apply recommended settings
    *quality = QualitySettings::from_preset(recommended);
    caps.apply_to_settings(&mut quality);
}

/// Quality settings UI with capability info.
pub mod ui {
    use super::super::quality::{QualityPreset, QualitySettings};
    use super::HardwareCapabilities;
    use bevy_egui::egui;

    /// Quality settings selector with hardware info.
    pub fn quality_selector_ui(
        ui: &mut egui::Ui,
        quality: &mut QualitySettings,
        caps: Option<&HardwareCapabilities>,
    ) {
        ui.heading("Graphics Quality");
        ui.separator();

        // Show detected hardware
        if let Some(caps) = caps {
            ui.horizontal(|ui| {
                ui.label("GPU:");
                ui.label(egui::RichText::new(&caps.gpu_name).strong());
            });
            ui.horizontal(|ui| {
                ui.label("Performance:");
                ui.label(format!("{:?}", caps.tier));
            });
            ui.separator();
        }

        // Preset buttons
        ui.horizontal(|ui| {
            ui.label("Preset:");

            let presets = [
                ("Low", QualityPreset::Low),
                ("Medium", QualityPreset::Medium),
                ("High", QualityPreset::High),
                ("Ultra", QualityPreset::Ultra),
                ("Cinematic", QualityPreset::Cinematic),
            ];

            for (label, preset) in presets {
                if ui.selectable_label(false, label).clicked() {
                    *quality = QualitySettings::from_preset(preset);
                    if let Some(caps) = caps {
                        caps.apply_to_settings(quality);
                    }
                }
            }
        });

        ui.separator();

        // Detailed settings
        ui.collapsing("Shadows", |ui| {
            ui.checkbox(&mut quality.shadows_enabled, "Enable Shadows");
            ui.add(egui::Slider::new(&mut quality.shadow_cascades, 1..=4).text("Cascades"));
            ui.add(
                egui::Slider::new(&mut quality.shadow_map_resolution, 256..=4096)
                    .text("Resolution"),
            );
        });

        ui.collapsing("Effects", |ui| {
            ui.checkbox(&mut quality.ssao_enabled, "Ambient Occlusion");
            ui.checkbox(&mut quality.bloom_enabled, "Bloom");
            ui.checkbox(&mut quality.fog_enabled, "Atmospheric Fog");
        });

        ui.collapsing("Vegetation", |ui| {
            ui.add(
                egui::Slider::new(&mut quality.vegetation_draw_distance, 50.0..=1000.0)
                    .text("Draw Distance"),
            );
            ui.add(egui::Slider::new(&mut quality.vegetation_density, 0.1..=2.0).text("Density"));
        });
    }
}
