use crate::config::ColorMode;

// ── RGB type ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Rgb { r, g, b })
    }
}

// ── Palette ─────────────────────────────────────────────────────────────────

pub struct Palette {
    pub name: &'static str,
    pub cloud_light: Rgb,
    pub cloud_dark: Rgb,
    pub sky_top: Rgb,
    pub sky_bottom: Rgb,
}

pub fn builtin_palettes() -> Vec<Palette> {
    vec![
        Palette {
            name: "white-grey",
            cloud_light: Rgb::new(245, 245, 250),
            cloud_dark: Rgb::new(160, 165, 180),
            sky_top: Rgb::new(135, 185, 235),
            sky_bottom: Rgb::new(60, 110, 195),
        },
        Palette {
            name: "sunset",
            cloud_light: Rgb::new(255, 200, 140),
            cloud_dark: Rgb::new(200, 100, 80),
            sky_top: Rgb::new(255, 155, 80),
            sky_bottom: Rgb::new(120, 40, 100),
        },
        Palette {
            name: "midnight",
            cloud_light: Rgb::new(120, 130, 165),
            cloud_dark: Rgb::new(50, 55, 80),
            sky_top: Rgb::new(15, 20, 50),
            sky_bottom: Rgb::new(5, 5, 20),
        },
        Palette {
            name: "storm",
            cloud_light: Rgb::new(145, 150, 160),
            cloud_dark: Rgb::new(60, 62, 72),
            sky_top: Rgb::new(80, 92, 115),
            sky_bottom: Rgb::new(38, 42, 58),
        },
    ]
}

pub fn find_palette_index(name: &str) -> Option<usize> {
    builtin_palettes().iter().position(|p| p.name == name)
}

pub fn parse_custom_palette(s: &str) -> Option<Palette> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        let light = Rgb::from_hex(parts[0].trim())?;
        let dark = Rgb::from_hex(parts[1].trim())?;
        return Some(Palette {
            name: "custom",
            cloud_light: light,
            cloud_dark: dark,
            sky_top: Rgb::new(100, 150, 220),
            sky_bottom: Rgb::new(45, 85, 165),
        });
    }
    None
}

// ── Color blending ──────────────────────────────────────────────────────────

/// Linear interpolation: `blend(a, b, 0.0) == a`, `blend(a, b, 1.0) == b`.
pub fn blend(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Rgb {
        r: (a.r as f64 * inv + b.r as f64 * t).round() as u8,
        g: (a.g as f64 * inv + b.g as f64 * t).round() as u8,
        b: (a.b as f64 * inv + b.b as f64 * t).round() as u8,
    }
}

// ── Color-mode conversion ───────────────────────────────────────────────────

/// Convert an RGB colour to the nearest xterm 256-colour index.
pub fn rgb_to_256(c: Rgb) -> u8 {
    // Check for near-grey first (greyscale ramp 232-255)
    let max_diff = (c.r as i16 - c.g as i16)
        .abs()
        .max((c.g as i16 - c.b as i16).abs())
        .max((c.r as i16 - c.b as i16).abs());

    if max_diff < 15 {
        let avg = (c.r as u16 + c.g as u16 + c.b as u16) / 3;
        if avg < 8 {
            return 16; // closest black in cube
        }
        if avg > 248 {
            return 231; // closest white in cube
        }
        // greyscale ramp: 232 → grey(8), 255 → grey(238)
        return ((avg as f64 - 8.0) / 10.0).round().clamp(0.0, 23.0) as u8 + 232;
    }

    // 6×6×6 colour cube (indices 16–231)
    let ri = ((c.r as f64 / 255.0 * 5.0).round() as u8).min(5);
    let gi = ((c.g as f64 / 255.0 * 5.0).round() as u8).min(5);
    let bi = ((c.b as f64 / 255.0 * 5.0).round() as u8).min(5);
    16 + 36 * ri + 6 * gi + bi
}

/// Standard ANSI-16 reference colours (VGA palette).
const ANSI16: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // 0  Black
    (170, 0, 0),     // 1  Red
    (0, 170, 0),     // 2  Green
    (170, 85, 0),    // 3  Yellow / Brown
    (0, 0, 170),     // 4  Blue
    (170, 0, 170),   // 5  Magenta
    (0, 170, 170),   // 6  Cyan
    (170, 170, 170), // 7  White
    (85, 85, 85),    // 8  Bright Black
    (255, 85, 85),   // 9  Bright Red
    (85, 255, 85),   // 10 Bright Green
    (255, 255, 85),  // 11 Bright Yellow
    (85, 85, 255),   // 12 Bright Blue
    (255, 85, 255),  // 13 Bright Magenta
    (85, 255, 255),  // 14 Bright Cyan
    (255, 255, 255), // 15 Bright White
];

/// Convert an RGB colour to the nearest ANSI-16 index (Euclidean distance).
pub fn rgb_to_ansi16(c: Rgb) -> u8 {
    let mut best = 0u8;
    let mut best_dist = u32::MAX;
    for (i, &(r, g, b)) in ANSI16.iter().enumerate() {
        let dr = c.r as i32 - r as i32;
        let dg = c.g as i32 - g as i32;
        let db = c.b as i32 - b as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = i as u8;
        }
    }
    best
}

/// Map an `Rgb` to the appropriate `crossterm::style::Color` for the active
/// colour mode.
pub fn to_crossterm_color(rgb: Rgb, mode: ColorMode) -> crossterm::style::Color {
    use crossterm::style::Color;
    match mode {
        ColorMode::TrueColor | ColorMode::Auto => Color::Rgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        },
        ColorMode::Color256 => Color::AnsiValue(rgb_to_256(rgb)),
        ColorMode::Ansi16 => {
            // Use named colours so crossterm emits classic SGR codes (30–37, 90–97)
            match rgb_to_ansi16(rgb) {
                0 => Color::Black,
                1 => Color::DarkRed,
                2 => Color::DarkGreen,
                3 => Color::DarkYellow,
                4 => Color::DarkBlue,
                5 => Color::DarkMagenta,
                6 => Color::DarkCyan,
                7 => Color::Grey,
                8 => Color::DarkGrey,
                9 => Color::Red,
                10 => Color::Green,
                11 => Color::Yellow,
                12 => Color::Blue,
                13 => Color::Magenta,
                14 => Color::Cyan,
                15 => Color::White,
                _ => Color::Reset,
            }
        }
    }
}
