//! Render quality plugin — cinematic visuals configuration.
//!
//! Configures Bevy's rendering pipeline for maximum visual quality.

use bevy::log::{debug, info};
use bevy::prelude::*;

use super::atmosphere::AtmosphereConfig;
use super::lighting::CinematicLighting;
use super::post_processing::PostProcessingConfig;
use super::quality::QualitySettings;

/// Plugin for cinematic render quality.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(RenderQualityPlugin::default());
/// ```
#[derive(Default)]
pub struct RenderQualityPlugin {
    /// Override quality settings (None = use defaults).
    pub quality: Option<QualitySettings>,
}

impl Plugin for RenderQualityPlugin {
    fn build(&self, app: &mut App) {
        info!("RenderQualityPlugin: initializing cinematic render quality");

        // Add quality settings resource
        let quality = self
            .quality
            .clone()
            .unwrap_or_else(QualitySettings::cinematic);
        app.insert_resource(quality);

        // Add lighting config
        app.init_resource::<CinematicLighting>();
        app.init_resource::<PostProcessingConfig>();
        app.init_resource::<AtmosphereConfig>();

        // Add systems
        app.add_systems(Startup, setup_render_quality);
        app.add_systems(Update, update_lighting_system);

        info!("RenderQualityPlugin: registered systems");
    }
}

/// Setup render quality on startup.
fn setup_render_quality(
    mut commands: Commands,
    quality: Res<QualitySettings>,
    cameras: Query<(Entity, &Camera)>,
) {
    debug!("setup_render_quality: configuring render quality");

    // Log camera count
    let camera_count = cameras.iter().count();
    debug!("  - Found {} cameras", camera_count);

    // Spawn sun light
    spawn_sun_light(&mut commands, &quality);

    // Log fog status
    if quality.fog_enabled {
        debug!("  - Fog enabled with density {}", quality.fog_density);
    }

    info!("RenderQualityPlugin: render quality configured");
}

/// Spawn the main directional (sun) light with shadows.
fn spawn_sun_light(commands: &mut Commands, quality: &QualitySettings) {
    commands.spawn((
        Name::new("CinematicSunLight"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9), // Warm sunlight
            illuminance: 100_000.0,             // Bright sunlight
            shadows_enabled: quality.shadows_enabled,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        Visibility::default(),
    ));

    debug!("  - Spawned sun light with shadows");
}

/// Update lighting based on time of day (future integration).
fn update_lighting_system(
    time: Res<Time>,
    lighting: Res<CinematicLighting>,
    mut query: Query<(&mut DirectionalLight, &mut Transform)>,
) {
    if !lighting.auto_sun_position {
        return;
    }

    let elapsed = time.elapsed_secs();
    let day_time = (elapsed * 0.01) % 1.0; // Slow day cycle for demo

    // Calculate sun position based on time of day
    let sun_angle = day_time * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let sun_height = sun_angle.sin().max(0.0);
    let sun_x = sun_angle.cos() * 100.0;
    let sun_y = sun_height * 100.0;

    for (mut light, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(sun_x, sun_y.max(10.0), 50.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);

        // Adjust light intensity based on time
        let day_intensity = sun_height.powf(0.5);
        light.illuminance = 20_000.0 + 80_000.0 * day_intensity;

        // Warm colors at sunrise/sunset
        let warmth = (1.0 - sun_height).powf(2.0);
        light.color = Color::srgb(1.0, 0.95 - warmth * 0.2, 0.9 - warmth * 0.4);
    }
}
