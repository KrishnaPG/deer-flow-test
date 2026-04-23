//! Automatic quality detection based on hardware capabilities.
//!
//! Profiles the GPU and system to recommend appropriate quality settings.

use bevy::log::{info, warn};
use bevy::prelude::*;

use super::quality::{QualityPreset, QualitySettings};

/// GPU vendor identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Unknown,
}

/// GPU tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuTier {
    /// Integrated graphics (Intel UHD, AMD Vega).
    Integrated = 0,
    /// Entry-level discrete (GTX 1650, RX 5500 XT).
    Entry = 1,
    /// Mid-range (RTX 3060, RX 6700 XT).
    MidRange = 2,
    /// High-end (RTX 3080, RX 6800 XT).
    HighEnd = 3,
    /// Enthusiast (RTX 4080+, RX 7900 XT+).
    Enthusiast = 4,
}

/// System memory tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    Low,    // < 8 GB
    Medium, // 8-16 GB
    High,   // 16-32 GB
    Ultra,  // > 32 GB
}

/// Hardware profile for quality selection.
#[derive(Debug, Clone)]
pub struct HardwareProfile {
    /// Detected GPU vendor.
    pub gpu_vendor: GpuVendor,
    /// GPU tier classification.
    pub gpu_tier: GpuTier,
    /// GPU name string.
    pub gpu_name: String,
    /// Estimated VRAM in GB.
    pub vram_gb: f32,
    /// System memory tier.
    pub memory_tier: MemoryTier,
    /// Whether discrete GPU is present.
    pub is_discrete: bool,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            gpu_vendor: GpuVendor::Unknown,
            gpu_tier: GpuTier::MidRange,
            gpu_name: "Unknown".to_string(),
            vram_gb: 4.0,
            memory_tier: MemoryTier::Medium,
            is_discrete: true,
        }
    }
}

impl HardwareProfile {
    /// Create hardware profile from GPU name.
    pub fn from_name(name: &str) -> Self {
        let vendor = detect_vendor(name);
        let vram_gb = estimate_vram(name);
        let tier = classify_gpu(name, vram_gb);
        let is_discrete = tier >= GpuTier::Entry;

        info!(
            "HardwareProfile: GPU='{}' vendor={:?} tier={:?} vram={}GB",
            name, vendor, tier, vram_gb
        );

        Self {
            gpu_vendor: vendor,
            gpu_tier: tier,
            gpu_name: name.to_string(),
            vram_gb,
            memory_tier: MemoryTier::Medium, // Default, would need system API
            is_discrete,
        }
    }

    /// Get recommended quality preset based on hardware.
    pub fn recommended_preset(&self) -> QualityPreset {
        match self.gpu_tier {
            GpuTier::Integrated => QualityPreset::Low,
            GpuTier::Entry => QualityPreset::Medium,
            GpuTier::MidRange => QualityPreset::High,
            GpuTier::HighEnd => QualityPreset::Ultra,
            GpuTier::Enthusiast => QualityPreset::Cinematic,
        }
    }

    /// Apply hardware-specific optimizations to settings.
    pub fn optimize_settings(&self, settings: &mut QualitySettings) {
        // Reduce shadow quality on integrated GPUs
        if self.gpu_tier == GpuTier::Integrated {
            settings.shadows_enabled = false;
            settings.ssao_enabled = false;
            settings.bloom_enabled = false;
        }

        // Adjust texture quality based on VRAM
        if self.vram_gb < 4.0 {
            settings.texture_quality = 0;
            settings.anisotropic_filtering = 1;
        } else if self.vram_gb < 6.0 {
            settings.texture_quality = 1;
            settings.anisotropic_filtering = 4;
        }

        // Apple Silicon specific optimizations
        if self.gpu_vendor == GpuVendor::Apple {
            // Apple GPUs are efficient but benefit from specific settings
            settings.antialiasing = super::quality::AntiAliasing::Taa;
        }
    }
}

/// Detect GPU vendor from name string.
fn detect_vendor(name: &str) -> GpuVendor {
    let name_lower = name.to_lowercase();

    if name_lower.contains("nvidia")
        || name_lower.contains("geforce")
        || name_lower.contains("rtx")
        || name_lower.contains("gtx")
    {
        GpuVendor::Nvidia
    } else if name_lower.contains("amd")
        || name_lower.contains("radeon")
        || name_lower.contains("rx")
    {
        GpuVendor::Amd
    } else if name_lower.contains("intel")
        || name_lower.contains("uhd")
        || name_lower.contains("iris")
    {
        GpuVendor::Intel
    } else if name_lower.contains("apple")
        || name_lower.contains("m1")
        || name_lower.contains("m2")
        || name_lower.contains("m3")
    {
        GpuVendor::Apple
    } else if name_lower.contains("adreno") || name_lower.contains("qualcomm") {
        GpuVendor::Qualcomm
    } else {
        GpuVendor::Unknown
    }
}

