use clap::Parser;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    TrueColor,
    Color256,
    Ansi16,
}

impl FromStr for ColorMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "truecolor" => Ok(ColorMode::TrueColor),
            "256" => Ok(ColorMode::Color256),
            "ansi16" => Ok(ColorMode::Ansi16),
            _ => Err(format!("Unknown color mode: '{}'. Expected: auto, truecolor, 256, ansi16", s)),
        }
    }
}

impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::TrueColor => write!(f, "truecolor"),
            ColorMode::Color256 => write!(f, "256"),
            ColorMode::Ansi16 => write!(f, "ansi16"),
        }
    }
}

/// Animated terminal clouds using noise-field simulation
#[derive(Parser, Debug)]
#[command(name = "ruclouds", version, about)]
pub struct CliArgs {
    /// Animation speed multiplier
    #[arg(long, default_value_t = 1.0)]
    pub speed: f32,

    /// Cloud density threshold (0.0–1.0)
    #[arg(long, default_value_t = 0.5)]
    pub density: f32,

    /// Palette name or custom hex pair (e.g. "sunset" or "AABBCC,112233")
    #[arg(long, default_value = "white-grey")]
    pub palette: String,

    /// Wind speed
    #[arg(long, default_value_t = 0.3)]
    pub wind_speed: f32,

    /// Wind angle in degrees (0 = left-to-right drift)
    #[arg(long, default_value_t = 0.0)]
    pub wind_angle: f32,

    /// Target frames per second
    #[arg(long, default_value_t = 30)]
    pub fps: u32,

    /// Random seed (random if omitted)
    #[arg(long)]
    pub seed: Option<u64>,

    /// Color mode: auto, truecolor, 256, ansi16
    #[arg(long, default_value = "auto")]
    pub color_mode: ColorMode,

    /// Use terminal background instead of sky gradient
    #[arg(long)]
    pub no_sky: bool,
}

pub struct Config {
    pub speed: f32,
    pub density: f32,
    pub edge_softness: f32,
    pub palette_index: usize,
    pub wind_speed: f32,
    pub wind_angle: f32,
    pub fps: u32,
    pub seed: u64,
    pub effective_color_mode: ColorMode,
    pub no_sky: bool,
    pub storm_mode: bool,
    pub warp_strong: bool,
}

impl Config {
    /// Speed with storm-mode boost applied
    pub fn effective_speed(&self) -> f32 {
        if self.storm_mode {
            self.speed * 3.0
        } else {
            self.speed
        }
    }

    /// Density with storm-mode boost applied
    pub fn effective_density(&self) -> f32 {
        if self.storm_mode {
            (self.density + 0.2).min(1.0)
        } else {
            self.density
        }
    }

    /// Domain-warp intensity (subtle ↔ strong toggle)
    pub fn warp_intensity(&self) -> f64 {
        if self.warp_strong {
            2.0
        } else {
            0.5
        }
    }
}
