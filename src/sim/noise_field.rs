use noise::{NoiseFn, Perlin};

/// Precomputed Perlin fBm field with a separate warp noise instance.
pub struct NoiseField {
    primary: Perlin,
    warp: Perlin,
    /// (frequency_multiplier, amplitude) for each fBm octave
    octaves: Vec<(f64, f64)>,
    /// Sum of all octave amplitudes, for normalisation
    max_amplitude: f64,
}

impl NoiseField {
    pub fn new(seed: u32) -> Self {
        let primary = Perlin::new(seed);
        // Independent seed for domain warp so it doesn't correlate with primary
        let warp = Perlin::new(seed.wrapping_add(7919));

        // 4-octave fBm: frequency doubles, amplitude halves
        let octaves = vec![
            (1.0, 1.0),
            (2.0, 0.5),
            (4.0, 0.25),
            (8.0, 0.125),
        ];
        let max_amplitude: f64 = octaves.iter().map(|(_, a)| a).sum();

        NoiseField {
            primary,
            warp,
            octaves,
            max_amplitude,
        }
    }

    /// Sample fractal Brownian motion at `(x, y, z)`.
    ///
    /// Returns a value in `[0.0, 1.0]`.
    pub fn sample_fbm(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut value = 0.0;
        for &(freq, amp) in &self.octaves {
            value += self.primary.get([x * freq, y * freq, z * freq]) * amp;
        }
        // Perlin range is approx [-1, 1]; normalise summed range to [0, 1]
        ((value / self.max_amplitude) + 1.0) * 0.5
    }

    /// Compute domain-warp displacement `(dx, dy)`.
    ///
    /// Uses the warp Perlin at a low frequency to bend the coordinate
    /// space before the primary fBm sample, giving wispy cloud shapes.
    pub fn sample_warp(&self, x: f64, y: f64, z: f64, intensity: f64) -> (f64, f64) {
        let warp_freq = 0.3;
        let dx = self.warp.get([x * warp_freq, y * warp_freq, z * 0.3]) * intensity;
        let dy = self.warp.get([
            (x + 100.0) * warp_freq,
            (y + 100.0) * warp_freq,
            z * 0.3,
        ]) * intensity;
        (dx, dy)
    }
}