/// Estimate VRAM based on GPU name (heuristic).
fn estimate_vram(name: &str) -> f32 {
    let name_lower = name.to_lowercase();

    // NVIDIA RTX 40 series
    if name_lower.contains("4090") {
        return 24.0;
    }
    if name_lower.contains("4080") {
        return 16.0;
    }
    if name_lower.contains("4070") {
        return 12.0;
    }
    if name_lower.contains("4060") {
        return 8.0;
    }

    // NVIDIA RTX 30 series
    if name_lower.contains("3090") {
        return 24.0;
    }
    if name_lower.contains("3080") {
        return 10.0;
    }
    if name_lower.contains("3070") {
        return 8.0;
    }
    if name_lower.contains("3060") {
        return 12.0;
    }

    // NVIDIA RTX 20 series
    if name_lower.contains("2080") {
        return 8.0;
    }
    if name_lower.contains("2070") {
        return 8.0;
    }
    if name_lower.contains("2060") {
        return 6.0;
    }

    // NVIDIA GTX 16 series
    if name_lower.contains("1660") {
        return 6.0;
    }
    if name_lower.contains("1650") {
        return 4.0;
    }

    // AMD RX 7000 series
    if name_lower.contains("7900") {
        return 20.0;
    }
    if name_lower.contains("7800") {
        return 16.0;
    }
    if name_lower.contains("7700") {
        return 12.0;
    }
    if name_lower.contains("7600") {
        return 8.0;
    }

    // AMD RX 6000 series
    if name_lower.contains("6900") {
        return 16.0;
    }
    if name_lower.contains("6800") {
        return 16.0;
    }
    if name_lower.contains("6700") {
        return 12.0;
    }
    if name_lower.contains("6600") {
        return 8.0;
    }

    // Intel integrated
    if name_lower.contains("intel") {
        return 1.5;
    }

    // Apple Silicon
    if name_lower.contains("m1") || name_lower.contains("m2") {
        return 16.0;
    } // Unified memory

    // Default estimate
    4.0
}

/// Classify GPU into tier based on name and VRAM.
fn classify_gpu(name: &str, vram_gb: f32) -> GpuTier {
    let name_lower = name.to_lowercase();

    // Enthusiast tier
    if name_lower.contains("4090")
        || name_lower.contains("4080")
        || name_lower.contains("7900")
        || vram_gb >= 16.0
    {
        return GpuTier::Enthusiast;
    }

    // High-end tier
    if name_lower.contains("3080")
        || name_lower.contains("3090")
        || name_lower.contains("4070")
        || name_lower.contains("6800")
        || name_lower.contains("6900")
        || vram_gb >= 10.0
    {
        return GpuTier::HighEnd;
    }

    // Mid-range tier
    if name_lower.contains("3060")
        || name_lower.contains("3070")
        || name_lower.contains("2070")
        || name_lower.contains("2080")
        || name_lower.contains("6700")
        || name_lower.contains("7600")
        || name_lower.contains("m1")
        || name_lower.contains("m2")
        || vram_gb >= 6.0
    {
        return GpuTier::MidRange;
    }

    // Entry tier
    if name_lower.contains("1660")
        || name_lower.contains("1650")
        || name_lower.contains("2060")
        || name_lower.contains("6600")
        || name_lower.contains("580")
        || vram_gb >= 4.0
    {
        return GpuTier::Entry;
    }

    // Integrated
    GpuTier::Integrated
}

/// Plugin for automatic quality detection.
pub struct AutoQualityPlugin;

impl Plugin for AutoQualityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, detect_and_apply_quality);
    }
}

/// Detect hardware and apply appropriate quality settings.
fn detect_and_apply_quality(
    adapter_info: Option<Res<bevy::render::renderer::RenderAdapterInfo>>,
    mut quality: ResMut<QualitySettings>,
) {
    if let Some(info) = adapter_info {
        let name = info.0.name.clone();
        let profile = HardwareProfile::from_name(&name);
        let recommended = profile.recommended_preset();

        info!(
            "AutoQuality: Detected {:?} GPU '{}', recommending {:?} quality",
            profile.gpu_tier, profile.gpu_name, recommended
        );

        // Apply recommended settings
        *quality = QualitySettings::from_preset(recommended);

        // Apply hardware-specific optimizations
        profile.optimize_settings(&mut quality);
    } else {
        warn!("AutoQuality: No adapter info found, using default High quality");
    }
}

/// Quality settings UI widget.
pub mod ui {
    use super::super::quality::{QualityPreset, QualitySettings};
    use bevy_egui::egui;

    /// Quality settings selector widget.
    pub fn quality_selector_ui(ui: &mut egui::Ui, quality: &mut QualitySettings) {
        ui.heading("Graphics Quality");
        ui.separator();

        // Preset selector
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
                }
            }
        });

        ui.separator();

        // Detailed settings
        egui::CollapsingHeader::new("Shadows")
            .id_salt("shadows")
            .show(ui, |ui| {
                ui.checkbox(&mut quality.shadows_enabled, "Enable Shadows");
                ui.add(
                    egui::Slider::new(&mut quality.shadow_cascades, 1..=4).text("Shadow Cascades"),
                );
                ui.add(
                    egui::Slider::new(&mut quality.shadow_map_resolution, 256..=4096)
                        .text("Shadow Resolution"),
                );
            });

        egui::CollapsingHeader::new("Post-Processing")
            .id_salt("post_process")
            .show(ui, |ui| {
                ui.checkbox(&mut quality.ssao_enabled, "Ambient Occlusion (SSAO)");
                ui.checkbox(&mut quality.bloom_enabled, "Bloom");
                ui.checkbox(&mut quality.fog_enabled, "Atmospheric Fog");
            });

        egui::CollapsingHeader::new("Vegetation")
            .id_salt("vegetation")
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut quality.vegetation_draw_distance, 50.0..=1000.0)
                        .text("Draw Distance"),
                );
                ui.add(
                    egui::Slider::new(&mut quality.vegetation_density, 0.1..=2.0).text("Density"),
                );
            });

        egui::CollapsingHeader::new("Textures")
            .id_salt("textures")
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut quality.anisotropic_filtering, 1..=16)
                        .text("Anisotropic Filtering"),
                );
                ui.checkbox(&mut quality.normal_maps_enabled, "Normal Maps");
                ui.checkbox(&mut quality.parallax_mapping_enabled, "Parallax Mapping");
            });
    }
}
