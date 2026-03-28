//! Weather state machine — maps system health metrics to weather states.
//!
//! The [`WeatherMachine`] evaluates [`SystemHealth`] each frame and
//! drives transitions in [`AtmosphereConfig`]. Weather states are:
//! - `Clear` — all metrics healthy
//! - `Foggy` — elevated latency
//! - `Rainy` — high CPU load
//! - `Stormy` — critical error rates

use bevy::log::{debug, info, trace};
use bevy::prelude::*;

use crate::constants::timing::WEATHER_TRANSITION_SECS;
use crate::constants::weather::{FOGGY_LATENCY_MS, RAINY_LOAD_PCT, STORMY_ERROR_RATE};
use crate::world::state::SystemHealth;

use super::atmosphere::{AtmosphereConfig, AtmosphereState};

// ---------------------------------------------------------------------------
// Weather state
// ---------------------------------------------------------------------------

/// Weather states driven by system health metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeatherState {
    #[default]
    Clear,
    Foggy,
    Rainy,
    Stormy,
}

impl WeatherState {
    /// Maps a weather state to the corresponding atmosphere state.
    pub fn to_atmosphere(self) -> AtmosphereState {
        match self {
            Self::Clear => AtmosphereState::Clear,
            Self::Foggy => AtmosphereState::Foggy,
            Self::Rainy => AtmosphereState::Hazy,
            Self::Stormy => AtmosphereState::Stormy,
        }
    }
}

// ---------------------------------------------------------------------------
// Weather machine resource
// ---------------------------------------------------------------------------

/// State machine that evaluates system health and drives weather transitions.
#[derive(Resource, Debug)]
pub struct WeatherMachine {
    /// Currently active weather state.
    pub current: WeatherState,
    /// Target weather state being transitioned to.
    pub target: WeatherState,
    /// Transition progress in `[0.0, 1.0]`.
    pub transition_progress: f32,
}

impl Default for WeatherMachine {
    fn default() -> Self {
        trace!("WeatherMachine::default — Clear state");
        Self {
            current: WeatherState::Clear,
            target: WeatherState::Clear,
            transition_progress: 1.0,
        }
    }
}

impl WeatherMachine {
    /// Creates a new weather machine in the Clear state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates system health metrics and determines the target weather.
    ///
    /// Priority (highest wins): error rate → CPU load → latency → clear.
    /// This is a pure function with no side effects.
    pub fn evaluate_target(health: &SystemHealth) -> WeatherState {
        trace!(
            "WeatherMachine::evaluate_target — error={:.3} cpu={:.3} latency={:.1}ms",
            health.error_rate,
            health.cpu_load,
            health.avg_latency_ms
        );

        if health.error_rate >= STORMY_ERROR_RATE {
            return WeatherState::Stormy;
        }
        if health.cpu_load >= RAINY_LOAD_PCT {
            return WeatherState::Rainy;
        }
        if health.avg_latency_ms >= FOGGY_LATENCY_MS {
            return WeatherState::Foggy;
        }
        WeatherState::Clear
    }

    /// Advances the transition toward the target state.
    ///
    /// Returns `true` when the transition is complete.
    pub fn advance(&mut self, dt: f32) -> bool {
        if self.transition_progress >= 1.0 {
            return true;
        }

        let step = dt / WEATHER_TRANSITION_SECS;
        self.transition_progress = (self.transition_progress + step).min(1.0);

        trace!(
            "WeatherMachine::advance — progress={:.3}",
            self.transition_progress
        );

        if self.transition_progress >= 1.0 {
            self.current = self.target;
            debug!(
                "WeatherMachine::advance — transition complete → {:?}",
                self.current
            );
            true
        } else {
            false
        }
    }

    /// Sets a new target weather state and resets transition progress.
    fn set_target(&mut self, target: WeatherState) {
        if target != self.target {
            info!(
                "WeatherMachine — weather changing: {:?} → {:?}",
                self.target, target
            );
            self.current = self.target;
            self.target = target;
            self.transition_progress = 0.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Evaluates system health and updates the weather target.
pub fn weather_update_system(
    world_state: Res<crate::world::state::WorldState>,
    mut weather: ResMut<WeatherMachine>,
) {
    let target = WeatherMachine::evaluate_target(&world_state.system_health);
    weather.set_target(target);
}

/// Advances weather transitions and syncs to atmosphere config.
pub fn weather_transition_system(
    time: Res<Time>,
    mut weather: ResMut<WeatherMachine>,
    mut atmosphere: ResMut<AtmosphereConfig>,
) {
    weather.advance(time.delta_secs());

    // Sync weather target to atmosphere
    let atmo_target = weather.target.to_atmosphere();
    atmosphere.transition_to(atmo_target);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_clear_when_healthy() {
        let health = SystemHealth::default();
        assert_eq!(
            WeatherMachine::evaluate_target(&health),
            WeatherState::Clear
        );
    }

    #[test]
    fn evaluate_stormy_on_high_error_rate() {
        let health = SystemHealth {
            error_rate: 0.15,
            ..Default::default()
        };
        assert_eq!(
            WeatherMachine::evaluate_target(&health),
            WeatherState::Stormy
        );
    }

    #[test]
    fn evaluate_rainy_on_high_cpu() {
        let health = SystemHealth {
            cpu_load: 0.8,
            ..Default::default()
        };
        assert_eq!(
            WeatherMachine::evaluate_target(&health),
            WeatherState::Rainy
        );
    }

    #[test]
    fn evaluate_foggy_on_high_latency() {
        let health = SystemHealth {
            avg_latency_ms: 600.0,
            ..Default::default()
        };
        assert_eq!(
            WeatherMachine::evaluate_target(&health),
            WeatherState::Foggy
        );
    }

    #[test]
    fn stormy_takes_priority_over_rainy() {
        let health = SystemHealth {
            error_rate: 0.15,
            cpu_load: 0.9,
            ..Default::default()
        };
        assert_eq!(
            WeatherMachine::evaluate_target(&health),
            WeatherState::Stormy
        );
    }

    #[test]
    fn advance_completes_transition() {
        let mut machine = WeatherMachine::new();
        machine.set_target(WeatherState::Stormy);
        assert_eq!(machine.transition_progress, 0.0);

        // Advance past the full duration
        let completed = machine.advance(WEATHER_TRANSITION_SECS + 1.0);
        assert!(completed);
        assert_eq!(machine.current, WeatherState::Stormy);
        assert_eq!(machine.transition_progress, 1.0);
    }
}
