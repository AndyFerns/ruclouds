use crate::actions::Action;
use crate::config::{CliArgs, ColorMode, Config};
use crate::input::poll_action;
use crate::render::palette::{builtin_palettes, find_palette_index, parse_custom_palette, Palette};
use crate::render::Renderer;
use crate::sim::Simulation;
use crate::term::capability::detect_color_mode;

use anyhow::Result;
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const WIND_ANGLES: [f32; 8] = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];

pub fn run(args: CliArgs) -> Result<()> {
    // ── Resolve seed ─────────────────────────────────────────────────────
    let seed = args.seed.unwrap_or_else(|| rand::thread_rng().gen());

    // ── Resolve palette ──────────────────────────────────────────────────
    let (palette_index, custom_palette) = resolve_palette(&args.palette);

    // ── Resolve colour mode ──────────────────────────────────────────────
    let effective_color_mode = if args.color_mode == ColorMode::Auto {
        detect_color_mode()
    } else {
        args.color_mode
    };

    let mut config = Config {
        speed: args.speed,
        density: args.density,
        edge_softness: 0.15,
        palette_index,
        wind_speed: args.wind_speed,
        wind_angle: args.wind_angle,
        fps: args.fps.max(1),
        seed,
        effective_color_mode,
        no_sky: args.no_sky,
        storm_mode: false,
        warp_strong: false,
    };

    let (init_w, init_h) = crossterm::terminal::size()?;
    let mut sim = Simulation::new(seed);
    let mut renderer = Renderer::new(init_w, init_h, custom_palette);
    let mut prev_size = (init_w, init_h);
    let mut wind_angle_idx = nearest_wind_angle_index(args.wind_angle);

    let frame_duration = Duration::from_secs_f64(1.0 / config.fps as f64);
    let dt = 1.0 / config.fps as f64;

    let mut render_buf: Vec<u8> = Vec::with_capacity(65_536);

    // ── Main loop ────────────────────────────────────────────────────────
    loop {
        let frame_start = Instant::now();

        // ── Resize check ─────────────────────────────────────────────────
        let (new_w, new_h) = crossterm::terminal::size()?;
        let mut force_full = false;
        if (new_w, new_h) != prev_size {
            renderer.resize(new_w, new_h);
            prev_size = (new_w, new_h);
            force_full = true;
        }

        // ── Drain input (non-blocking) ───────────────────────────────────
        while let Some(action) = poll_action() {
            match action {
                Action::Quit => return Ok(()),
                Action::IncreaseSpeed => {
                    config.speed = (config.speed + 0.1).min(5.0);
                }
                Action::DecreaseSpeed => {
                    config.speed = (config.speed - 0.1).max(0.1);
                }
                Action::IncreaseDensity => {
                    config.density = (config.density + 0.05).min(1.0);
                }
                Action::DecreaseDensity => {
                    config.density = (config.density - 0.05).max(0.0);
                }
                Action::CyclePalette => {
                    config.palette_index =
                        (config.palette_index + 1) % renderer.palette_count();
                }
                Action::CycleWind => {
                    wind_angle_idx = (wind_angle_idx + 1) % WIND_ANGLES.len();
                    config.wind_angle = WIND_ANGLES[wind_angle_idx];
                }
                Action::ToggleWarp => {
                    config.warp_strong = !config.warp_strong;
                }
                Action::Reseed => {
                    let new_seed: u64 = rand::thread_rng().gen();
                    config.seed = new_seed;
                    sim.reseed(new_seed);
                }
                Action::ToggleStorm => {
                    config.storm_mode = !config.storm_mode;
                }
            }
        }

        // ── Update simulation ────────────────────────────────────────────
        sim.update(dt, &config);

        // ── Render into byte buffer, then flush to stdout ────────────────
        render_buf.clear();
        renderer.render_frame(&sim, &config, force_full, &mut render_buf)?;
        {
            let mut stdout = io::stdout().lock();
            stdout.write_all(&render_buf)?;
            stdout.flush()?;
        }

        // ── Sleep remainder of frame budget (no debt accumulation) ───────
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn resolve_palette(name: &str) -> (usize, Option<Palette>) {
    if let Some(idx) = find_palette_index(name) {
        return (idx, None);
    }
    if let Some(custom) = parse_custom_palette(name) {
        let idx = builtin_palettes().len(); // custom sits at the end
        return (idx, Some(custom));
    }
    // Fallback: default palette
    (0, None)
}

fn nearest_wind_angle_index(angle: f32) -> usize {
    WIND_ANGLES
        .iter()
        .position(|&a| (a - angle).abs() < 1.0)
        .unwrap_or(0)
}
