//! Wave animation for water surfaces.
//!
//! Provides configurable wave parameters including simple sine waves
//! and Gerstner waves for more realistic ocean simulation.

/// Type of wave function.
#[derive(Debug, Clone, Copy)]
pub enum WaveType {
    /// Simple sine wave.
    Sine,
    /// Gerstner wave (creates sharper peaks).
    Gerstner,
}

/// Parameters for a single wave.
#[derive(Debug, Clone)]
pub struct WaveParams {
    /// Wave type.
    pub wave_type: WaveType,
    /// Wave amplitude (height).
    pub amplitude: f32,
    /// Wave wavelength (distance between peaks).
    pub wavelength: f32,
    /// Wave speed (units per second).
    pub speed: f32,
    /// Wave direction (x, z) - will be normalized.
    pub direction: (f32, f32),
    /// Sharpness parameter for Gerstner waves (0.0 - 1.0).
    pub sharpness: f32,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            wave_type: WaveType::Sine,
            amplitude: 0.5,
            wavelength: 10.0,
            speed: 2.0,
            direction: (1.0, 0.0),
            sharpness: 0.5,
        }
    }
}

impl WaveParams {
    /// Create a new sine wave.
    pub fn sine(amplitude: f32, wavelength: f32, speed: f32, direction: (f32, f32)) -> Self {
        Self {
            wave_type: WaveType::Sine,
            amplitude,
            wavelength,
            speed,
            direction,
            sharpness: 0.0,
        }
    }

    /// Create a new Gerstner wave.
    pub fn gerstner(
        amplitude: f32,
        wavelength: f32,
        speed: f32,
        direction: (f32, f32),
        sharpness: f32,
    ) -> Self {
        Self {
            wave_type: WaveType::Gerstner,
            amplitude,
            wavelength,
            speed,
            direction,
            sharpness: sharpness.clamp(0.0, 1.0),
        }
    }

    /// Calculate wave displacement and normal at a position and time.
    ///
    /// Returns (displacement_y, [normal_x, normal_y, normal_z]).
    pub fn calculate(&self, x: f32, z: f32, time: f32) -> (f32, [f32; 3]) {
        // Normalize direction
        let len =
            (self.direction.0 * self.direction.0 + self.direction.1 * self.direction.1).sqrt();
        let dx = self.direction.0 / len;
        let dz = self.direction.1 / len;

        // Phase: dot product with direction, plus time offset
        let phase =
            (x * dx + z * dz) * (2.0 * std::f32::consts::PI / self.wavelength) + time * self.speed;

        match self.wave_type {
            WaveType::Sine => {
                let displacement = self.amplitude * phase.sin();

                // Normal from partial derivatives
                let d_dx = self.amplitude
                    * (2.0 * std::f32::consts::PI / self.wavelength)
                    * phase.cos()
                    * dx;
                let d_dz = self.amplitude
                    * (2.0 * std::f32::consts::PI / self.wavelength)
                    * phase.cos()
                    * dz;

                // Normal is perpendicular to tangent
                let normal = normalize_normal([-d_dx, 1.0, -d_dz]);

                (displacement, normal)
            }
            WaveType::Gerstner => {
                // Gerstner wave: sharper peaks, broader troughs
                let steepness = self.sharpness * self.amplitude;
                let k = 2.0 * std::f32::consts::PI / self.wavelength;

                let a = steepness / k;
                let displacement = self.amplitude * phase.cos();

                // Gerstner displacement in XZ plane
                // Note: px, pz are used for position displacement but we only return y displacement
                let _px = x - dx * a * phase.sin();
                let _pz = z - dz * a * phase.sin();

                // Normal calculation
                let d_dx = -dx * steepness * phase.sin();
                let d_dz = -dz * steepness * phase.sin();

                let normal = normalize_normal([d_dx, 1.0 - steepness * phase.cos(), d_dz]);

                (displacement, normal)
            }
        }
    }
}

