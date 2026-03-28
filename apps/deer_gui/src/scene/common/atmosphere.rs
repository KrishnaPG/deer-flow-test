//! Atmosphere system — maps system health to ambient lighting.
//!
//! The [`AtmosphereConfig`] resource tracks the current and target
//! atmosphere states, smoothly transitioning [`GlobalAmbientLight`]
//! brightness and color over [`WEATHER_TRANSITION_SECS`].

use bevy::color::Color;
use bevy::light::GlobalAmbientLight;
use bevy::log::{debug, trace};
use bevy::prelude::{Res, ResMut, Resource, Time};

use crate::constants::timing::WEATHER_TRANSITION_SECS;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Visual atmosphere state derived from system health metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AtmosphereState {
    /// Normal operation — bright, clear lighting.
    #[default]
    Clear,
    /// Elevated latency — slightly dimmed, warm haze.
    Hazy,
    /// High latency — fog-like reduced visibility.
    Foggy,
    /// Critical errors — dark, flashing storm lighting.
    Stormy,
}

impl AtmosphereState {
    /// Target brightness for this state (0.0–1.0 scale, mapped to lux).
    pub fn target_brightness(&self) -> f32 {
        match self {
            Self::Clear => 300.0,
            Self::Hazy => 200.0,
            Self::Foggy => 100.0,
            Self::Stormy => 50.0,
        }
    }

    /// Target ambient color for this state.
    pub fn target_color(&self) -> Color {
        match self {
            Self::Clear => Color::WHITE,
            Self::Hazy => Color::srgb(1.0, 0.95, 0.85),
            Self::Foggy => Color::srgb(0.7, 0.75, 0.85),
            Self::Stormy => Color::srgb(0.4, 0.35, 0.5),
        }
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks the current and target atmosphere, with transition progress.
#[derive(Resource, Debug)]
pub struct AtmosphereConfig {
    /// Currently rendering atmosphere.
    pub current: AtmosphereState,
    /// Target atmosphere to transition toward.
    pub target: AtmosphereState,
    /// Transition progress in `[0.0, 1.0]`.
    pub progress: f32,
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        trace!("AtmosphereConfig::default — Clear state");
        Self {
            current: AtmosphereState::Clear,
            target: AtmosphereState::Clear,
            progress: 1.0, // fully settled
        }
    }
}

impl AtmosphereConfig {
    /// Begin a transition to a new atmosphere state.
    pub fn transition_to(&mut self, state: AtmosphereState) {
        if state != self.target {
            debug!(
                "AtmosphereConfig::transition_to — {:?} → {:?}",
                self.target, state
            );
            self.current = self.target;
            self.target = state;
            self.progress = 0.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Smoothly interpolates [`GlobalAmbientLight`] between atmosphere states.
///
/// Reads [`AtmosphereConfig`] to determine the source/target brightness
/// and color, then advances `progress` based on elapsed time and
/// [`WEATHER_TRANSITION_SECS`].
pub fn atmosphere_transition_system(
    time: Res<Time>,
    mut config: ResMut<AtmosphereConfig>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    // Already at target — nothing to interpolate.
    if config.progress >= 1.0 {
        return;
    }

    let dt = time.delta_secs();
    let step = dt / WEATHER_TRANSITION_SECS;
    config.progress = (config.progress + step).min(1.0);

    let t = config.progress;

    // Interpolate brightness.
    let src_brightness = config.current.target_brightness();
    let dst_brightness = config.target.target_brightness();
    ambient.brightness = src_brightness + (dst_brightness - src_brightness) * t;

    // Interpolate color (linear in sRGB — acceptable for ambient).
    let src = config.current.target_color();
    let dst = config.target.target_color();
    ambient.color = lerp_color(src, dst, t);

    trace!(
        "atmosphere_transition: {:?}→{:?} progress={:.3} brightness={:.1}",
        config.current,
        config.target,
        t,
        ambient.brightness,
    );

    // Snap to target when done.
    if config.progress >= 1.0 {
        config.current = config.target;
        debug!("atmosphere_transition: completed → {:?}", config.current);
    }
}

/// Linearly interpolate between two sRGB colors.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atmosphere_brightness_ordering() {
        assert!(
            AtmosphereState::Clear.target_brightness()
                > AtmosphereState::Stormy.target_brightness()
        );
    }

    #[test]
    fn transition_resets_progress() {
        let mut config = AtmosphereConfig::default();
        config.transition_to(AtmosphereState::Foggy);
        assert_eq!(config.progress, 0.0);
        assert_eq!(config.target, AtmosphereState::Foggy);
    }

    #[test]
    fn no_op_transition_to_same_state() {
        let mut config = AtmosphereConfig::default();
        config.transition_to(AtmosphereState::Clear);
        assert_eq!(config.progress, 1.0); // unchanged
    }
}
