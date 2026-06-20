pub mod noise_field;
pub mod wind;

use crate::config::Config;
use noise_field::NoiseField;
use wind::Wind;

/// Spatial scale applied to pixel coordinates before noise sampling.
/// Lower = larger, puffier clouds.  Higher = finer detail.
const SPATIAL_SCALE: f64 = 0.03;

/// Time-axis scale fed into the 3rd dimension of Perlin noise.
const TIME_SCALE: f64 = 0.15;

/// Coordinate shift for the shading sample (simulates light from upper-left).
const SHADE_OFFSET: f64 = 0.4;

pub struct Simulation {
    pub noise: NoiseField,
    pub wind: Wind,
    pub time: f64,
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        Simulation {
            noise: NoiseField::new(seed as u32),
            wind: Wind::new(),
            time: 0.0,
        }
    }

    /// Advance simulation by one fixed timestep.
    pub fn update(&mut self, dt: f64, config: &Config) {
        self.time += dt * config.effective_speed() as f64;
        self.wind.update(config.wind_speed, config.wind_angle, dt);
    }

    /// Pure function: given a sub-pixel position `(px, py)`, return
    /// `(opacity, shade)` both in `[0.0, 1.0]`.
    ///
    /// This intentionally depends only on `(px, py, time, wind_offset, config)`
    /// and **never** on the grid width/height, so resize is seamless.
    pub fn sample_pixel(&self, px: f64, py: f64, config: &Config) -> (f64, f64) {
        // Step 1: wind offset
        let x = px * SPATIAL_SCALE + self.wind.offset_x;
        let y = py * SPATIAL_SCALE + self.wind.offset_y;
        let t = self.time * TIME_SCALE;

        // Step 2: domain warp
        let (dx, dy) = self.noise.sample_warp(x, y, t, config.warp_intensity());
        let warped_x = x + dx;
        let warped_y = y + dy;

        // Step 3: fBm  (time is the 3rd axis → smooth animation, no popping)
        let fbm = self.noise.sample_fbm(warped_x, warped_y, t);

        // Step 4: density threshold via smoothstep
        let density = config.effective_density() as f64;
        let edge = config.edge_softness as f64;
        let opacity = smoothstep(density - edge, density + edge, fbm);

        // Step 5: shading — second sample shifted up-left for puffy depth
        let shade = self.noise.sample_fbm(
            warped_x - SHADE_OFFSET,
            warped_y - SHADE_OFFSET,
            t,
        );

        (opacity, shade)
    }

    /// Reseed the noise fields and reset simulation time.
    pub fn reseed(&mut self, seed: u64) {
        self.noise = NoiseField::new(seed as u32);
        self.time = 0.0;
        // Wind offset is NOT reset — keeps spatial continuity
    }
}

/// Hermite smoothstep: 0 below `edge0`, 1 above `edge1`, smooth in between.
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if edge1 <= edge0 {
        return if x >= edge0 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
