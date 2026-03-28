//! Particle effect configurations for the TET scene using `bevy_hanabi`.
//!
//! Provides factory functions that build [`EffectAsset`] instances for
//! agent particles and data stream trails.

use bevy::log::debug;
use bevy::math::{Vec3, Vec4};
use bevy_hanabi::prelude::*;

use crate::constants::visual::AGENT_PARTICLE_SIZE;

// ---------------------------------------------------------------------------
// Agent particles
// ---------------------------------------------------------------------------

/// Creates a particle effect for agents near the TET structure.
///
/// Particles spawn in a sphere, drift outward, and fade over their lifetime.
/// The particle size is controlled by [`AGENT_PARTICLE_SIZE`].
pub fn create_agent_particle_effect() -> EffectAsset {
    debug!("create_agent_particle_effect — building effect asset");

    let writer = ExprWriter::new();

    // Initialize position within a small sphere.
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Random outward velocity.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(3.0).uniform(writer.lit(8.0)).expr(),
    };

    // Lifetime 1–3 seconds.
    let lifetime = writer.lit(1.0).uniform(writer.lit(3.0)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let spawner = SpawnerSettings::rate(10.0_f32.into());

    // Color gradient: bright cyan → transparent.
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.3, 0.8, 1.0, 1.0));
    color_gradient.add_key(0.7, Vec4::new(0.2, 0.6, 1.0, 0.8));
    color_gradient.add_key(1.0, Vec4::new(0.1, 0.3, 0.8, 0.0));

    // Size gradient: grow then shrink.
    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(AGENT_PARTICLE_SIZE * 0.5));
    size_gradient.add_key(0.3, Vec3::splat(AGENT_PARTICLE_SIZE));
    size_gradient.add_key(1.0, Vec3::splat(AGENT_PARTICLE_SIZE * 0.1));

    let module = writer.finish();

    EffectAsset::new(512, spawner, module)
        .with_name("agent_particles")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_age)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
}

// ---------------------------------------------------------------------------
// Data stream trail
// ---------------------------------------------------------------------------

/// Creates a flowing data stream particle trail effect.
///
/// Particles emit in a directional cone, simulating data flowing
/// along network paths toward the TET structure.
pub fn create_data_stream_effect() -> EffectAsset {
    debug!("create_data_stream_effect — building effect asset");

    let writer = ExprWriter::new();

    // Initialize along a line (simplified cone via sphere).
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Directed velocity toward the TET center.
    let vel = writer.lit(Vec3::new(0.0, 0.0, -5.0)).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    // Short lifetime for trail effect.
    let lifetime = writer.lit(0.5).uniform(writer.lit(1.5)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Linear drag for deceleration.
    let drag = writer.lit(2.0).expr();
    let update_drag = LinearDragModifier::new(drag);

    let spawner = SpawnerSettings::rate(30.0_f32.into());

    // Green data-stream gradient.
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.0, 2.0, 0.5, 1.0));
    color_gradient.add_key(0.5, Vec4::new(0.0, 1.5, 0.4, 0.7));
    color_gradient.add_key(1.0, Vec4::new(0.0, 0.5, 0.2, 0.0));

    let module = writer.finish();

    EffectAsset::new(256, spawner, module)
        .with_name("data_stream")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_age)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec3::splat(0.15)),
            screen_space_size: false,
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_effect_has_capacity() {
        let effect = create_agent_particle_effect();
        assert_eq!(effect.capacity(), 512);
    }

    #[test]
    fn data_stream_effect_has_capacity() {
        let effect = create_data_stream_effect();
        assert_eq!(effect.capacity(), 256);
    }
}
