/// Accumulated wind offset that drifts the noise-field coordinates each tick.
pub struct Wind {
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Wind {
    pub fn new() -> Self {
        Wind {
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Advance the wind offset by `dt` seconds.
    ///
    /// The offset is *subtracted* from noise coordinates so that
    /// `angle_deg = 0` produces left-to-right cloud drift.
    pub fn update(&mut self, speed: f32, angle_deg: f32, dt: f64) {
        let angle_rad = (angle_deg as f64).to_radians();
        self.offset_x -= speed as f64 * dt * angle_rad.cos();
        self.offset_y -= speed as f64 * dt * angle_rad.sin();
    }
}
