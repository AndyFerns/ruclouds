use crate::config::ColorMode;
use std::env;

/// Detect the terminal's color capability from environment variables.
pub fn detect_color_mode() -> ColorMode {
    // COLORTERM is the most reliable indicator
    if let Ok(ct) = env::var("COLORTERM") {
        let ct = ct.to_lowercase();
        if ct == "truecolor" || ct == "24bit" {
            return ColorMode::TrueColor;
        }
    }

    // Windows Terminal always supports truecolor
    if env::var("WT_SESSION").is_ok() {
        return ColorMode::TrueColor;
    }

    // TERM_PROGRAM for macOS / known terminals
    if let Ok(tp) = env::var("TERM_PROGRAM") {
        let tp = tp.to_lowercase();
        if tp.contains("iterm")
            || tp.contains("hyper")
            || tp.contains("wezterm")
            || tp.contains("alacritty")
        {
            return ColorMode::TrueColor;
        }
    }

    // TERM variable
    if let Ok(term) = env::var("TERM") {
        let term = term.to_lowercase();
        if term.contains("kitty") || term.contains("alacritty") {
            return ColorMode::TrueColor;
        }
        if term.contains("256color") {
            return ColorMode::Color256;
        }
    }

    // Safe fallback
    ColorMode::Color256
}