/// Normalize a normal vector.
fn normalize_normal(n: [f32; 3]) -> [f32; 3] {
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Configuration for multiple waves.
#[derive(Debug, Clone)]
pub struct WaveConfig {
    /// List of wave parameters (waves are summed).
    pub waves: Vec<WaveParams>,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            waves: vec![
                WaveParams {
                    amplitude: 0.3,
                    wavelength: 20.0,
                    speed: 1.5,
                    direction: (1.0, 0.3),
                    ..Default::default()
                },
                WaveParams {
                    amplitude: 0.15,
                    wavelength: 10.0,
                    speed: 2.5,
                    direction: (0.7, 0.7),
                    ..Default::default()
                },
                WaveParams {
                    amplitude: 0.08,
                    wavelength: 5.0,
                    speed: 3.0,
                    direction: (0.0, 1.0),
                    ..Default::default()
                },
            ],
        }
    }
}

impl WaveConfig {
    /// Create a wave config with no waves (flat water).
    pub fn none() -> Self {
        Self { waves: vec![] }
    }

    /// Create a simple single-wave config.
    pub fn simple(amplitude: f32, wavelength: f32, speed: f32) -> Self {
        Self {
            waves: vec![WaveParams::sine(amplitude, wavelength, speed, (1.0, 0.0))],
        }
    }

    /// Calculate total wave displacement and averaged normal.
    ///
    /// Returns (displacement_y, [normal_x, normal_y, normal_z]).
    pub fn calculate(&self, x: f32, z: f32, time: f32) -> (f32, [f32; 3]) {
        if self.waves.is_empty() {
            return (0.0, [0.0, 1.0, 0.0]);
        }

        let mut total_displacement = 0.0;
        let mut normal_sum = [0.0, 0.0, 0.0];

        for wave in &self.waves {
            let (disp, normal) = wave.calculate(x, z, time);
            total_displacement += disp;
            normal_sum[0] += normal[0];
            normal_sum[1] += normal[1];
            normal_sum[2] += normal[2];
        }

        // Average and normalize the combined normal
        let normal = normalize_normal(normal_sum);

        (total_displacement, normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_params_default() {
        let params = WaveParams::default();
        assert_eq!(params.amplitude, 0.5);
        assert_eq!(params.wavelength, 10.0);
    }

    #[test]
    fn sine_wave_displacement() {
        let wave = WaveParams::sine(1.0, 10.0, 1.0, (1.0, 0.0));

        // At phase = 0 (x=0, t=0), displacement should be 0
        let (disp, _) = wave.calculate(0.0, 0.0, 0.0);
        assert!(disp.abs() < 1e-6);

        // At phase = PI/2 (quarter wavelength), displacement should be amplitude
        let (disp, _) = wave.calculate(2.5, 0.0, 0.0); // wavelength/4
        assert!((disp - 1.0).abs() < 0.1);
    }

    #[test]
    fn wave_normal_points_up() {
        let wave = WaveParams::sine(0.5, 10.0, 1.0, (1.0, 0.0));
        let (_, normal) = wave.calculate(5.0, 5.0, 0.0);

        // Normal y component should be positive
        assert!(normal[1] > 0.0);

        // Normal should be unit length
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn wave_config_none() {
        let config = WaveConfig::none();
        let (disp, normal) = config.calculate(0.0, 0.0, 0.0);

        assert_eq!(disp, 0.0);
        assert_eq!(normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn wave_config_simple() {
        let config = WaveConfig::simple(0.5, 10.0, 2.0);
        assert_eq!(config.waves.len(), 1);
    }

    #[test]
    fn wave_config_sum() {
        let config = WaveConfig::default();
        let (disp, _) = config.calculate(0.0, 0.0, 0.0);

        // Multiple waves should sum up
        // At t=0, x=0, all sine waves start at 0
        assert!(disp.abs() < 1e-6);
    }

    #[test]
    fn gerstner_wave_sharper() {
        let sine = WaveParams::sine(1.0, 10.0, 1.0, (1.0, 0.0));
        let gerstner = WaveParams::gerstner(1.0, 10.0, 1.0, (1.0, 0.0), 0.8);

        // Gerstner uses cosine for displacement, so it's at amplitude at t=0, x=0
        let (sine_disp, _) = sine.calculate(0.0, 0.0, 0.0);
        let (gerstner_disp, _) = gerstner.calculate(0.0, 0.0, 0.0);

        // Sine at 0 should be 0
        assert!(sine_disp.abs() < 1e-6);
        // Gerstner at 0 uses cos(0) = 1, so displacement = amplitude
        assert!((gerstner_disp - 1.0).abs() < 1e-6);
    }
}
