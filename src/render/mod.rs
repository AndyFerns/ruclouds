pub mod buffer;
pub mod halfblock;
pub mod palette;

use buffer::FrameBuffer;
use halfblock::{Cell, UPPER_HALF_BLOCK};
use palette::{blend, builtin_palettes, to_crossterm_color, Palette, Rgb};

use crate::config::Config;
use crate::sim::Simulation;

use crossterm::{cursor, queue, style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor}};

/// Renders simulation state into a frame buffer and emits only the diff
/// as ANSI commands into a provided byte buffer.
pub struct Renderer {
    current: FrameBuffer,
    previous: FrameBuffer,
    palettes: Vec<Palette>,
    custom_palette: Option<Palette>,
    first_frame: bool,
}

impl Renderer {
    pub fn new(width: u16, height: u16, custom_palette: Option<Palette>) -> Self {
        Renderer {
            current: FrameBuffer::new(width, height),
            previous: FrameBuffer::new(width, height),
            palettes: builtin_palettes(),
            custom_palette,
            first_frame: true,
        }
    }

    /// Total number of available palettes (built-in + optional custom).
    pub fn palette_count(&self) -> usize {
        self.palettes.len() + if self.custom_palette.is_some() { 1 } else { 0 }
    }

    /// Resize both frame buffers to the new terminal dimensions.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.current.resize(width, height);
        self.previous.resize(width, height);
    }

    /// Compute the current frame from the simulation, diff against the
    /// previous frame, and write only changed cells as ANSI commands
    /// into `buf`.
    pub fn render_frame(
        &mut self,
        sim: &Simulation,
        config: &Config,
        force_full: bool,
        buf: &mut Vec<u8>,
    ) -> anyhow::Result<()> {
        let width = self.current.width;
        let height = self.current.height;
        let sub_height = height as f64 * 2.0;

        // Extract palette data into locals to avoid borrowing `self` across
        // the mutable `self.current.set()` call below.
        let palette = if config.palette_index < self.palettes.len() {
            &self.palettes[config.palette_index]
        } else {
            self.custom_palette
                .as_ref()
                .expect("custom palette index out of range")
        };
        let cloud_light = palette.cloud_light;
        let cloud_dark = palette.cloud_dark;
        let sky_top = palette.sky_top;
        let sky_bottom = palette.sky_bottom;

        // ── Fill the current buffer ──────────────────────────────────────
        for cy in 0..height {
            for cx in 0..width {
                let sub_y_top = cy as f64 * 2.0;
                let sub_y_bottom = sub_y_top + 1.0;

                let top_color = compute_pixel_color(
                    cx as f64, sub_y_top, sub_height, sim, config,
                    cloud_light, cloud_dark, sky_top, sky_bottom,
                );
                let bottom_color = compute_pixel_color(
                    cx as f64, sub_y_bottom, sub_height, sim, config,
                    cloud_light, cloud_dark, sky_top, sky_bottom,
                );

                self.current.set(cx, cy, Cell::new(top_color, bottom_color));
            }
        }

        // ── Diff & emit ──────────────────────────────────────────────────
        let do_full = force_full || self.first_frame;
        self.first_frame = false;

        for cy in 0..height {
            let mut cursor_valid = false; // true when cursor is at (cx, cy)
            for cx in 0..width {
                let cell = *self.current.get(cx, cy);
                if !do_full && cell.same_as(self.previous.get(cx, cy)) {
                    cursor_valid = false;
                    continue;
                }
                if !cursor_valid {
                    queue!(buf, cursor::MoveTo(cx, cy))?;
                }
                let fg = to_crossterm_color(cell.top, config.effective_color_mode);
                let bg = to_crossterm_color(cell.bottom, config.effective_color_mode);
                queue!(
                    buf,
                    SetForegroundColor(fg),
                    SetBackgroundColor(bg),
                    Print(UPPER_HALF_BLOCK)
                )?;
                cursor_valid = true; // cursor auto-advanced one column
            }
        }

        queue!(buf, ResetColor)?;

        // ── Swap buffers ─────────────────────────────────────────────────
        std::mem::swap(&mut self.current, &mut self.previous);

        Ok(())
    }
}

/// Pure function: compute the final colour for a single sub-pixel.
fn compute_pixel_color(
    px: f64,
    py: f64,
    sub_height: f64,
    sim: &Simulation,
    config: &Config,
    cloud_light: Rgb,
    cloud_dark: Rgb,
    sky_top: Rgb,
    sky_bottom: Rgb,
) -> Rgb {
    let (opacity, shade) = sim.sample_pixel(px, py, config);

    // Sky gradient (top → bottom of viewport)
    let sky_color = if config.no_sky {
        Rgb::new(0, 0, 0)
    } else {
        let t = py / sub_height;
        blend(sky_top, sky_bottom, t)
    };

    // Cloud colour modulated by shading (shade=0 → dark, shade=1 → light)
    let cloud_color = blend(cloud_dark, cloud_light, shade);

    // Composite cloud over sky by opacity
    blend(sky_color, cloud_color, opacity)
}
